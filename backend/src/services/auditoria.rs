//! Auditoría básica de acciones sensibles (006, FR-019).
//!
//! Solo registra autor, acción, tabla/registro y timestamp; no guarda diffs ni
//! valores. Se invoca dentro de la misma transacción que el cambio auditado:
//! si la acción se revierte, la fila de auditoría también desaparece.

use crate::error::AppError;
use crate::services::normalizar_paginacion;
use sqlx::{Postgres, Transaction};
use sqlx::PgPool;

/// Tabla de referencia de las acciones auditadas.
pub const TABLA_USUARIOS: &str = "usuarios";
pub const TABLA_PRODUCTOS: &str = "productos";
pub const TABLA_MOVIMIENTOS: &str = "movimientos";
pub const TABLA_TRANSFERENCIAS: &str = "transferencias";
pub const TABLA_CONTEOS: &str = "conteos";

/// Acciones auditadas (valores estables en `auditoria.accion`).
pub const ACCION_CREAR: &str = "CREAR";
pub const ACCION_ACTUALIZAR: &str = "ACTUALIZAR";
pub const ACCION_DESACTIVAR: &str = "DESACTIVAR";
pub const ACCION_REACTIVAR: &str = "REACTIVAR";
pub const ACCION_PASSWORD_TEMPORAL: &str = "PASSWORD_TEMPORAL";

/// Inserta la fila de auditoría usando la transacción del cambio auditado.
pub async fn registrar(
    tx: &mut Transaction<'_, Postgres>,
    tabla_ref: &str,
    registro_id: i32,
    accion: &str,
    usuario_id: i32,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO auditoria (tabla_ref, registro_id, accion, usuario_id)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(tabla_ref)
    .bind(registro_id)
    .bind(accion)
    .bind(usuario_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
pub struct AuditoriaFila {
    pub id: i32,
    pub tabla_ref: String,
    pub registro_id: i32,
    pub accion: String,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub cuando: chrono::DateTime<chrono::Utc>,
}

pub async fn listar(
    pool: &PgPool,
    tabla_ref: Option<&str>,
    usuario_id: Option<i32>,
    desde: Option<chrono::DateTime<chrono::Utc>>,
    hasta: Option<chrono::DateTime<chrono::Utc>>,
    pagina: Option<i64>,
    por_pagina: Option<i64>,
) -> Result<(Vec<AuditoriaFila>, i64), AppError> {
    let (_p, pp, offset) = normalizar_paginacion(pagina, por_pagina)?;
    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM auditoria a
           WHERE ($1::text IS NULL OR a.tabla_ref = $1)
             AND ($2::int IS NULL OR a.usuario_id = $2)
             AND ($3::timestamptz IS NULL OR a.cuando >= $3)
             AND ($4::timestamptz IS NULL OR a.cuando <= $4)"#,
    )
    .bind(tabla_ref)
    .bind(usuario_id)
    .bind(desde)
    .bind(hasta)
    .fetch_one(pool)
    .await?;
    let filas = sqlx::query_as::<_, AuditoriaFila>(
        r#"SELECT a.id, a.tabla_ref, a.registro_id, a.accion, a.usuario_id, u.nombre AS usuario_nombre, a.cuando
           FROM auditoria a JOIN usuarios u ON u.id = a.usuario_id
           WHERE ($1::text IS NULL OR a.tabla_ref = $1)
             AND ($2::int IS NULL OR a.usuario_id = $2)
             AND ($3::timestamptz IS NULL OR a.cuando >= $3)
             AND ($4::timestamptz IS NULL OR a.cuando <= $4)
           ORDER BY a.cuando DESC, a.id DESC
           LIMIT $5 OFFSET $6"#,
    )
    .bind(tabla_ref)
    .bind(usuario_id)
    .bind(desde)
    .bind(hasta)
    .bind(pp)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok((filas, total))
}