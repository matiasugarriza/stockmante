//! Servicio de proveedores (012).

use crate::error::AppError;
use crate::models::proveedor::{self, Proveedor, ListaProveedores};
use crate::services::{auditoria, normalizar_paginacion};
use sqlx::PgPool;

pub struct NuevoProveedor {
    pub nombre: String,
    pub cuit: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub direccion: Option<String>,
}

pub struct FiltrosProveedores {
    pub activo: Option<bool>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

pub async fn crear(pool: &PgPool, nuevo: NuevoProveedor, usuario_id: i32) -> Result<Proveedor, AppError> {
    let nombre = proveedor::validar_nombre(&nuevo.nombre)?;
    let cuit = proveedor::validar_cuit(nuevo.cuit.as_deref())?;
    let email = proveedor::normalizar_opcional(nuevo.email.as_deref(), proveedor::EMAIL_MAX, "El email")?;
    let telefono = proveedor::normalizar_opcional(nuevo.telefono.as_deref(), proveedor::TELEFONO_MAX, "El teléfono")?;
    let direccion = proveedor::normalizar_opcional(nuevo.direccion.as_deref(), proveedor::DIRECCION_MAX, "La dirección")?;

    let existe: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM proveedores WHERE lower(nombre) = lower($1))",
    )
    .bind(&nombre)
    .fetch_one(pool)
    .await?;
    if existe {
        return Err(AppError::ProveedorDuplicado);
    }
    if let Some(c) = &cuit {
        let dup: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM proveedores WHERE cuit = $1)")
            .bind(c)
            .fetch_one(pool)
            .await?;
        if dup {
            return Err(AppError::CuitDuplicado);
        }
    }

    let id: i32 = sqlx::query_scalar(
        r#"INSERT INTO proveedores (nombre, cuit, email, telefono, direccion)
           VALUES ($1,$2,$3,$4,$5) RETURNING id"#,
    )
    .bind(&nombre).bind(&cuit).bind(&email).bind(&telefono).bind(&direccion)
    .fetch_one(pool).await?;

    let prov = obtener(pool, id).await?;
    // auditoria best-effort: registrar
    let mut tx = pool.begin().await?;
    auditoria::registrar(&mut tx, "proveedores", id, auditoria::ACCION_CREAR, usuario_id).await?;
    tx.commit().await?;
    Ok(prov)
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<Proveedor, AppError> {
    sqlx::query_as::<_, Proveedor>(
        "SELECT id, nombre, cuit, email, telefono, direccion, activo FROM proveedores WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("El proveedor no existe.".into()))
}

pub async fn listar(pool: &PgPool, filtros: FiltrosProveedores) -> Result<ListaProveedores, AppError> {
    let (pagina, por_pagina, offset) = normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;
    let activo = filtros.activo;
    let total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM proveedores WHERE ($1::bool IS NULL OR activo = $1)",
    )
    .bind(activo)
    .fetch_one(pool).await?;
    let items = sqlx::query_as::<_, Proveedor>(
        r#"SELECT id, nombre, cuit, email, telefono, direccion, activo
             FROM proveedores
            WHERE ($1::bool IS NULL OR activo = $1)
            ORDER BY nombre
            LIMIT $2 OFFSET $3"#,
    )
    .bind(activo).bind(por_pagina).bind(offset)
    .fetch_all(pool).await?;
    Ok(ListaProveedores { items, total, pagina, por_pagina })
}

pub async fn actualizar(pool: &PgPool, id: i32, nuevo: NuevoProveedor, usuario_id: i32) -> Result<Proveedor, AppError> {
    let existe = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM proveedores WHERE id=$1)")
        .bind(id).fetch_one(pool).await?;
    if !existe { return Err(AppError::RecursoNoEncontrado("El proveedor no existe.".into())); }
    let nombre = proveedor::validar_nombre(&nuevo.nombre)?;
    let cuit = proveedor::validar_cuit(nuevo.cuit.as_deref())?;
    let email = proveedor::normalizar_opcional(nuevo.email.as_deref(), proveedor::EMAIL_MAX, "El email")?;
    let telefono = proveedor::normalizar_opcional(nuevo.telefono.as_deref(), proveedor::TELEFONO_MAX, "El teléfono")?;
    let direccion = proveedor::normalizar_opcional(nuevo.direccion.as_deref(), proveedor::DIRECCION_MAX, "La dirección")?;

    // duplicado nombre
    let dup: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM proveedores WHERE lower(nombre)=lower($1) AND id<>$2)")
        .bind(&nombre).bind(id).fetch_one(pool).await?;
    if dup { return Err(AppError::ProveedorDuplicado); }

    // duplicado CUIT (si se informa)
    if let Some(c) = &cuit {
        let dup_cuit: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM proveedores WHERE cuit = $1 AND id<>$2)")
            .bind(c).bind(id).fetch_one(pool).await?;
        if dup_cuit { return Err(AppError::CuitDuplicado); }
    }

    sqlx::query(
        "UPDATE proveedores SET nombre=$1, cuit=$2, email=$3, telefono=$4, direccion=$5, actualizado_en=now() WHERE id=$6",
    )
    .bind(&nombre).bind(&cuit).bind(&email).bind(&telefono).bind(&direccion).bind(id)
    .execute(pool).await?;

    let mut tx = pool.begin().await?;
    auditoria::registrar(&mut tx, "proveedores", id, auditoria::ACCION_ACTUALIZAR, usuario_id).await?;
    tx.commit().await?;
    obtener(pool, id).await
}

pub async fn set_activo(pool: &PgPool, id: i32, activo: bool, usuario_id: i32) -> Result<Proveedor, AppError> {
    let res = sqlx::query("UPDATE proveedores SET activo=$1, actualizado_en=now() WHERE id=$2")
        .bind(activo).bind(id).execute(pool).await?;
    if res.rows_affected() == 0 { return Err(AppError::RecursoNoEncontrado("El proveedor no existe.".into())); }
    let mut tx = pool.begin().await?;
    let accion = if activo { auditoria::ACCION_REACTIVAR } else { auditoria::ACCION_DESACTIVAR };
    auditoria::registrar(&mut tx, "proveedores", id, accion, usuario_id).await?;
    tx.commit().await?;
    obtener(pool, id).await
}
