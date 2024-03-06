use std::{
    env,
    path::{Path, PathBuf},
};

use crate::database::db::DB;
use axum_typed_multipart::FieldData;
use mongodb::{bson::doc, gridfs::FilesCollectionDocument};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Clone)]
pub struct FileService {
    db: DB,
}

#[derive(Deserialize, Serialize)]
pub struct Metadata {
    pub content_type: String,
    pub thumbnail: String,
}

#[derive(Deserialize, Serialize)]
pub struct File {
    pub id: String,
    pub length: u64,
    pub metadata: Metadata,
    pub filename: String,
}

impl FileService {
    pub fn new(db: DB) -> Self {
        FileService { db }
    }

    pub async fn save_file(
        &self,
        file: FieldData<NamedTempFile>,
        thumbnail: Option<&str>,
    ) -> Result<String, String> {
        let filename = file
            .metadata
            .file_name
            .ok_or(String::from("Missing file name"))?;
        let content_type = file
            .metadata
            .content_type
            .ok_or(String::from("Missing content type"))?;

        let metadata = doc! {
            "contentType": content_type,
            "thumbnail": match thumbnail {
                Some(e) => e,
                None => ""
            }
        };

        let f = file.contents.into_file();
        self.db.gridfs().upload_file(f, &filename, metadata).await
    }

    pub async fn list_files(&self) -> Result<Vec<File>, String> {
        let files: Vec<File> = self
            .db
            .gridfs()
            .list()
            .await?
            .iter()
            .map(|f| self.extract_file_from_mongo(f).unwrap())
            .collect();

        Ok(files)
    }
    // pub async fn get_download_stream(
    //     &self,
    //     id: &str,
    // ) -> Result<(ReaderStream<tokio::fs::File>, File), String> {
    //     let file = match self.db.gridfs().find_one_by_id(id).await? {
    //         Some(f) => f,
    //         None => return Err("Could not find file".to_string()),
    //     };

    //     let file_id = file
    //         .id
    //         .as_object_id()
    //         .ok_or("Invalid object id")?
    //         .to_string();

    //     let file_dir = env::var("FILE_DIR").unwrap();
    //     let return_file = self.extract_file_from_mongo(&file)?;

    //     let new_filename = format!("{}_stream", &file_id);
    //     let output_path = Path::new(&file_dir).join(&new_filename);

    //     self.db
    //         .gridfs()
    //         .download_file_to_disk(file, output_path.clone())
    //         .await?;

    //     let file = tokio::fs::File::open(output_path)
    //         .await
    //         .map_err(extract_error)?;
    //     let stream: ReaderStream<tokio::fs::File> = ReaderStream::new(file);

    //     Ok((stream, return_file))
    // }

    pub async fn get_download_path(&self, id: &str) -> Result<(PathBuf, File), String> {
        let file = match self.db.gridfs().find_one_by_id(id).await? {
            Some(f) => f,
            None => return Err("Could not find file".to_string()),
        };

        let file_id = file
            .id
            .as_object_id()
            .ok_or("Invalid object id")?
            .to_string();

        let file_dir = env::var("FILE_DIR").unwrap();
        let return_file = self.extract_file_from_mongo(&file)?;

        let new_filename = format!("{}_stream", &file_id);
        let output_path = Path::new(&file_dir).join(&new_filename);

        self.db
            .gridfs()
            .download_file_to_disk(file, output_path.clone())
            .await?;

        Ok((output_path, return_file))
    }

    fn extract_file_from_mongo(&self, file: &FilesCollectionDocument) -> Result<File, String> {
        let id = file
            .id
            .as_object_id()
            .ok_or("Invalid Object ID")?
            .to_string();
        let content_type = file
            .metadata
            .as_ref()
            .ok_or("invalid metadata")?
            .get_str("contentType")
            .map_err(|e| e.to_string())?
            .to_string();
        let thumbnail = file
            .metadata
            .as_ref()
            .ok_or("invalid metadata")?
            .get_str("thumbnail")
            .map_err(|e| e.to_string())?
            .to_string();

        let filename = file.filename.as_ref().ok_or("invalid filename")?.to_owned();

        let metadata: Metadata = Metadata {
            content_type,
            thumbnail,
        };
        Ok(File {
            filename,
            metadata,
            id,
            length: file.length,
        })
    }
}
