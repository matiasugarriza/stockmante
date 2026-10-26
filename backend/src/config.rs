//! Carga de configuración desde variables de entorno (.env en dev).

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub app_url: String,
    pub port: u16,
    pub session_cookie_name: String,
    pub session_expiry_hours: i64,
    pub email_from: String,
    pub smtp_url: Option<String>,
    pub app_env: String,
}

impl AppConfig {
    /// `APP_ENV=production` (sin distinguir mayúsculas ni espacios) desactiva
    /// las ayudas de desarrollo; cualquier otro valor se trata como dev.
    pub fn es_produccion(&self) -> bool {
        Self::es_entorno_produccion(&self.app_env)
    }

    pub fn es_entorno_produccion(app_env: &str) -> bool {
        app_env.trim().eq_ignore_ascii_case("production")
    }

    /// Fail-closed de producción (T029): sin `SMTP_URL` no hay forma de entregar
    /// la contraseña temporal, así que el arranque falla en vez de crear cuentas
    /// sin credenciales. En desarrollo el stub de consola alcanza.
    pub fn validar(&self) -> Result<(), AppError> {
        if !self.es_produccion() {
            return Ok(());
        }
        let smtp_configurado = self
            .smtp_url
            .as_deref()
            .map(str::trim)
            .is_some_and(|url| !url.is_empty());
        if !smtp_configurado {
            tracing::error!("APP_ENV=production requiere SMTP_URL para enviar la contraseña temporal");
            return Err(AppError::Interno("SMTP_URL no definida en producción".into()));
        }
        Ok(())
    }

    pub fn desde_entorno() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            AppError::Interno("DATABASE_URL no definida".into())
        })?;
        let app_url =
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".into());
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        let session_cookie_name = std::env::var("SESSION_COOKIE_NAME")
            .unwrap_or_else(|_| "stockmante_sesion".into());
        let session_expiry_hours: i64 = std::env::var("SESSION_EXPIRY_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(8);
        let email_from =
            std::env::var("EMAIL_FROM").unwrap_or_else(|_| "soporte@stockmante.local".into());
        let smtp_url = std::env::var("SMTP_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        Ok(Self {
            database_url,
            app_url,
            port,
            session_cookie_name,
            session_expiry_hours,
            email_from,
            smtp_url,
            app_env,
        })
    }
}
