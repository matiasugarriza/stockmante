//! SSO con Google y Microsoft (OAuth 2.0 / OIDC) — 001 enmienda.

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::usuario::Usuario;
use serde::Deserialize;
use sqlx::PgPool;

/// Estado de disponibilidad de proveedores SSO para el frontend.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SsoConfig {
    pub google_disponible: bool,
    pub microsoft_disponible: bool,
}

pub fn sso_config(config: &AppConfig) -> SsoConfig {
    SsoConfig {
        google_disponible: config.google_disponible(),
        microsoft_disponible: config.microsoft_disponible(),
    }
}

/// Sonda de salud de los proveedores SSO (FR-071): credenciales presentes no
/// alcanzan — si el proveedor no responde, el frontend oculta su botón.
/// Resultado cacheado 60 s; timeout 1,5 s por sondeo. En modo `fija` (tests)
/// devuelve valores fijos sin tocar la red.
pub struct SondaSso {
    cache: std::sync::Mutex<std::collections::HashMap<String, (std::time::Instant, bool)>>,
    fijo: Option<std::collections::HashMap<String, bool>>,
}

impl SondaSso {
    /// Modo producción: sondea a los proveedores por HTTP.
    pub fn http() -> Self {
        Self {
            cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            fijo: None,
        }
    }

    /// Modo prueba: valores fijos por proveedor, sin red.
    pub fn fija(google: bool, microsoft: bool) -> Self {
        let mut mapa = std::collections::HashMap::new();
        mapa.insert("google".to_string(), google);
        mapa.insert("microsoft".to_string(), microsoft);
        Self {
            cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            fijo: Some(mapa),
        }
    }

    /// `true` si el proveedor responde. Cachea el resultado 60 segundos; un
    /// proveedor desconocido se considera caído.
    pub async fn salud(&self, proveedor: &str) -> bool {
        if let Some(fijo) = &self.fijo {
            return fijo.get(proveedor).copied().unwrap_or(false);
        }
        if let Ok(cache) = self.cache.lock() {
            if let Some((ts, ok)) = cache.get(proveedor) {
                if ts.elapsed() < std::time::Duration::from_secs(60) {
                    return *ok;
                }
            }
        }
        let ok = self.sonpear(proveedor).await;
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(proveedor.to_string(), (std::time::Instant::now(), ok));
        }
        ok
    }

    async fn sonpear(&self, proveedor: &str) -> bool {
        let url = match proveedor {
            "google" => "https://accounts.google.com/o/oauth2/v2/auth",
            "microsoft" => {
                "https://login.microsoftonline.com/common/v2.0/.well-known/openid-configuration"
            }
            _ => return false,
        };
        let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1500))
            .build()
        else {
            return false;
        };
        match client.get(url).send().await {
            // Cualquier respuesta HTTP < 500 = el servicio está levantado.
            Ok(resp) => resp.status().as_u16() < 500,
            Err(_) => false,
        }
    }
}

fn proveedor_valido(proveedor: &str) -> bool {
    proveedor == "google" || proveedor == "microsoft"
}

pub fn url_autorizacion(
    proveedor: &str,
    client_id: &str,
    state: &str,
    redirect_uri: &str,
) -> Result<String, AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }
    if client_id.is_empty() {
        return Err(AppError::SsoNoConfigurado);
    }
    let scope = match proveedor {
        "google" => "openid email profile",
        "microsoft" => "openid email profile",
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    let auth_url = match proveedor {
        "google" => "https://accounts.google.com/o/oauth2/v2/auth",
        "microsoft" => "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    Ok(format!(
        "{auth_url}?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope={scope}&state={state}",
        auth_url = auth_url,
        client_id = urlencoding(client_id),
        redirect_uri = urlencoding(redirect_uri),
        scope = urlencoding(scope),
        state = urlencoding(state),
    ))
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    email: String,
    email_verified: Option<bool>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct MicrosoftUserInfo {
    mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

pub async fn procesar_callback(
    pool: &PgPool,
    proveedor: &str,
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_base: &str,
    _correo: &dyn crate::mail::envio::EnvioCorreo,
    _base_url: &str,
) -> Result<Usuario, AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }

    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::SsoNoConfigurado);
    }

    let token_url = match proveedor {
        "google" => "https://oauth2.googleapis.com/token",
        "microsoft" => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    let userinfo_url = match proveedor {
        "google" => "https://www.googleapis.com/oauth2/v3/userinfo",
        "microsoft" => "https://graph.microsoft.com/v1.0/me",
        _ => return Err(AppError::ProveedorNoSoportado),
    };

    let redirect_uri = format!(
        "{}/api/auth/sso/{}/callback",
        redirect_base.trim_end_matches('/'),
        proveedor
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(token_url)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|_| AppError::ProveedorNoDisponible)?;

    if !resp.status().is_success() {
        return Err(AppError::ProveedorNoDisponible);
    }
    let token: TokenResponse = resp.json().await.map_err(|_| AppError::ProveedorNoDisponible)?;

    let uresp = client
        .get(userinfo_url)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|_| AppError::ProveedorNoDisponible)?;
    if !uresp.status().is_success() {
        return Err(AppError::ProveedorNoDisponible);
    }

    let (email, nombre, verificado) = if proveedor == "google" {
        let info: GoogleUserInfo = uresp.json().await.map_err(|_| AppError::ProveedorNoDisponible)?;
        let verificado = info.email_verified.unwrap_or(false);
        (info.email, info.name.unwrap_or_else(|| "Usuario".to_string()), verificado)
    } else {
        let info: MicrosoftUserInfo = uresp.json().await.map_err(|_| AppError::ProveedorNoDisponible)?;
        let email = info.mail.or(info.user_principal_name).unwrap_or_default();
        (email, info.display_name.unwrap_or_else(|| "Usuario".to_string()), true)
    };

    if email.is_empty() || !verificado {
        return Err(AppError::Validacion("El proveedor no devolvió un correo verificado.".into()));
    }

    let email = email.to_lowercase();

    let mut tx = pool.begin().await?;

    let usuario_id_existente: Option<i32> = sqlx::query_scalar(
        "SELECT usuario_id FROM identidades_externas WHERE proveedor = $1 AND subject = $2",
    )
    .bind(proveedor)
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await?;

    let usuario_id = if let Some(uid) = usuario_id_existente {
        uid
    } else {
        let uid_existente: Option<i32> =
            sqlx::query_scalar("SELECT id FROM usuarios WHERE email = $1")
                .bind(&email)
                .fetch_optional(&mut *tx)
                .await?;

        let uid = if let Some(uid) = uid_existente {
            uid
        } else {
            let new_uid: i32 = sqlx::query_scalar(
                r#"INSERT INTO usuarios (nombre, email, password_hash, password_temporal, rol, familia_rol, activo, email_verificado)
                   VALUES ($1, $2, '', FALSE, 'OPERARIO', 'OPERACION', TRUE, TRUE)
                   RETURNING id"#,
            )
            .bind(&nombre)
            .bind(&email)
            .fetch_one(&mut *tx)
            .await?;
            new_uid
        };

        sqlx::query(
            r#"INSERT INTO identidades_externas (usuario_id, proveedor, subject, email)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (proveedor, subject) DO NOTHING"#,
        )
        .bind(uid)
        .bind(proveedor)
        .bind(&email)
        .bind(&email)
        .execute(&mut *tx)
        .await?;

        uid
    };

    tx.commit().await?;

    let usuario = sqlx::query_as::<_, Usuario>(
        "SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en FROM usuarios WHERE id = $1",
    )
    .bind(usuario_id)
    .fetch_one(pool)
    .await?;

    Ok(usuario)
}

pub async fn listar_identidades(
    pool: &PgPool,
    usuario_id: i32,
) -> Result<Vec<(String, String, chrono::DateTime<chrono::Utc>)>, AppError> {
    let rows: Vec<(String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT proveedor, email, creado_en FROM identidades_externas WHERE usuario_id = $1 ORDER BY creado_en",
    )
    .bind(usuario_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn desvincular(
    pool: &PgPool,
    usuario_id: i32,
    proveedor: &str,
) -> Result<(), AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }
    let mut tx = pool.begin().await?;

    let tiene_password: bool = sqlx::query_scalar(
        "SELECT (password_hash <> '') FROM usuarios WHERE id = $1",
    )
    .bind(usuario_id)
    .fetch_one(&mut *tx)
    .await?;

    let otras: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM identidades_externas WHERE usuario_id = $1 AND proveedor <> $2",
    )
    .bind(usuario_id)
    .bind(proveedor)
    .fetch_one(&mut *tx)
    .await?;

    let esta_vinculada: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM identidades_externas WHERE usuario_id = $1 AND proveedor = $2)",
    )
    .bind(usuario_id)
    .bind(proveedor)
    .fetch_one(&mut *tx)
    .await?;

    if !esta_vinculada {
        return Err(AppError::Validacion("La identidad no está vinculada.".into()));
    }

    if !tiene_password && otras == 0 {
        return Err(AppError::UltimaIdentidad);
    }

    sqlx::query("DELETE FROM identidades_externas WHERE usuario_id = $1 AND proveedor = $2")
        .bind(usuario_id)
        .bind(proveedor)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn vincular(
    pool: &PgPool,
    usuario_id: i32,
    proveedor: &str,
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_base: &str,
    _correo: &dyn crate::mail::envio::EnvioCorreo,
    _base_url: &str,
) -> Result<(), AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::SsoNoConfigurado);
    }
    let token_url = match proveedor {
        "google" => "https://oauth2.googleapis.com/token",
        "microsoft" => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    let userinfo_url = match proveedor {
        "google" => "https://www.googleapis.com/oauth2/v3/userinfo",
        "microsoft" => "https://graph.microsoft.com/v1.0/me",
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    let redirect_uri = format!(
        "{}/api/auth/identidades/{}/vincular",
        redirect_base.trim_end_matches('/'),
        proveedor
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(token_url)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|_| AppError::ProveedorNoDisponible)?;
    if !resp.status().is_success() {
        return Err(AppError::ProveedorNoDisponible);
    }
    let token: TokenResponse = resp.json().await.map_err(|_| AppError::ProveedorNoDisponible)?;

    let uresp = client
        .get(userinfo_url)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|_| AppError::ProveedorNoDisponible)?;
    if !uresp.status().is_success() {
        return Err(AppError::ProveedorNoDisponible);
    }

    let (email, verificado) = if proveedor == "google" {
        let info: GoogleUserInfo = uresp.json().await.map_err(|_| AppError::ProveedorNoDisponible)?;
        (info.email, info.email_verified.unwrap_or(false))
    } else {
        let info: MicrosoftUserInfo = uresp.json().await.map_err(|_| AppError::ProveedorNoDisponible)?;
        let email = info.mail.or(info.user_principal_name).unwrap_or_default();
        (email, true)
    };
    if email.is_empty() || !verificado {
        return Err(AppError::Validacion("El proveedor no devolvió un correo verificado.".into()));
    }
    let email = email.to_lowercase();

    let owner: Option<i32> = sqlx::query_scalar(
        "SELECT usuario_id FROM identidades_externas WHERE proveedor = $1 AND subject = $2",
    )
    .bind(proveedor)
    .bind(&email)
    .fetch_optional(pool)
    .await?;
    if let Some(owner_id) = owner {
        if owner_id != usuario_id {
            return Err(AppError::IdentidadYaVinculada);
        }
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO identidades_externas (usuario_id, proveedor, subject, email) VALUES ($1, $2, $3, $4)",
    )
    .bind(usuario_id)
    .bind(proveedor)
    .bind(&email)
    .bind(&email)
    .execute(pool)
    .await
    .map_err(|_| AppError::IdentidadYaVinculada)?;

    Ok(())
}

pub fn proveedores() -> &'static [&'static str] {
    &["google", "microsoft"]
}