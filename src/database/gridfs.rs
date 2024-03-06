use futures::StreamExt;
use futures_util::io::AsyncWriteExt;
use futures_util::TryStreamExt;
use log::info;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    gridfs::FilesCollectionDocument,
    options::GridFsUploadOptions,
    Database,
};
use std::{fs::File, path::PathBuf, str::FromStr};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::shared::errors::extract_error;

pub struct GridFs {
    db: Database,
}

impl GridFs {
    pub fn new(db: Database) -> Self {
        GridFs { db }
    }

    pub async fn upload_file(
        &self,
        file: File,
        filename: &str,
        metadata: Document,
    ) -> Result<String, String> {
        let bucket = self.db.gridfs_bucket(None);
        let mut upload_stream = bucket
            .open_upload_stream(
                filename,
                GridFsUploadOptions::builder().metadata(metadata).build(),
            )
            .compat_write();

        let mut tok_file = tokio::fs::File::from_std(file);
        let bytes_copied = tokio::io::copy(&mut tok_file, &mut upload_stream)
            .await
            .map_err(extract_error)?;

        upload_stream
            .get_mut()
            .close()
            .await
            .map_err(extract_error)?;

        info!("Bytes written to mongo: {}", bytes_copied);
        Ok(upload_stream.get_mut().id().to_string())
    }

    pub async fn find_one_by_id(
        &self,
        id: &str,
    ) -> Result<Option<FilesCollectionDocument>, String> {
        let bucket = self.db.gridfs_bucket(None);
        let oid = ObjectId::from_str(id).map_err(|err| err.to_string())?;
        let filter = doc! {"_id": oid};

        let mut results = match bucket.find(filter, None).await {
            Ok(f) => f,
            Err(err) => return Err(err.to_string()),
        };

        match results.try_next().await {
            Ok(f) => return Ok(f),
            Err(err) => return Err(err.to_string()),
        };
    }

    pub async fn list(&self) -> Result<Vec<FilesCollectionDocument>, String> {
        let bucket = self.db.gridfs_bucket(None);
        let filter = doc! {};

        let results = bucket.find(filter, None).await.map_err(|e| e.to_string())?;

        let output: Vec<FilesCollectionDocument> =
            results.map(|file| file.unwrap()).collect().await;

        Ok(output)
    }

    pub async fn download_file_to_disk(
        &self,
        file: FilesCollectionDocument,
        path: PathBuf,
    ) -> Result<(), String> {
        let bucket = self.db.gridfs_bucket(None);
        let download_stream = bucket
            .open_download_stream(file.id)
            .await
            .map_err(|e| e.to_string())?;
        let mut tokio_upload_stream = download_stream.compat();

        let output_file = tokio::fs::File::create(path).await.map_err(extract_error)?;
        let mut buf_writer = tokio::io::BufWriter::new(output_file);

        let bytes_copied = tokio::io::copy(&mut tokio_upload_stream, &mut buf_writer)
            .await
            .map_err(extract_error)?;

        info!("Bytes written to disk: {}", bytes_copied);
        Ok(())
    }
}
