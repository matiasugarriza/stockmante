pub mod auth;

use axum::Json;
use serde_json::json;

pub async fn salud() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}
