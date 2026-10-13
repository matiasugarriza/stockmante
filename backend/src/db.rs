//! Pool SQLx compartido.

use crate::error::AppError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn crear_pool(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "no se pudo conectar a la base");
            AppError::Interno("sin conexión a la base de datos".into())
        })
}
