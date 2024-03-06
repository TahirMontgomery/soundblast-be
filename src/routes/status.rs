use axum::{routing::get, Router};

use crate::shared::global::GlobalState;

async fn status() -> &'static str {
    "Up and running!"
}

pub fn router() -> Router<GlobalState> {
    Router::new().route("/", get(status))
}
