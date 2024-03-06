use mongodb::{bson::doc, options::ClientOptions, Client, Database};
use std::env;

use super::{gridfs::GridFs, tables::transcripts::Transcripts};

#[derive(Clone)]

pub struct DB {
    // client: Client,
    db: Database,
}

impl DB {
    pub async fn new() -> Self {
        let user = env::var("MONGO_USER").unwrap();
        let password = env::var("MONGO_PASSWORD").unwrap();
        let host = env::var("MONGO_HOST").unwrap();
        let port = env::var("MONGO_PORT").unwrap();

        let conn_str = format!(
            "mongodb://{}:{}@{}:{}/?authSource=admin",
            user, password, host, port
        );
        let client_opts = ClientOptions::parse(conn_str).await.unwrap();
        let client = Client::with_options(client_opts).unwrap();
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await
            .unwrap();

        let db = client.database("soundblast");
        DB { db }
    }

    pub fn gridfs(&self) -> GridFs {
        GridFs::new(self.db.clone())
    }

    pub fn transcripts(&self) -> Transcripts {
        Transcripts::new(self.db.clone())
    }
}
