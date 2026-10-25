//! Estado de la aplicación y armado del router (compartido por `main` y tests).

use crate::auth::reloj::Reloj;
use crate::config::AppConfig;
use crate::handlers;
use crate::mail::envio::EnvioCorreo;
use axum::{http::{header, Method}, routing::get, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Estado compartido de la aplicación (Principio II: centralizado en un solo struct).
#[derive(Clone)]
pub struct AppEstado {
    pub pool: PgPool,
    pub config: AppConfig,
    pub reloj: Arc<dyn Reloj>,
    pub correo: Arc<dyn EnvioCorreo>,
}

/// Router completo bajo `/api` con CORS y rutas del Módulo 1.
pub fn construir_router(estado: AppEstado) -> Router {
    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);
    match estado.config.app_url.parse::<header::HeaderValue>() {
        Ok(origin) => cors = cors.allow_origin(origin),
        Err(_) => tracing::warn!(
            app_url = %estado.config.app_url,
            "APP_URL no es un origin válido; CORS sin allow_origin"
        ),
    }

    Router::new()
        .route("/api/salud", get(handlers::salud))
        .nest("/api/auth", handlers::auth::router())
        .nest("/api/usuarios", handlers::usuarios::router())
        .layer(cors)
        .with_state(estado)
}
