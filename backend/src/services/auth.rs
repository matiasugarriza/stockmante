//! Servicio de autenticación (Habilidad M1-A + research R1):
//! una sesión vigente por usuario, expiración controlada por `Reloj`,
//! fallos de login indistinguibles (anti-enumeración).

use crate::auth::password::{hash_password, validar_politica_password, verify_password};
use crate::auth::reloj::Reloj;
use crate::error::AppError;
use crate::models::usuario::{normalizar_email, SesionVigente, Usuario, UsuarioConHash};
use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

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
    let fila = sqlx::query_as!(
        UsuarioConHash,
        r#"SELECT id, nombre, email, password_hash, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios WHERE email = $1"#,
        email_norm
    )
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

    // El lock del usuario serializa logins concurrentes del mismo usuario: el
    // segundo espera, revoca la sesión del primero e inserta la suya. El índice
    // único parcial (`idx_sesiones_usuario_vigente`, migración 0005) respalda la
    // invariante aunque otro camino intente insertar sin pasar por acá.
    let usuario_bloqueado =
        sqlx::query_scalar!(r#"SELECT id FROM usuarios WHERE id = $1 FOR UPDATE"#, u.id)
            .fetch_optional(&mut *tx)
            .await?;
    if usuario_bloqueado.is_none() {
        // el usuario se borró entre la lectura y el lock
        return Err(AppError::NoAutenticado);
    }

    sqlx::query!(
        r#"UPDATE sesiones SET revocada_en = $1
           WHERE usuario_id = $2 AND revocada_en IS NULL"#,
        ahora,
        u.id
    )
    .execute(&mut *tx)
    .await?;
    let insert = sqlx::query!(
        r#"INSERT INTO sesiones (id, usuario_id, expira_en) VALUES ($1, $2, $3)"#,
        sesion_id,
        u.id,
        expira_en
    )
    .execute(&mut *tx)
    .await;
    if let Err(e) = insert {
        if matches!(&e, sqlx::Error::Database(db) if db.is_unique_violation()) {
            // Invariante de sesión única violada: la BD manda y no se filtran
            // detalles internos al cliente (la transacción se revierte al caer).
            return Err(AppError::Interno("violación de sesión única".into()));
        }
        return Err(e.into());
    }
    tx.commit().await?;

    Ok((sesion_id, u.into_publica()))
}

/// Crea una sesión nueva para un usuario ya autenticado (SSO/registro).
/// Revoca la sesión anterior del usuario.
pub async fn crear_sesion_para_usuario(
    pool: &PgPool,
    reloj: &dyn Reloj,
    usuario_id: i32,
    expiry_hours: i64,
) -> Result<Uuid, AppError> {
    let ahora = reloj.ahora();
    let expira_en = ahora + Duration::hours(expiry_hours);
    let sesion_id = Uuid::new_v4();

    let mut tx = pool.begin().await?;

    let usuario_bloqueado =
        sqlx::query_scalar!(r#"SELECT id FROM usuarios WHERE id = $1 FOR UPDATE"#, usuario_id)
            .fetch_optional(&mut *tx)
            .await?;
    if usuario_bloqueado.is_none() {
        return Err(AppError::NoAutenticado);
    }

    sqlx::query!(
        r#"UPDATE sesiones SET revocada_en = $1
           WHERE usuario_id = $2 AND revocada_en IS NULL"#,
        ahora,
        usuario_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"INSERT INTO sesiones (id, usuario_id, expira_en) VALUES ($1, $2, $3)"#,
        sesion_id,
        usuario_id,
        expira_en
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(sesion_id)
}
pub async fn logout(pool: &PgPool, reloj: &dyn Reloj, sesion_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"UPDATE sesiones SET revocada_en = $1 WHERE id = $2 AND revocada_en IS NULL"#,
        reloj.ahora(),
        sesion_id
    )
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
    let fila = sqlx::query_as!(
        SesionVigente,
        r#"SELECT u.id, u.nombre, u.email, u.rol, u.familia_rol, u.activo, u.password_temporal,
                  u.creado_en, s.expira_en
           FROM sesiones s
           JOIN usuarios u ON u.id = s.usuario_id
           WHERE s.id = $1 AND s.revocada_en IS NULL"#,
        sesion_id
    )
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

    let fila = sqlx::query!(
        r#"SELECT u.id, u.password_hash, u.activo
           FROM sesiones s
           JOIN usuarios u ON u.id = s.usuario_id
           WHERE s.id = $1 AND s.revocada_en IS NULL"#,
        sesion_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(fila) = fila else {
        return Err(AppError::NoAutenticado);
    };
    if !fila.activo {
        return Err(AppError::NoAutenticado);
    }
    if !verify_password(password_actual, &fila.password_hash) {
        return Err(AppError::CredencialesInvalidas);
    }

    let hash_nuevo = hash_password(password_nueva)?;
    sqlx::query!(
        r#"UPDATE usuarios
           SET password_hash = $1, password_temporal = FALSE, actualizado_en = $2
           WHERE id = $3"#,
        hash_nuevo,
        reloj.ahora(),
        fila.id
    )
    .execute(pool)
    .await?;
    Ok(())
}
