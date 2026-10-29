//! SSO con Google y Microsoft (OAuth 2.0 / OIDC) — 001 enmienda.

use crate::error::AppError;
use crate::models::usuario::Usuario;
use crate::services::auditoria;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::env;

fn proveedor_valido(proveedor: &str) -> bool {
    proveedor == "google" || proveedor == "microsoft"
}

pub fn url_autorizacion(proveedor: &str, state: &str, redirect_uri: &str) -> Result<String, AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }
    let (auth_url, client_id, scope) = match proveedor {
        "google" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            "openid email profile",
        ),
        "microsoft" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            "openid email profile",
        ),
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    if client_id.is_empty() {
        return Err(AppError::ProveedorNoDisponible);
    }
    Ok(format!(
        "{auth_url}?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope={scope}&state={state}",
        auth_url = auth_url,
        client_id = urlencoding(&client_id),
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
    _correo: &dyn crate::mail::envio::EnvioCorreo,
    _base_url: &str,
) -> Result<Usuario, AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }

    let (client_id, client_secret, token_url, userinfo_url) = match proveedor {
        "google" => (
            env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/oauth2/v3/userinfo",
        ),
        "microsoft" => (
            env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            "https://graph.microsoft.com/v1.0/me",
        ),
        _ => return Err(AppError::ProveedorNoSoportado),
    };

    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::ProveedorNoDisponible);
    }

    let redirect_uri = env::var("SSO_REDIRECT_BASE")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        + &format!("/api/auth/sso/{}/callback", proveedor);

    let client = reqwest::Client::new();
    let resp = client
        .post(token_url)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
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
    _correo: &dyn crate::mail::envio::EnvioCorreo,
    _base_url: &str,
) -> Result<(), AppError> {
    if !proveedor_valido(proveedor) {
        return Err(AppError::ProveedorNoSoportado);
    }
    let (client_id, client_secret, token_url, userinfo_url) = match proveedor {
        "google" => (
            env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/oauth2/v3/userinfo",
        ),
        "microsoft" => (
            env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            "https://graph.microsoft.com/v1.0/me",
        ),
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::ProveedorNoDisponible);
    }
    let redirect_uri = env::var("SSO_REDIRECT_BASE")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        + &format!("/api/auth/identidades/{}/vincular", proveedor);

    let client = reqwest::Client::new();
    let resp = client
        .post(token_url)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
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