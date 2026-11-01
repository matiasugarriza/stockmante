//! T034 — GET /api/auth/sso/config: disponibilidad de proveedores SSO.

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

async fn app_con_config(google_id: &str, google_secret: &str, ms_id: &str, ms_secret: &str) -> (axum::Router, sqlx::PgPool) {
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
        google_client_id: google_id.to_string(),
        google_client_secret: google_secret.to_string(),
        microsoft_client_id: ms_id.to_string(),
        microsoft_client_secret: ms_secret.to_string(),
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
async fn sso_config_sin_credenciales_devuelve_false() {
    let (app, _pool) = app_con_config("", "", "", "").await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/auth/sso/config")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = cuerpo_json(res).await;
    assert_eq!(body["google_disponible"], json!(false));
    assert_eq!(body["microsoft_disponible"], json!(false));
}

#[tokio::test]
async fn sso_config_con_credenciales_devuelve_true() {
    let (app, _pool) = app_con_config("g-id", "g-secret", "m-id", "m-secret").await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/auth/sso/config")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = cuerpo_json(res).await;
    assert_eq!(body["google_disponible"], json!(true));
    assert_eq!(body["microsoft_disponible"], json!(true));
}
