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
    /// Swagger UI + OpenAPI (000). Default: encendido fuera de producción.
    pub docs_enabled: bool,
    /// Credenciales SSO (Google/Microsoft). Vacías = proveedor no disponible.
    pub google_client_id: String,
    pub google_client_secret: String,
    pub microsoft_client_id: String,
    pub microsoft_client_secret: String,
    pub sso_redirect_base: String,
    /// Minutos entre ciclos de alertas de stock (011); `0` desactiva el scheduler.
    pub alertas_intervalo_minutos: u64,
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

    /// Interruptor textual tolerante (`true`/`1`, `false`/`0`, sin distinguir
    /// mayúsculas). Cualquier otro valor es error de configuración: no se
    /// adivina el default cuando el operador escribió algo distinto.
    pub fn parsear_interruptor(nombre: &str, valor: &str) -> Result<bool, AppError> {
        match valor.trim().to_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(AppError::Validacion(format!(
                "{nombre} debe ser true o false"
            ))),
        }
    }

    /// `ALERTAS_INTERVALO_MINUTOS`: ausente → 60; `0` desactiva; no numérico → error
    /// de configuración (no se arranca con un valor que el operador no quiso).
    pub fn parsear_intervalo_alertas(valor: Option<&str>) -> Result<u64, AppError> {
        match valor.map(str::trim) {
            None | Some("") => Ok(60),
            Some(texto) => texto.parse::<u64>().map_err(|_| {
                AppError::Validacion(
                    "ALERTAS_INTERVALO_MINUTOS debe ser un entero mayor o igual a 0".into(),
                )
            }),
        }
    }

    /// Default de `DOCS_ENABLED`: encendido en desarrollo, apagado en
    /// producción. Un valor explícito siempre gana.
    pub fn docs_por_defecto(app_env: &str) -> bool {
        !Self::es_entorno_produccion(app_env)
    }

    /// Indica si Google OAuth está configurado (client_id y secret presentes).
    pub fn google_disponible(&self) -> bool {
        !self.google_client_id.trim().is_empty() && !self.google_client_secret.trim().is_empty()
    }

    /// Indica si Microsoft OAuth está configurado (client_id y secret presentes).
    pub fn microsoft_disponible(&self) -> bool {
        !self.microsoft_client_id.trim().is_empty() && !self.microsoft_client_secret.trim().is_empty()
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
        let docs_enabled = match std::env::var("DOCS_ENABLED") {
            Ok(valor) => Self::parsear_interruptor("DOCS_ENABLED", &valor)?,
            Err(_) => Self::docs_por_defecto(&app_env),
        };
        Ok(Self {
            database_url,
            app_url,
            port,
            session_cookie_name,
            session_expiry_hours,
            email_from,
            smtp_url,
            app_env,
            docs_enabled,
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            microsoft_client_id: std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            microsoft_client_secret: std::env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
            sso_redirect_base: std::env::var("SSO_REDIRECT_BASE")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
            alertas_intervalo_minutos: Self::parsear_intervalo_alertas(
                std::env::var("ALERTAS_INTERVALO_MINUTOS").ok().as_deref(),
            )?,
        })
    }
}
