use stockmante_backend::{
    app::{construir_router, AppEstado},
    auth::reloj::RelojSistema,
    config::AppConfig,
    db,
    error::AppError,
    mail::envio::CorreoDev,
};
use std::sync::Arc;

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

    // En dev el stub imprime la temporal en consola; en producción es silencioso.
    let correo = if config.es_produccion() {
        CorreoDev::new()
    } else {
        CorreoDev::para_desarrollo()
    };

    let estado = AppEstado {
        pool,
        config: config.clone(),
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(correo),
    };

    let app = construir_router(estado);

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
