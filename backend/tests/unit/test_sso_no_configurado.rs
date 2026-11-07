//! T035 — GET /api/auth/sso/{proveedor}/iniciar sin credenciales → 503 `sso_no_configurado`.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::{cuerpo_json, pool_de_test};
use serde_json::{json, Value};
use stockmante_backend::app::{construir_router, AppEstado};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::config::AppConfig;
use stockmante_backend::mail::envio::CorreoDev;
use stockmante_backend::services::sso::SondaSso;
use std::sync::Arc;
use tower::ServiceExt;

fn config_con(google_id: &str, microsoft_id: &str) -> AppConfig {
    AppConfig {
        database_url: String::new(),
        app_url: "http://localhost:5173".to_string(),
        port: 3000,
        session_cookie_name: "stockmante_sesion".to_string(),
        session_expiry_hours: 8,
        email_from: "test@test.local".to_string(),
        smtp_url: None,
        app_env: "development".to_string(),
        docs_enabled: false,
        google_client_id: google_id.to_string(),
        google_client_secret: String::new(),
        microsoft_client_id: microsoft_id.to_string(),
        microsoft_client_secret: String::new(),
        sso_redirect_base: "http://localhost:3000".to_string(),
    }
}

async fn app_con(google_id: &str, microsoft_id: &str) -> (axum::Router, sqlx::PgPool) {
    let _ = dotenvy::dotenv();
    let pool = pool_de_test().await;
    let estado = AppEstado {
        pool: pool.clone(),
        config: config_con(google_id, microsoft_id),
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(CorreoDev::new()),
        salud_sso: Arc::new(SondaSso::fija(true, true)),
        asistente: Arc::new(stockmante_backend::services::asistente::Asistente::deshabilitado()),
    };
    (construir_router(estado), pool)
}

async fn pedir(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .expect("request");
    let res = app.oneshot(req).await.expect("request");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

#[tokio::test]
async fn sso_iniciar_sin_credenciales_devuelve_sso_no_configurado() {
    let (app, _pool) = app_con("", "").await;
    let (status, body) = pedir(app, "/api/auth/sso/google/iniciar").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["codigo"], json!("sso_no_configurado"));
}

/// Regresión: `iniciar` debe usar el client_id del proveedor pedido. Con Google
/// configurado y Microsoft sin credenciales, `/sso/microsoft/iniciar` tiene que
/// responder 503 (antes reusaba el client_id de Google y redirigía roto).
#[tokio::test]
async fn sso_iniciar_microsoft_sin_credenciales_devuelve_sso_no_configurado() {
    let (app, _pool) = app_con("g-id", "").await;
    let (status, body) = pedir(app, "/api/auth/sso/microsoft/iniciar").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["codigo"], json!("sso_no_configurado"));
}
