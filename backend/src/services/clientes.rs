//! Servicio de clientes (013).

use crate::error::AppError;
use crate::models::cliente::{self as vm, Cliente, ListaClientes};
use crate::services::{auditoria, normalizar_paginacion};
use sqlx::PgPool;

pub struct NuevoCliente {
    pub nombre: String,
    pub documento: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub direccion: Option<String>,
}

pub struct FiltrosClientes {
    pub activo: Option<bool>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

pub async fn crear(pool: &PgPool, nuevo: NuevoCliente, usuario_id: i32) -> Result<Cliente, AppError> {
    let nombre = vm::validar_nombre(&nuevo.nombre)?;
    let documento = vm::validar_documento(nuevo.documento.as_deref())?;
    let email = vm::normalizar_opcional(nuevo.email.as_deref(), vm::EMAIL_MAX, "El email")?;
    let telefono = vm::normalizar_opcional(nuevo.telefono.as_deref(), vm::TELEFONO_MAX, "El teléfono")?;
    let direccion = vm::normalizar_opcional(nuevo.direccion.as_deref(), vm::DIRECCION_MAX, "La dirección")?;

    if let Some(d) = &documento {
        let dup: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clientes WHERE documento=$1)")
            .bind(d).fetch_one(pool).await?;
        if dup { return Err(AppError::DocumentoDuplicado); }
    }

    let id: i32 = sqlx::query_scalar(
        "INSERT INTO clientes (nombre, documento, email, telefono, direccion) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(&nombre).bind(&documento).bind(&email).bind(&telefono).bind(&direccion)
    .fetch_one(pool).await?;

    let mut tx = pool.begin().await?;
    auditoria::registrar(&mut tx, "clientes", id, auditoria::ACCION_CREAR, usuario_id).await?;
    tx.commit().await?;
    obtener(pool, id).await
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<Cliente, AppError> {
    sqlx::query_as::<_, Cliente>("SELECT id, nombre, documento, email, telefono, direccion, activo FROM clientes WHERE id=$1")
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("El cliente no existe.".into()))
}

pub async fn listar(pool: &PgPool, filtros: FiltrosClientes) -> Result<ListaClientes, AppError> {
    let (pagina, por_pagina, offset) = normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;
    let activo = filtros.activo;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM clientes WHERE ($1::bool IS NULL OR activo=$1)")
        .bind(activo).fetch_one(pool).await?;
    let items = sqlx::query_as::<_, Cliente>(
        "SELECT id, nombre, documento, email, telefono, direccion, activo FROM clientes WHERE ($1::bool IS NULL OR activo=$1) ORDER BY nombre LIMIT $2 OFFSET $3",
    )
    .bind(activo).bind(por_pagina).bind(offset).fetch_all(pool).await?;
    Ok(ListaClientes { items, total, pagina, por_pagina })
}

pub async fn actualizar(pool: &PgPool, id: i32, nuevo: NuevoCliente, usuario_id: i32) -> Result<Cliente, AppError> {
    let existe: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clientes WHERE id=$1)").bind(id).fetch_one(pool).await?;
    if !existe { return Err(AppError::RecursoNoEncontrado("El cliente no existe.".into())); }
    let nombre = vm::validar_nombre(&nuevo.nombre)?;
    let documento = vm::validar_documento(nuevo.documento.as_deref())?;
    let email = vm::normalizar_opcional(nuevo.email.as_deref(), vm::EMAIL_MAX, "El email")?;
    let telefono = vm::normalizar_opcional(nuevo.telefono.as_deref(), vm::TELEFONO_MAX, "El teléfono")?;
    let direccion = vm::normalizar_opcional(nuevo.direccion.as_deref(), vm::DIRECCION_MAX, "La dirección")?;

    if let Some(d) = &documento {
        let dup: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clientes WHERE documento=$1 AND id<>$2)")
            .bind(d).bind(id).fetch_one(pool).await?;
        if dup { return Err(AppError::DocumentoDuplicado); }
    }

    sqlx::query("UPDATE clientes SET nombre=$1, documento=$2, email=$3, telefono=$4, direccion=$5, actualizado_en=now() WHERE id=$6")
        .bind(&nombre).bind(&documento).bind(&email).bind(&telefono).bind(&direccion).bind(id)
        .execute(pool).await?;

    let mut tx = pool.begin().await?;
    auditoria::registrar(&mut tx, "clientes", id, auditoria::ACCION_ACTUALIZAR, usuario_id).await?;
    tx.commit().await?;
    obtener(pool, id).await
}

pub async fn set_activo(pool: &PgPool, id: i32, activo: bool, usuario_id: i32) -> Result<Cliente, AppError> {
    let res = sqlx::query("UPDATE clientes SET activo=$1, actualizado_en=now() WHERE id=$2")
        .bind(activo).bind(id).execute(pool).await?;
    if res.rows_affected() == 0 { return Err(AppError::RecursoNoEncontrado("El cliente no existe.".into())); }
    let mut tx = pool.begin().await?;
    let accion = if activo { auditoria::ACCION_REACTIVAR } else { auditoria::ACCION_DESACTIVAR };
    auditoria::registrar(&mut tx, "clientes", id, accion, usuario_id).await?;
    tx.commit().await?;
    obtener(pool, id).await
}
