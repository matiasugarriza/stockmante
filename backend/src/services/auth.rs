//! Servicio de autenticación (Habilidad M1-A + research R1):
//! una sesión vigente por usuario, expiración controlada por `Reloj`,
//! fallos de login indistinguibles (anti-enumeración).

use crate::auth::password::{hash_password, validar_politica_password, verify_password};
use crate::auth::reloj::Reloj;
use crate::error::AppError;
use crate::models::usuario::{SesionVigente, Usuario, UsuarioConHash};
use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

/// Normaliza email para lookup: minúsculas + sin espacios sobrantes.
fn normalizar_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Login: valida credenciales y crea una sesión nueva revocando la anterior.
/// Devuelve el id de sesión (cookie) y el usuario público.
pub async fn login(
    pool: &PgPool,
    reloj: &dyn Reloj,
    email: &str,
    password: &str,
    expiry_hours: i64,
) -> Result<(Uuid, Usuario), AppError> {
    let email_norm = normalizar_email(email);
    let fila = sqlx::query_as::<_, UsuarioConHash>(
        r#"SELECT id, nombre, email, password_hash, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios WHERE email = $1"#,
    )
    .bind(&email_norm)
    .fetch_optional(pool)
    .await?;

    // Verificamos SIEMPRE argon2 aunque el usuario no exista (anti timing-oracle).
    let hash_ok = match &fila {
        Some(u) => verify_password(password, &u.password_hash),
        None => {
            // hash dummy: consume tiempo comparable al path real
            let _ = verify_password(
                password,
                "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            );
            false
        }
    };

    // Mismo error (y mensaje) para: email inexistente, password mal o cuenta inactiva.
    let Some(u) = fila.filter(|u| hash_ok && u.activo) else {
        return Err(AppError::CredencialesInvalidas);
    };

    let ahora = reloj.ahora();
    let expira_en = ahora + Duration::hours(expiry_hours);
    let sesion_id = Uuid::new_v4();

    // Sesión única: revocar la vigente anterior e insertar la nueva atómicamente.
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"UPDATE sesiones SET revocada_en = $1
           WHERE usuario_id = $2 AND revocada_en IS NULL"#,
    )
    .bind(ahora)
    .bind(u.id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(r#"INSERT INTO sesiones (id, usuario_id, expira_en) VALUES ($1, $2, $3)"#)
        .bind(sesion_id)
        .bind(u.id)
        .bind(expira_en)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok((sesion_id, u.into_publica()))
}

/// Cierra la sesión (idempotente: no falla si ya estaba revocada o no existe).
pub async fn logout(pool: &PgPool, reloj: &dyn Reloj, sesion_id: Uuid) -> Result<(), AppError> {
    sqlx::query(r#"UPDATE sesiones SET revocada_en = $1 WHERE id = $2 AND revocada_en IS NULL"#)
        .bind(reloj.ahora())
        .bind(sesion_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Resuelve la sesión vigente → usuario. Errores:
/// - no existe / revocada / usuario inactivo → `NoAutenticado` (401)
/// - expirada → `SesionExpirada` (401)
pub async fn sesion_actual(
    pool: &PgPool,
    reloj: &dyn Reloj,
    sesion_id: Uuid,
) -> Result<Usuario, AppError> {
    let fila = sqlx::query_as::<_, SesionVigente>(
        r#"SELECT u.id, u.nombre, u.email, u.rol, u.familia_rol, u.activo, u.password_temporal,
                  u.creado_en, s.expira_en
           FROM sesiones s
           JOIN usuarios u ON u.id = s.usuario_id
           WHERE s.id = $1 AND s.revocada_en IS NULL"#,
    )
    .bind(sesion_id)
    .fetch_optional(pool)
    .await?;

    let Some(s) = fila else {
        return Err(AppError::NoAutenticado);
    };
    if !s.activo {
        return Err(AppError::NoAutenticado);
    }
    if s.expira_en <= reloj.ahora() {
        return Err(AppError::SesionExpirada);
    }
    Ok(s.into_usuario())
}

/// Cambio de contraseña con sesión autenticada.
/// Valida política, verifica la actual (puede ser la temporal) y limpia `password_temporal`.
pub async fn cambiar_password(
    pool: &PgPool,
    reloj: &dyn Reloj,
    sesion_id: Uuid,
    password_actual: &str,
    password_nueva: &str,
) -> Result<(), AppError> {
    validar_politica_password(password_nueva)?;

    let fila = sqlx::query_as::<_, (i32, String, bool)>(
        r#"SELECT u.id, u.password_hash, u.activo
           FROM sesiones s
           JOIN usuarios u ON u.id = s.usuario_id
           WHERE s.id = $1 AND s.revocada_en IS NULL"#,
    )
    .bind(sesion_id)
    .fetch_optional(pool)
    .await?;

    let Some((usuario_id, hash_viejo, activo)) = fila else {
        return Err(AppError::NoAutenticado);
    };
    if !activo {
        return Err(AppError::NoAutenticado);
    }
    if !verify_password(password_actual, &hash_viejo) {
        return Err(AppError::CredencialesInvalidas);
    }

    let hash_nuevo = hash_password(password_nueva)?;
    sqlx::query(
        r#"UPDATE usuarios
           SET password_hash = $1, password_temporal = FALSE, actualizado_en = $2
           WHERE id = $3"#,
    )
    .bind(&hash_nuevo)
    .bind(reloj.ahora())
    .bind(usuario_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Helper para otros servicios: usuario por id (o 404).
pub async fn usuario_por_id(pool: &PgPool, id: i32) -> Result<Usuario, AppError> {
    sqlx::query_as::<_, Usuario>(
        r#"SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("usuario no encontrado".into()))
}
