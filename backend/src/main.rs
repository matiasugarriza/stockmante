use stockmante_backend::{
    app::{construir_router, AppEstado},
    auth::reloj::RelojSistema,
    config::AppConfig,
    db,
    error::AppError,
    mail::{envio::{CorreoDev, EnvioCorreo}, smtp::CorreoSmtp},
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
    config.validar()?;
    let pool = db::crear_pool(&config.database_url).await?;

    // Con SMTP_URL definida se envía por SMTP real. En desarrollo sin SMTP el
    // stub imprime la temporal en consola; en producción `validar()` ya impide
    // arrancar sin SMTP_URL (fail-closed, T029).
    let correo: Arc<dyn EnvioCorreo> = match config.smtp_url.as_deref() {
        Some(url) => {
            let transporte = CorreoSmtp::desde_url(url, &config.email_from).map_err(|e| {
                tracing::error!(error = %e, "configuración SMTP inválida");
                AppError::Interno("configuración SMTP inválida".into())
            })?;
            Arc::new(transporte)
        }
        None if config.es_produccion() => {
            return Err(AppError::Interno("SMTP_URL no definida en producción".into()));
        }
        None => Arc::new(CorreoDev::para_desarrollo()),
    };

    let estado = AppEstado {
        pool,
        config: config.clone(),
        reloj: Arc::new(RelojSistema),
        correo,
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
