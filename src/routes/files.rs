use crate::shared::global::GlobalState;
use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde_json::{json, Value};
use tempfile::NamedTempFile;
use tower_http::services::fs::ServeFile;

#[derive(TryFromMultipart)]
struct CreateUserRequest {
    #[form_data(limit = "unlimited")]
    pub file: FieldData<NamedTempFile>,
    pub thumbnail: String,
}

async fn upload_file(
    State(state): State<GlobalState>,
    data: TypedMultipart<CreateUserRequest>,
) -> (StatusCode, Json<Value>) {
    match state
        .services
        .files()
        .save_file(data.0.file, Some(&data.0.thumbnail))
        .await
    {
        Ok(id) => (StatusCode::CREATED, Json(json!({"success": true,"id": id}))),
        Err(msg) => (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))),
    }
}

async fn list_files(State(state): State<GlobalState>) -> (StatusCode, Json<Value>) {
    match state.services.files().list_files().await {
        Ok(files) => (
            StatusCode::CREATED,
            Json(json!({"success": true,"files": files})),
        ),
        Err(msg) => (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))),
    }
}

async fn download_file(
    State(state): State<GlobalState>,
    Path(id): Path<String>,
    request: Request,
) -> impl IntoResponse {
    let (output_path, file) = match state.services.files().get_download_path(&id).await {
        Ok(s) => s,
        Err(msg) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": msg})))),
    };

    let mut res = match ServeFile::new(output_path).try_call(request).await {
        Ok(s) => s,
        Err(msg) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": msg.to_string()})),
            ))
        }
    };

    res.headers_mut().append(
        header::CONTENT_TYPE,
        file.metadata.content_type.parse().unwrap(),
    );
    res.headers_mut().append(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename={}", file.filename)
            .parse()
            .unwrap(),
    );

    Ok(res)
}

pub fn router() -> Router<GlobalState> {
    Router::new()
        .route("/", post(upload_file))
        .route("/list", get(list_files))
        .route("/download/:id", get(download_file))
}
