//! Registro por correo y verificación (001 enmienda).

use crate::auth::password::{hash_password, validar_politica_password};
use crate::error::AppError;
use crate::mail::envio::EnvioCorreo;
use crate::models::usuario::Usuario;
use crate::services::auditoria;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

fn hash_token(token: &str) -> String {
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    format!("{:x}", h.finalize())
}

fn generar_token() -> String {
    uuid::Uuid::new_v4().simple().to_string() + &uuid::Uuid::new_v4().simple().to_string()
}

pub async fn registrar(
    pool: &PgPool,
    correo: &dyn EnvioCorreo,
    nombre: &str,
    email: &str,
    password: &str,
    base_url: &str,
) -> Result<(), AppError> {
    // Validar política de contraseña.
    validar_politica_password(password)?;

    let email = email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::Validacion("El correo no es válido.".into()));
    }
    if nombre.trim().is_empty() {
        return Err(AppError::Validacion("El nombre es obligatorio.".into()));
    }

    // Anti-enumeración: si ya existe, responder OK sin revelar.
    let existe: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM usuarios WHERE email = $1)")
        .bind(&email)
        .fetch_one(pool)
        .await?;
    if existe {
        return Ok(());
    }

    let hash = hash_password(password)?;
    let mut tx = pool.begin().await?;

    let usuario_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO usuarios (nombre, email, password_hash, password_temporal, rol, familia_rol, activo, email_verificado)
           VALUES ($1, $2, $3, FALSE, 'OPERARIO', 'OPERACION', TRUE, FALSE)
           RETURNING id"#,
    )
    .bind(nombre.trim())
    .bind(&email)
    .bind(&hash)
    .fetch_one(&mut *tx)
    .await?;

    let token = generar_token();
    let token_hash = hash_token(&token);
    let expira = Utc::now() + Duration::hours(24);

    sqlx::query(
        r#"INSERT INTO tokens_verificacion (usuario_id, token_hash, expira_en)
           VALUES ($1, $2, $3)"#,
    )
    .bind(usuario_id)
    .bind(&token_hash)
    .bind(expira)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Enviar correo (fail-closed en producción).
    if let Err(e) =     correo.enviar_verificacion(&email, nombre.trim(), &token, base_url)
        .await
    {
        tracing::error!(error = %e, "fallo envío de verificación");
        // No revertimos: la cuenta queda pendiente; el admin puede reenviar.
    }

    Ok(())
}

pub async fn verificar_correo(pool: &PgPool, token: &str) -> Result<(), AppError> {
    let token_hash = hash_token(token);
    let mut tx = pool.begin().await?;

    let row: Option<(i32, bool, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT usuario_id, usado, expira_en FROM tokens_verificacion WHERE token_hash = $1 FOR UPDATE",
    )
    .bind(&token_hash)
    .fetch_optional(&mut *tx)
    .await?;

    let (usuario_id, usado, expira) = row.ok_or(AppError::TokenVerificacionInvalido)?;
    if usado {
        return Err(AppError::TokenVerificacionInvalido);
    }
    if Utc::now() > expira {
        return Err(AppError::TokenVerificacionInvalido);
    }

    sqlx::query("UPDATE tokens_verificacion SET usado = TRUE WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE usuarios SET email_verificado = TRUE, activo = TRUE WHERE id = $1")
        .bind(usuario_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}