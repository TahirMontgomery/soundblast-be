use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::shared::global::GlobalState;

#[derive(Deserialize)]
struct TranscribeFileRequest {
    id: String,
}

async fn transcribe_file(
    State(state): State<GlobalState>,
    Json(data): Json<TranscribeFileRequest>,
) -> (StatusCode, Json<Value>) {
    match state
        .services
        .transcription()
        .transcribe_file(&data.id)
        .await
    {
        Ok(result) => (
            StatusCode::CREATED,
            Json(json!({"success": true, "result": result})),
        ),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))),
    }
}

pub fn router() -> Router<GlobalState> {
    Router::new().route("/", post(transcribe_file))
}
