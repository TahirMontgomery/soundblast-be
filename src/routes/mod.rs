use axum::Router;

use crate::shared::global::GlobalState;
mod files;
mod status;
pub mod transcription;

pub fn init_routes() -> Router<GlobalState> {
    Router::new()
        .nest("/status", status::router())
        .nest("/files", files::router())
        .nest("/transcribe", transcription::router())
}
