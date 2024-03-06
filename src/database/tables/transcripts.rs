use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperResultWord {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperResultSegment {
    pub id: u32,
    pub seek: u32,
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub tokens: Vec<u32>,
    pub confidence: f32,
    pub words: Vec<WhisperResultWord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperResult {
    pub file_id: Option<String>,
    pub text: String,
    pub segments: Vec<WhisperResultSegment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptDocument {
    pub _id: ObjectId,
    pub file_id: String,
    pub text: String,
    pub segments: Vec<WhisperResultSegment>,
}

impl From<TranscriptDocument> for WhisperResult {
    fn from(document: TranscriptDocument) -> Self {
        WhisperResult {
            file_id: Some(document.file_id),
            text: document.text,
            segments: document.segments,
        }
    }
}

pub struct Transcripts {
    db: Database,
}

impl Transcripts {
    pub fn new(db: Database) -> Self {
        Transcripts { db }
    }

    pub async fn insert(&self, doc: &WhisperResult) -> Result<String, String> {
        let coll: Collection<WhisperResult> = self.db.collection("transcripts");
        let res = coll
            .insert_one(doc, None)
            .await
            .map_err(|e| e.to_string())?;

        Ok(res.inserted_id.to_string())
    }

    pub async fn find_by_file_id(&self, id: &str) -> Result<Option<TranscriptDocument>, String> {
        let coll: Collection<TranscriptDocument> = self.db.collection("transcripts");
        let filter = doc! {"file_id": id};

        let results = match coll.find_one(filter, None).await {
            Ok(f) => f,
            Err(err) => return Err(err.to_string()),
        };

        Ok(results)
    }
}
