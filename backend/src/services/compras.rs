//! Servicio de compras (012).

use crate::error::AppError;
use crate::models::proveedor::{
    self as pm, Compra, CompraDetalle, CompraLinea, ListaCompras, ResultadoCompra,
};
use crate::services::{auditoria, movimientos, normalizar_paginacion};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashSet;

pub struct NuevaCompraLinea {
    pub producto_id: i32,
    pub cantidad: i32,
    pub costo_unitario: Decimal,
}

pub struct NuevaCompra {
    pub proveedor_id: i32,
    pub deposito_id: i32,
    pub numero: Option<String>,
    pub observacion: Option<String>,
    pub lineas: Vec<NuevaCompraLinea>,
}

pub struct FiltrosCompras {
    pub proveedor_id: Option<i32>,
    pub estado: Option<String>,
    pub desde: Option<DateTime<Utc>>,
    pub hasta: Option<DateTime<Utc>>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

fn proveedor_inactivo(nombre: &str) -> AppError {
    AppError::ProveedorInactivo(format!(
        "El proveedor «{nombre}» está dado de baja; reactivalo o elegí otro proveedor."
    ))
}

async fn validar_destinos(pool: &PgPool, proveedor_id: i32, deposito_id: i32) -> Result<(), AppError> {
    let (nombre, prov_activo): (String, bool) =
        sqlx::query_as("SELECT nombre, activo FROM proveedores WHERE id=$1")
            .bind(proveedor_id).fetch_optional(pool).await?
            .ok_or_else(|| AppError::RecursoNoEncontrado("El proveedor no existe.".into()))?;
    if !prov_activo { return Err(proveedor_inactivo(&nombre)); }

    let dep_activo: bool = sqlx::query_scalar("SELECT activo FROM depositos WHERE id=$1")
        .bind(deposito_id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("El depósito no existe.".into()))?;
    if !dep_activo { return Err(AppError::Validacion("El depósito está inactivo.".into())); }
    Ok(())
}

async fn validar_lineas(pool: &PgPool, lineas: &[NuevaCompraLinea]) -> Result<(), AppError> {
    if lineas.is_empty() {
        return Err(AppError::Validacion("La compra debe tener al menos una línea.".into()));
    }
    let mut productos: HashSet<i32> = HashSet::new();
    for l in lineas {
        if l.cantidad <= 0 {
            return Err(AppError::Validacion("La cantidad debe ser mayor a cero.".into()));
        }
        pm::validar_costo(l.costo_unitario)?;
        if !productos.insert(l.producto_id) {
            return Err(AppError::Validacion("No se puede repetir el mismo producto en dos líneas.".into()));
        }
        let activo: bool = sqlx::query_scalar("SELECT activo FROM productos WHERE id=$1")
            .bind(l.producto_id).fetch_optional(pool).await?
            .ok_or_else(|| AppError::RecursoNoEncontrado("El producto no existe.".into()))?;
        if !activo { return Err(AppError::ProductoInactivo); }
    }
    Ok(())
}

pub async fn crear(pool: &PgPool, nueva: NuevaCompra, usuario: &crate::models::usuario::Usuario) -> Result<CompraDetalle, AppError> {
    let numero = pm::validar_numero(nueva.numero.as_deref())?;
    validar_destinos(pool, nueva.proveedor_id, nueva.deposito_id).await?;
    validar_lineas(pool, &nueva.lineas).await?;

    let mut tx = pool.begin().await?;
    let total: Decimal = nueva.lineas.iter().map(|l| l.costo_unitario * Decimal::from(l.cantidad)).sum();
    let obs = pm::validar_observacion(nueva.observacion.as_deref())?;
    let compra_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO compras (proveedor_id, deposito_id, numero, estado, observacion, total_costo, usuario_id)
           VALUES ($1,$2,$3,'BORRADOR',$4,$5,$6) RETURNING id"#,
    )
    .bind(nueva.proveedor_id).bind(nueva.deposito_id).bind(numero.as_deref())
    .bind(obs.as_deref()).bind(total).bind(usuario.id)
    .fetch_one(&mut *tx).await?;

    for l in &nueva.lineas {
        let subtotal = l.costo_unitario * Decimal::from(l.cantidad);
        sqlx::query(
            r#"INSERT INTO compra_lineas (compra_id, producto_id, cantidad, costo_unitario, subtotal)
               VALUES ($1,$2,$3,$4,$5)"#,
        )
        .bind(compra_id).bind(l.producto_id).bind(l.cantidad).bind(l.costo_unitario).bind(subtotal)
        .execute(&mut *tx).await?;
    }
    auditoria::registrar(&mut tx, "compras", compra_id, auditoria::ACCION_CREAR, usuario.id).await?;
    tx.commit().await?;
    obtener(pool, compra_id).await
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<CompraDetalle, AppError> {
    let compra = sqlx::query_as::<_, Compra>(
        r#"SELECT c.id, c.proveedor_id, p.nombre AS proveedor_nombre, c.deposito_id, d.nombre AS deposito_nombre,
                  c.numero, c.estado, c.observacion, c.total_costo, c.usuario_id, u.nombre AS usuario_nombre,
                  c.creado_en, c.confirmado_en, c.anulado_en
             FROM compras c
             JOIN proveedores p ON p.id = c.proveedor_id
             JOIN depositos d ON d.id = c.deposito_id
             JOIN usuarios u ON u.id = c.usuario_id
            WHERE c.id = $1"#,
    )
    .bind(id).fetch_optional(pool).await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("La compra no existe.".into()))?;

    let lineas = sqlx::query_as::<_, CompraLinea>(
        r#"SELECT cl.id, cl.producto_id, prod.nombre AS producto_nombre, prod.sku,
                  cl.cantidad, cl.costo_unitario, cl.subtotal
             FROM compra_lineas cl
             JOIN productos prod ON prod.id = cl.producto_id
            WHERE cl.compra_id = $1
            ORDER BY cl.id"#,
    )
    .bind(id).fetch_all(pool).await?;

    Ok(CompraDetalle { compra, lineas })
}

pub async fn listar(pool: &PgPool, filtros: FiltrosCompras) -> Result<ListaCompras, AppError> {
    let (pagina, por_pagina, offset) = normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;
    let estado = pm::validar_estado(filtros.estado.as_deref())?;
    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM compras c
            WHERE ($1::int IS NULL OR c.proveedor_id = $1)
              AND ($2::text IS NULL OR c.estado = $2)
              AND ($3::timestamptz IS NULL OR c.creado_en >= $3)
              AND ($4::timestamptz IS NULL OR c.creado_en <= $4)"#,
    )
    .bind(filtros.proveedor_id).bind(estado.as_deref())
    .bind(filtros.desde).bind(filtros.hasta)
    .fetch_one(pool).await?;

    let items = sqlx::query_as::<_, Compra>(
        r#"SELECT c.id, c.proveedor_id, p.nombre AS proveedor_nombre, c.deposito_id, d.nombre AS deposito_nombre,
                  c.numero, c.estado, c.observacion, c.total_costo, c.usuario_id, u.nombre AS usuario_nombre,
                  c.creado_en, c.confirmado_en, c.anulado_en
             FROM compras c
             JOIN proveedores p ON p.id = c.proveedor_id
             JOIN depositos d ON d.id = c.deposito_id
             JOIN usuarios u ON u.id = c.usuario_id
            WHERE ($1::int IS NULL OR c.proveedor_id = $1)
              AND ($2::text IS NULL OR c.estado = $2)
              AND ($3::timestamptz IS NULL OR c.creado_en >= $3)
              AND ($4::timestamptz IS NULL OR c.creado_en <= $4)
            ORDER BY c.creado_en DESC, c.id DESC
            LIMIT $5 OFFSET $6"#,
    )
    .bind(filtros.proveedor_id).bind(estado.as_deref())
    .bind(filtros.desde).bind(filtros.hasta)
    .bind(por_pagina).bind(offset)
    .fetch_all(pool).await?;

    Ok(ListaCompras { items, total, pagina, por_pagina })
}

pub async fn actualizar(pool: &PgPool, id: i32, nueva: NuevaCompra, usuario_id: i32) -> Result<CompraDetalle, AppError> {
    let estado: String = sqlx::query_scalar("SELECT estado FROM compras WHERE id=$1")
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("La compra no existe.".into()))?;
    if estado != "BORRADOR" { return Err(AppError::CompraNoEditable); }
    let numero = pm::validar_numero(nueva.numero.as_deref())?;
    validar_destinos(pool, nueva.proveedor_id, nueva.deposito_id).await?;
    validar_lineas(pool, &nueva.lineas).await?;
    let obs = pm::validar_observacion(nueva.observacion.as_deref())?;
    // reuse crear logic: delete lineas and reinsert
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM compra_lineas WHERE compra_id=$1").bind(id).execute(&mut *tx).await?;
    let total: Decimal = nueva.lineas.iter().map(|l| l.costo_unitario * Decimal::from(l.cantidad)).sum();
    sqlx::query(
        "UPDATE compras SET proveedor_id=$1, deposito_id=$2, numero=$3, observacion=$4, total_costo=$5, actualizado_en=now() WHERE id=$6",
    )
    .bind(nueva.proveedor_id).bind(nueva.deposito_id).bind(numero.as_deref())
    .bind(obs.as_deref()).bind(total).bind(id)
    .execute(&mut *tx).await?;
    for l in &nueva.lineas {
        let subtotal = l.costo_unitario * Decimal::from(l.cantidad);
        sqlx::query(
            "INSERT INTO compra_lineas (compra_id, producto_id, cantidad, costo_unitario, subtotal) VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(id).bind(l.producto_id).bind(l.cantidad).bind(l.costo_unitario).bind(subtotal)
        .execute(&mut *tx).await?;
    }
    auditoria::registrar(&mut tx, "compras", id, auditoria::ACCION_ACTUALIZAR, usuario_id).await?;
    tx.commit().await?;
    obtener(pool, id).await
}

pub async fn eliminar(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let estado: String = sqlx::query_scalar("SELECT estado FROM compras WHERE id=$1")
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("La compra no existe.".into()))?;
    if estado != "BORRADOR" { return Err(AppError::CompraNoEditable); }
    sqlx::query("DELETE FROM compras WHERE id=$1").bind(id).execute(pool).await?;
    Ok(())
}

pub async fn confirmar(pool: &PgPool, id: i32, usuario: &crate::models::usuario::Usuario) -> Result<ResultadoCompra, AppError> {
    let mut tx = pool.begin().await?;
    let estado: String = sqlx::query_scalar("SELECT estado FROM compras WHERE id=$1 FOR UPDATE")
        .bind(id).fetch_optional(&mut *tx).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("La compra no existe.".into()))?;
    if estado == "CONFIRMADA" { return Err(AppError::CompraYaConfirmada); }
    if estado == "ANULADA" { return Err(AppError::CompraNoEditable); }
    if estado != "BORRADOR" { return Err(AppError::CompraNoEditable); }

    // load compra
    let (proveedor_id, deposito_id): (i32, i32) = sqlx::query_as("SELECT proveedor_id, deposito_id FROM compras WHERE id=$1")
        .bind(id).fetch_one(&mut *tx).await?;

    // validate proveedor/deposito active
    let (prov_nombre, prov_activo): (String, bool) =
        sqlx::query_as("SELECT nombre, activo FROM proveedores WHERE id=$1")
            .bind(proveedor_id).fetch_one(&mut *tx).await?;
    if !prov_activo { return Err(proveedor_inactivo(&prov_nombre)); }
    let dep: (String, bool) = sqlx::query_as("SELECT nombre, activo FROM depositos WHERE id=$1 FOR UPDATE").bind(deposito_id).fetch_one(&mut *tx).await?;
    if !dep.1 { return Err(AppError::Validacion("El depósito está inactivo.".into())); }

    // load lines
    let lineas: Vec<(i32, i32, Decimal)> = sqlx::query_as(
        "SELECT producto_id, cantidad, costo_unitario FROM compra_lineas WHERE compra_id=$1",
    )
    .bind(id).fetch_all(&mut *tx).await?;

    // validate products active
    let mut producto_ids: Vec<i32> = lineas.iter().map(|l| l.0).collect();
    producto_ids.sort(); producto_ids.dedup();
    let filas = sqlx::query_as::<_, (i32, bool)>(
        "SELECT id, activo FROM productos WHERE id = ANY($1) ORDER BY id FOR UPDATE",
    )
    .bind(&producto_ids).fetch_all(&mut *tx).await?;
    if filas.len() != producto_ids.len() { return Err(AppError::Validacion("Alguno de los productos no existe.".into())); }
    if let Some((_, false)) = filas.iter().find(|(_, a)| !*a) { return Err(AppError::ProductoInactivo); }

    // insert movimiento ENTRADA motivo COMPRA
    let movimiento_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id, compra_id)
           VALUES ('ENTRADA', $1, 'COMPRA', $2, $3, $4) RETURNING id"#,
    )
    .bind(deposito_id)
    .bind(format!("Compra #{}", id))
    .bind(usuario.id)
    .bind(id)
    .fetch_one(&mut *tx).await?;

    for (producto_id, cantidad, costo) in &lineas {
        // ensure stock row
        sqlx::query(
            r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
               VALUES ($1,$2,NULL,0)
               ON CONFLICT ON CONSTRAINT uq_stock_producto_deposito_ubicacion DO NOTHING"#,
        )
        .bind(producto_id).bind(deposito_id).execute(&mut *tx).await?;

        let actual: i32 = sqlx::query_scalar(
            "SELECT cantidad FROM stock_por_deposito WHERE producto_id=$1 AND deposito_id=$2 AND ubicacion_id IS NULL FOR UPDATE",
        )
        .bind(producto_id).bind(deposito_id).fetch_one(&mut *tx).await?;

        let resultante = actual.checked_add(*cantidad).ok_or_else(|| {
            AppError::Validacion(
                "La cantidad resultante supera el máximo admitido. Revisá la cantidad.".into(),
            )
        })?;
        sqlx::query("UPDATE stock_por_deposito SET cantidad=$1, actualizado_en=now() WHERE producto_id=$2 AND deposito_id=$3 AND ubicacion_id IS NULL")
            .bind(resultante).bind(producto_id).bind(deposito_id).execute(&mut *tx).await?;

        sqlx::query(
            "INSERT INTO movimiento_lineas (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante) VALUES ($1,$2,NULL,$3,$4)",
        )
        .bind(movimiento_id).bind(producto_id).bind(*cantidad).bind(resultante)
        .execute(&mut *tx).await?;

        // update producto costo + stock_actual
        sqlx::query("UPDATE productos SET costo=$1, actualizado_en=now() WHERE id=$2")
            .bind(costo).bind(producto_id).execute(&mut *tx).await?;
    }

    // recalc stock_actual
    sqlx::query(
        r#"UPDATE productos p SET stock_actual = COALESCE((SELECT SUM(s.cantidad) FROM stock_por_deposito s WHERE s.producto_id=p.id),0)::int, actualizado_en=now()
           WHERE p.id = ANY($1)"#,
    )
    .bind(&producto_ids).execute(&mut *tx).await?;

    // update compra
    sqlx::query("UPDATE compras SET estado='CONFIRMADA', confirmado_en=now(), actualizado_en=now() WHERE id=$1")
        .bind(id).execute(&mut *tx).await?;

    auditoria::registrar(&mut tx, "movimientos", movimiento_id, auditoria::ACCION_CREAR, usuario.id).await?;
    auditoria::registrar(&mut tx, "compras", id, "CONFIRMAR", usuario.id).await?;

    tx.commit().await?;
    resultado(pool, id, movimiento_id).await
}

/// Compra + movimiento generado, leídos después del commit (R2).
async fn resultado(pool: &PgPool, id: i32, movimiento_id: i32) -> Result<ResultadoCompra, AppError> {
    let CompraDetalle { compra, lineas } = obtener(pool, id).await?;
    let movimiento = movimientos::obtener(pool, movimiento_id).await?;
    Ok(ResultadoCompra { compra, lineas, movimiento })
}

pub async fn anular(pool: &PgPool, id: i32, usuario: &crate::models::usuario::Usuario) -> Result<ResultadoCompra, AppError> {
    let mut tx = pool.begin().await?;
    let (estado, deposito_id): (String, i32) =
        sqlx::query_as("SELECT estado, deposito_id FROM compras WHERE id=$1 FOR UPDATE")
            .bind(id).fetch_optional(&mut *tx).await?
            .ok_or_else(|| AppError::RecursoNoEncontrado("La compra no existe.".into()))?;
    match estado.as_str() {
        "CONFIRMADA" => {}
        "BORRADOR" => {
            return Err(AppError::CompraNoAnulable(
                "La compra está en borrador; no hay stock que revertir. Si no la vas a usar, eliminala.".into(),
            ))
        }
        _ => {
            return Err(AppError::CompraNoAnulable(
                "La compra ya fue anulada; no se puede anular dos veces.".into(),
            ))
        }
    }

    // Mismo orden de bloqueo que 007 (depósito → productos por id → filas de stock),
    // así una SALIDA concurrente espera o gana limpio, sin deadlock ni saldo negativo.
    let dep_nombre: String =
        sqlx::query_scalar("SELECT nombre FROM depositos WHERE id=$1 FOR UPDATE")
            .bind(deposito_id).fetch_one(&mut *tx).await?;
    let lineas: Vec<(i32, i32)> = sqlx::query_as(
        "SELECT producto_id, cantidad FROM compra_lineas WHERE compra_id=$1 ORDER BY producto_id",
    )
    .bind(id).fetch_all(&mut *tx).await?;
    let producto_ids: Vec<i32> = lineas.iter().map(|l| l.0).collect();
    sqlx::query("SELECT id FROM productos WHERE id = ANY($1) ORDER BY id FOR UPDATE")
        .bind(&producto_ids).execute(&mut *tx).await?;

    let movimiento_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id, compra_id)
           VALUES ('SALIDA', $1, 'DEVOLUCION', $2, $3, $4) RETURNING id"#,
    )
    .bind(deposito_id).bind(format!("Anulación compra #{}", id)).bind(usuario.id).bind(id)
    .fetch_one(&mut *tx).await?;

    for (producto_id, cantidad) in &lineas {
        // Descuento condicional: sin fila suficiente no se toca nada.
        let resultante: Option<i32> = sqlx::query_scalar(
            r#"UPDATE stock_por_deposito
                  SET cantidad = cantidad - $3, actualizado_en = now()
                WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL
                  AND cantidad >= $3
            RETURNING cantidad"#,
        )
        .bind(producto_id).bind(deposito_id).bind(cantidad)
        .fetch_optional(&mut *tx).await?;
        let Some(resultante) = resultante else {
            let disponibles: i32 = sqlx::query_scalar(
                "SELECT COALESCE((SELECT cantidad FROM stock_por_deposito WHERE producto_id=$1 AND deposito_id=$2 AND ubicacion_id IS NULL),0)",
            )
            .bind(producto_id).bind(deposito_id).fetch_one(&mut *tx).await?;
            return Err(AppError::StockInsuficiente {
                deposito: dep_nombre,
                disponibles,
                solicitadas: *cantidad,
            });
        };
        sqlx::query(
            "INSERT INTO movimiento_lineas (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante) VALUES ($1,$2,NULL,$3,$4)",
        )
        .bind(movimiento_id).bind(producto_id).bind(-*cantidad).bind(resultante)
        .execute(&mut *tx).await?;
    }
    sqlx::query(
        r#"UPDATE productos p SET stock_actual = COALESCE((SELECT SUM(s.cantidad) FROM stock_por_deposito s WHERE s.producto_id=p.id),0)::int, actualizado_en=now()
           WHERE p.id = ANY($1)"#,
    )
    .bind(&producto_ids).execute(&mut *tx).await?;

    sqlx::query("UPDATE compras SET estado='ANULADA', anulado_en=now(), actualizado_en=now() WHERE id=$1")
        .bind(id).execute(&mut *tx).await?;

    auditoria::registrar(&mut tx, "movimientos", movimiento_id, auditoria::ACCION_CREAR, usuario.id).await?;
    auditoria::registrar(&mut tx, "compras", id, "ANULAR", usuario.id).await?;

    tx.commit().await?;
    resultado(pool, id, movimiento_id).await
}
