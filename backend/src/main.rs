use axum::{routing::get, Json, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    let app = Router::new().route("/api/salud", get(salud));

    let addr = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{addr}"))
        .await
        .unwrap_or_else(|e| panic!("no se pudo escuchar en el puerto {addr}: {e}"));
    tracing::info!("backend escuchando en {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("error del servidor: {e}"));
}

async fn salud() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}
