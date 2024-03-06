use axum::extract::DefaultBodyLimit;
use axum::http::header;
use axum::Router;

use dotenv::dotenv;
use log::info;
use routes::init_routes;
mod database;
mod routes;
mod service;
mod shared;

#[tokio::main]
async fn main() {
    dotenv().unwrap();

    env_logger::init();

    let db = database::db::DB::new().await;
    let state = shared::global::GlobalState {
        services: service::Services::new(db),
    };

    let app = Router::new()
        .nest("/api", init_routes())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(
                    "http://localhost:7676"
                        .parse::<axum::http::HeaderValue>()
                        .unwrap(),
                )
                .allow_credentials(true)
                .allow_headers([header::CONTENT_TYPE]), // .allow_headers(Any),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8899")
        .await
        .unwrap();

    info!("Server is now listening on port 8899");
    axum::serve(listener, app).await.unwrap();
}
