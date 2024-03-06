use crate::database::db::DB;

use self::{files::FileService, transcription::TranscriptionService};

pub mod files;
pub mod transcription;

#[derive(Clone)]
pub struct Services {
    db: DB,
}

impl Services {
    pub fn new(db: DB) -> Self {
        Services { db }
    }

    pub fn files(&self) -> FileService {
        FileService::new(self.db.clone())
    }

    pub fn transcription(&self) -> TranscriptionService {
        TranscriptionService::new(self.db.clone())
    }
}
