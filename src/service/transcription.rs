use std::{env, path::Path, process::Stdio};
use tokio::{fs, io::AsyncReadExt, process::Command};

use crate::{
    database::{db::DB, tables::transcripts::WhisperResult},
    shared::errors::extract_error,
};

pub struct TranscriptionService {
    db: DB,
}

impl TranscriptionService {
    pub fn new(db: DB) -> Self {
        TranscriptionService { db }
    }

    pub async fn transcribe_file(&self, id: &str) -> Result<WhisperResult, String> {
        let file = match self.db.gridfs().find_one_by_id(id).await? {
            Some(f) => f,
            None => return Err("Could not find file".to_string()),
        };

        let file_id = file
            .id
            .as_object_id()
            .ok_or("Invalid object id")?
            .to_string();

        if let Some(existing_transcript) =
            match self.db.transcripts().find_by_file_id(&file_id).await {
                Ok(res) => res,
                Err(err) => {
                    println!("Error occurred: {:?}", err);
                    None
                }
            }
        {
            return Ok(existing_transcript.into());
        }

        let file_dir = env::var("FILE_DIR").unwrap();

        let output_path = Path::new(&file_dir).join(&file_id);

        self.db
            .gridfs()
            .download_file_to_disk(file, output_path.clone())
            .await?;

        let output_file = Path::new(&file_dir).join(format!("{}.wav", &file_id));

        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(&output_path)
            .arg("-vn")
            .arg("-ar")
            .arg("16000")
            .arg("-ac")
            .arg("1")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg(&output_file)
            .output()
            .await
            .map_err(extract_error)?;

        fs::remove_file(&output_path).await.map_err(extract_error)?;

        if !ffmpeg.status.success() {
            return Err(String::from_utf8_lossy(&ffmpeg.stderr).to_string());
        }

        let mut whisper = Command::new("whisper_timestamped")
            .arg("--output_dir")
            .arg(&file_dir)
            .arg("--output_format")
            .arg("json")
            .arg(&output_file)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(extract_error)?;

        let whisper_status = whisper.wait().await.map_err(extract_error)?;

        if !whisper_status.success() {
            return Err(String::from_utf8_lossy(&ffmpeg.stderr).to_string());
        }

        fs::remove_file(&output_file).await.map_err(extract_error)?;

        let f = format!("{}.words.json", output_file.to_str().unwrap());
        let transcript_file_path = Path::new(&f);
        let mut transcript_file = tokio::fs::File::open(&transcript_file_path)
            .await
            .map_err(extract_error)?;

        let mut buf = String::new();
        transcript_file
            .read_to_string(&mut buf)
            .await
            .map_err(extract_error)?;

        fs::remove_file(&transcript_file_path)
            .await
            .map_err(extract_error)?;

        let mut result: WhisperResult = serde_json::from_str(&buf).map_err(|e| e.to_string())?;
        result.file_id = Some(file_id);

        self.db.transcripts().insert(&result).await?;

        Ok(result)
    }
}
