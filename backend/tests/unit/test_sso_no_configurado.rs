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
use std::sync::Arc;
use tower::ServiceExt;

async fn app_sin_google() -> (axum::Router, sqlx::PgPool) {
    let _ = dotenvy::dotenv();
    let pool = pool_de_test().await;
    let config = AppConfig {
        database_url: String::new(),
        app_url: "http://localhost:5173".to_string(),
        port: 3000,
        session_cookie_name: "stockmante_sesion".to_string(),
        session_expiry_hours: 8,
        email_from: "test@test.local".to_string(),
        smtp_url: None,
        app_env: "development".to_string(),
        docs_enabled: false,
        google_client_id: String::new(),
        google_client_secret: String::new(),
        microsoft_client_id: String::new(),
        microsoft_client_secret: String::new(),
        sso_redirect_base: "http://localhost:3000".to_string(),
    };
    let estado = AppEstado {
        pool: pool.clone(),
        config,
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(CorreoDev::new()),
    };
    (construir_router(estado), pool)
}

#[tokio::test]
async fn sso_iniciar_sin_credenciales_devuelve_sso_no_configurado() {
    let (app, _pool) = app_sin_google().await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/auth/sso/google/iniciar")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("request");
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body: Value = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], json!("sso_no_configurado"));
}
