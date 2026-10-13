use axum::{
    http::{header, Method},
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use stockmante_backend::{
    auth::reloj::{Reloj, RelojSistema},
    config::AppConfig,
    db,
    error::AppError,
    mail::envio::{CorreoDev, EnvioCorreo},
};
use tower_http::cors::CorsLayer;

/// Estado compartido de la aplicación (Principio II: centralizado en un solo struct).
#[derive(Clone)]
pub struct AppEstado {
    pub pool: PgPool,
    pub config: AppConfig,
    pub reloj: Arc<dyn Reloj>,
    pub correo: Arc<dyn EnvioCorreo>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    if let Err(e) = arrancar().await {
        // bootstrap: sin unwrap (calidad); exit code ≠ 0
        eprintln!("No se pudo iniciar el backend: {e}");
        std::process::exit(1);
    }
}

async fn arrancar() -> Result<(), AppError> {
    let config = AppConfig::desde_entorno()?;
    let pool = db::crear_pool(&config.database_url).await?;

    let estado = AppEstado {
        pool,
        config: config.clone(),
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(CorreoDev::new()),
    };

    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);
    match config.app_url.parse::<header::HeaderValue>() {
        Ok(origin) => cors = cors.allow_origin(origin),
        Err(_) => tracing::warn!(app_url = %config.app_url, "APP_URL no es un origin válido; CORS sin allow_origin"),
    }

    let app = Router::new()
        .route("/api/salud", get(salud))
        .layer(cors)
        .with_state(estado);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        tracing::error!(error = %e, addr = %addr, "no se pudo enlazar");
        AppError::Interno("no se pudo escuchar el puerto".into())
    })?;
    tracing::info!("backend escuchando en {}", config.port);
    axum::serve(listener, app).await.map_err(|e| {
        tracing::error!(error = %e, "servidor interrumpido");
        AppError::Interno("servidor interrumpido".into())
    })?;
    Ok(())
}

async fn salud() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}
