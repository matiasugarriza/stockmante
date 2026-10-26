//! Servicio de usuarios (US2): CRUD con baja lógica, contraseña temporal por
//! correo y guarda del último usuario de gestión activo.
//!
//! Decisiones:
//! - No existe borrado físico de usuarios (`DELETE`), solo `activo = FALSE`.
//! - La unicidad de email se resuelve dejando que la BD la imponga (sin
//!   pre-chequeo TOCTOU) y mapeando la violación `23505`.
//! - El correo con la temporal se envía dentro de la transacción: si el envío
//!   falla, no queda una cuenta creada sin credenciales entregadas.
//! - Regenerar la temporal revoca las sesiones vigentes del usuario.

use crate::auth::password::{generar_password_temporal, hash_password};
use crate::auth::reloj::Reloj;
use crate::error::AppError;
use crate::mail::envio::EnvioCorreo;
use crate::models::usuario::{normalizar_email, Usuario};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use super::auditoria;
use utoipa::ToSchema;

const POR_PAGINA_DEFAULT: i64 = 20;
const POR_PAGINA_MAX: i64 = 100;

/// Roles del catálogo (data-model.md §usuarios + migrations/0001).
pub const ROLES: [&str; 4] = ["ADMIN", "GERENTE", "OPERARIO", "CAJERO"];

/// Mapa rol → familia. Es la única fuente de verdad de la coherencia que la BD
/// también exige con `chk_familia_rol_coherente`.
pub fn familia_de_rol(rol: &str) -> Result<&'static str, AppError> {
    match rol {
        "ADMIN" | "GERENTE" => Ok("GESTION"),
        "OPERARIO" | "CAJERO" => Ok("OPERACION"),
        _ => Err(AppError::Validacion(
            "El rol debe ser ADMIN, GERENTE, OPERARIO o CAJERO".into(),
        )),
    }
}

/// Cambios parciales de `PUT /usuarios/{id}` (`None` = no tocar).
#[derive(Debug, Default)]
pub struct CambiosUsuario {
    pub nombre: Option<String>,
    pub email: Option<String>,
    pub rol: Option<String>,
}

/// Página devuelta por `GET /usuarios`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ListaUsuarios {
    pub items: Vec<Usuario>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

fn validar_nombre(nombre: &str) -> Result<String, AppError> {
    let nombre = nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validacion("Ingresá el nombre del usuario".into()));
    }
    if nombre.chars().count() > 120 {
        return Err(AppError::Validacion(
            "El nombre no puede superar los 120 caracteres".into(),
        ));
    }
    Ok(nombre.to_string())
}

/// RFC básico: exactamente una `@`, partes no vacías y sin puntos consecutivos
/// ni en los bordes. Es deliberadamente conservador: lo ambiguo se rechaza (FR-008).
fn validar_email(email: &str) -> Result<String, AppError> {
    let email = normalizar_email(email);
    let parte_valida = |parte: &str| {
        !parte.is_empty()
            && !parte.starts_with('.')
            && !parte.ends_with('.')
            && !parte.contains("..")
    };
    let valido = email.len() <= 255
        && !email.chars().any(char::is_whitespace)
        && match email.split_once('@') {
            Some((usuario, dominio)) => {
                parte_valida(usuario)
                    && !dominio.contains('@')
                    && dominio.contains('.')
                    && parte_valida(dominio)
            }
            None => false,
        };
    if valido {
        Ok(email)
    } else {
        Err(AppError::Validacion("Ingresá un email válido".into()))
    }
}

fn normalizar_rol(rol: &str) -> Result<(String, &'static str), AppError> {
    let rol = rol.trim().to_uppercase();
    let familia = familia_de_rol(&rol)?;
    Ok((rol, familia))
}

fn normalizar_paginacion(
    pagina: Option<i64>,
    por_pagina: Option<i64>,
) -> Result<(i64, i64, i64), AppError> {
    let pagina = pagina.unwrap_or(1);
    let por_pagina = por_pagina.unwrap_or(POR_PAGINA_DEFAULT);
    if pagina < 1 {
        return Err(AppError::Validacion(
            "La página debe ser mayor o igual a 1".into(),
        ));
    }
    if !(1..=POR_PAGINA_MAX).contains(&por_pagina) {
        return Err(AppError::Validacion(format!(
            "por_pagina debe estar entre 1 y {POR_PAGINA_MAX}"
        )));
    }
    let offset = pagina
        .checked_sub(1)
        .and_then(|paginas_previas| paginas_previas.checked_mul(por_pagina))
        .ok_or_else(|| AppError::Validacion("La página es demasiado grande".into()))?;
    Ok((pagina, por_pagina, offset))
}

/// `23505` sobre `usuarios_email_key` → email duplicado.
fn es_email_duplicado(error: &sqlx::Error) -> bool {
    error.as_database_error().is_some_and(|db| {
        db.code().as_deref() == Some("23505")
            && db.constraint().is_some_and(|c| c == "usuarios_email_key")
    })
}

fn error_envio(error: String) -> AppError {
    tracing::error!(error = %error, "no se pudo enviar la contraseña temporal");
    AppError::Interno("no se pudo enviar la contraseña temporal".into())
}

/// Lista paginada, opcionalmente filtrada por estado (`None` = todas).
pub async fn listar(
    pool: &PgPool,
    activo: Option<bool>,
    pagina: Option<i64>,
    por_pagina: Option<i64>,
) -> Result<ListaUsuarios, AppError> {
    let (pagina, por_pagina, offset) = normalizar_paginacion(pagina, por_pagina)?;

    let items = sqlx::query_as!(
        Usuario,
        r#"SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios
           WHERE $1::boolean IS NULL OR activo = $1
           ORDER BY id
           LIMIT $2 OFFSET $3"#,
        activo,
        por_pagina,
        offset
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "total!" FROM usuarios
           WHERE $1::boolean IS NULL OR activo = $1"#,
        activo
    )
    .fetch_one(pool)
    .await?;

    Ok(ListaUsuarios {
        items,
        total,
        pagina,
        por_pagina,
    })
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<Usuario, AppError> {
    sqlx::query_as!(
        Usuario,
        r#"SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios
           WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("usuario no encontrado".into()))
}

/// Alta: genera la temporal, la envía por correo y deja el usuario activo.
pub async fn crear(
    pool: &PgPool,
    correo: &dyn EnvioCorreo,
    actor_id: i32,
    nombre: &str,
    email: &str,
    rol: &str,
) -> Result<Usuario, AppError> {
    let nombre = validar_nombre(nombre)?;
    let email = validar_email(email)?;
    let (rol, familia) = normalizar_rol(rol)?;
    let temporal = generar_password_temporal();
    let hash = hash_password(&temporal)?;

    let mut tx = pool.begin().await?;
    let creado = match sqlx::query_as!(
        Usuario,
        r#"INSERT INTO usuarios (nombre, email, password_hash, password_temporal, rol, familia_rol, activo)
           VALUES ($1, $2, $3, TRUE, $4, $5, TRUE)
           RETURNING id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en"#,
        nombre,
        email,
        hash,
        rol,
        familia
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(usuario) => usuario,
        Err(e) if es_email_duplicado(&e) => return Err(AppError::EmailDuplicado),
        Err(e) => return Err(e.into()),
    };

    correo
        .enviar_password_temporal(&creado.email, &creado.nombre, &temporal)
        .await
        .map_err(error_envio)?;

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_USUARIOS,
        creado.id,
        auditoria::ACCION_CREAR,
        actor_id,
    )
    .await?;

    tx.commit().await?;
    Ok(creado)
}

/// Actualización parcial. Cambiar el rol recalcula `familia_rol`.
pub async fn actualizar(
    pool: &PgPool,
    id: i32,
    cambios: CambiosUsuario,
    actor_id: i32,
) -> Result<Usuario, AppError> {
    let mut tx = pool.begin().await?;

    // Orden de locks: primero el conjunto de gestores, después la fila objetivo.
    // Así dos bajas/degradaciones concurrentes no se bloquean en cruz (40P01).
    let gestores = bloquear_gestores_activos(&mut tx).await?;

    let actual = sqlx::query_as!(
        Usuario,
        r#"SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios
           WHERE id = $1 FOR UPDATE"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("usuario no encontrado".into()))?;

    let nombre = match cambios.nombre {
        Some(nombre) => validar_nombre(&nombre)?,
        None => actual.nombre.clone(),
    };
    let email = match cambios.email {
        Some(email) => validar_email(&email)?,
        None => actual.email.clone(),
    };
    let (rol, familia) = match cambios.rol {
        Some(rol) => {
            let (rol, familia) = normalizar_rol(&rol)?;
            (rol, familia.to_string())
        }
        None => (actual.rol.clone(), actual.familia_rol.clone()),
    };

    // Degradar al último gestor activo dejaría el sistema sin administración.
    if actual.familia_rol == "GESTION" && familia != "GESTION" {
        asegurar_queda_gestor(&gestores, &actual)?;
    }

    let actualizado = match sqlx::query_as!(
        Usuario,
        r#"UPDATE usuarios
           SET nombre = $1, email = $2, rol = $3, familia_rol = $4, actualizado_en = now()
           WHERE id = $5
           RETURNING id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en"#,
        nombre,
        email,
        rol,
        familia,
        id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(usuario) => usuario,
        Err(e) if es_email_duplicado(&e) => return Err(AppError::EmailDuplicado),
        Err(e) => return Err(e.into()),
    };

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_USUARIOS,
        actualizado.id,
        auditoria::ACCION_ACTUALIZAR,
        actor_id,
    )
    .await?;

    tx.commit().await?;
    Ok(actualizado)
}

/// Baja lógica + revocación de sesiones. Idempotente si ya estaba inactivo.
pub async fn desactivar(
    pool: &PgPool,
    reloj: &dyn Reloj,
    id: i32,
    actor_id: i32,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Mismo orden que `actualizar`: conjunto de gestores antes que la fila objetivo.
    let gestores = bloquear_gestores_activos(&mut tx).await?;

    let usuario = sqlx::query_as!(
        Usuario,
        r#"SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios
           WHERE id = $1 FOR UPDATE"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("usuario no encontrado".into()))?;

    if usuario.activo {
        asegurar_queda_gestor(&gestores, &usuario)?;
    }

    let ahora = reloj.ahora();
    sqlx::query!(
        r#"UPDATE usuarios SET activo = FALSE, actualizado_en = $1 WHERE id = $2"#,
        ahora,
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"UPDATE sesiones SET revocada_en = $1 WHERE usuario_id = $2 AND revocada_en IS NULL"#,
        ahora,
        id
    )
    .execute(&mut *tx)
    .await?;

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_USUARIOS,
        id,
        auditoria::ACCION_DESACTIVAR,
        actor_id,
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Reactiva la cuenta. No toca la contraseña: si la olvidó, se regenera aparte.
pub async fn reactivar(
    pool: &PgPool,
    reloj: &dyn Reloj,
    id: i32,
    actor_id: i32,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let filas = sqlx::query!(
        r#"UPDATE usuarios SET activo = TRUE, actualizado_en = $1 WHERE id = $2"#,
        reloj.ahora(),
        id
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if filas == 0 {
        return Err(AppError::RecursoNoEncontrado(
            "usuario no encontrado".into(),
        ));
    }
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_USUARIOS,
        id,
        auditoria::ACCION_REACTIVAR,
        actor_id,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Nueva contraseña temporal: actualiza el hash, marca el cambio obligatorio y
/// revoca las sesiones vigentes antes de enviarla por correo.
pub async fn regenerar_password_temporal(
    pool: &PgPool,
    reloj: &dyn Reloj,
    correo: &dyn EnvioCorreo,
    id: i32,
    actor_id: i32,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let usuario = sqlx::query_as!(
        Usuario,
        r#"SELECT id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en
           FROM usuarios
           WHERE id = $1 FOR UPDATE"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("usuario no encontrado".into()))?;

    let temporal = generar_password_temporal();
    let hash = hash_password(&temporal)?;
    let ahora = reloj.ahora();

    sqlx::query!(
        r#"UPDATE usuarios
           SET password_hash = $1, password_temporal = TRUE, actualizado_en = $2
           WHERE id = $3"#,
        hash,
        ahora,
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"UPDATE sesiones SET revocada_en = $1 WHERE usuario_id = $2 AND revocada_en IS NULL"#,
        ahora,
        id
    )
    .execute(&mut *tx)
    .await?;

    correo
        .enviar_password_temporal(&usuario.email, &usuario.nombre, &temporal)
        .await
        .map_err(error_envio)?;

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_USUARIOS,
        id,
        auditoria::ACCION_PASSWORD_TEMPORAL,
        actor_id,
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Bloquea a todos los gestores activos en orden de id. Se llama **antes** de
/// tocar la fila objetivo para que dos operaciones concurrentes sobre
/// administradores adquieran los locks en el mismo orden (evita `40P01`).
async fn bloquear_gestores_activos(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<i32>, AppError> {
    let gestores = sqlx::query_scalar!(
        r#"SELECT id FROM usuarios
           WHERE familia_rol = 'GESTION' AND activo
           ORDER BY id
           FOR UPDATE"#,
    )
    .fetch_all(&mut **tx)
    .await?;
    Ok(gestores)
}

/// Falla si el usuario bajo cambio es el último gestor activo.
fn asegurar_queda_gestor(gestores: &[i32], usuario: &Usuario) -> Result<(), AppError> {
    if !usuario.activo || usuario.familia_rol != "GESTION" {
        return Ok(());
    }
    if gestores.len() <= 1 {
        return Err(AppError::UltimoAdministrador);
    }
    Ok(())
}
