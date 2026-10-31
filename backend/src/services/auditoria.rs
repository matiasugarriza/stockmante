//! Auditoría básica de acciones sensibles (006, FR-019).
//!
//! Solo registra autor, acción, tabla/registro y timestamp; no guarda diffs ni
//! valores. Se invoca dentro de la misma transacción que el cambio auditado:
//! si la acción se revierte, la fila de auditoría también desaparece.

use crate::error::AppError;
use sqlx::{Postgres, Transaction};

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
