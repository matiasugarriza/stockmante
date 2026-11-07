//! Servicio de ventas (013).

use crate::error::AppError;
use crate::models::usuario::Usuario;
use crate::models::venta::{self as vm, Venta, VentaDetalle, VentaLinea, ListaVentas};
use crate::services::{auditoria, normalizar_paginacion};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

/// Línea pedida: el precio siempre sale del catálogo (FR-050, R1).
pub struct NuevaVentaLinea {
    pub producto_id: i32,
    pub cantidad: i32,
}

/// Suma las cantidades por producto conservando el orden de primera aparición (R2).
pub fn agregar_lineas(lineas: &[NuevaVentaLinea]) -> Vec<(i32, i32)> {
    let mut agregadas: Vec<(i32, i32)> = Vec::new();
    for l in lineas {
        match agregadas.iter_mut().find(|(id, _)| *id == l.producto_id) {
            Some((_, cantidad)) => *cantidad = cantidad.saturating_add(l.cantidad),
            None => agregadas.push((l.producto_id, l.cantidad)),
        }
    }
    agregadas
}

pub struct NuevaVenta {
    pub deposito_id: i32,
    pub cliente_id: Option<i32>,
    pub medio_pago: String,
    pub descuento: Decimal,
    pub observacion: Option<String>,
    pub lineas: Vec<NuevaVentaLinea>,
}

pub struct FiltrosVentas {
    pub deposito_id: Option<i32>,
    pub cliente_id: Option<i32>,
    pub estado: Option<String>,
    pub desde: Option<DateTime<Utc>>,
    pub hasta: Option<DateTime<Utc>>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

pub async fn crear(pool: &PgPool, nueva: NuevaVenta, usuario: &Usuario) -> Result<VentaDetalle, AppError> {
    let medio = vm::validar_medio_pago(&nueva.medio_pago)?;
    let obs = vm::validar_observacion(nueva.observacion.as_deref())?;
    if nueva.lineas.is_empty() { return Err(AppError::Validacion("La venta debe tener al menos una línea.".into())); }
    for l in &nueva.lineas {
        if l.cantidad <= 0 { return Err(AppError::Validacion("La cantidad debe ser mayor a cero.".into())); }
    }
    if nueva.descuento < Decimal::ZERO { return Err(AppError::Validacion("El descuento no puede ser negativo.".into())); }
    let lineas = agregar_lineas(&nueva.lineas);

    let mut tx = pool.begin().await?;

    // Orden de bloqueo de 007: depósito → productos por id → filas de stock.
    let (dep_nombre, dep_activo): (String, bool) =
        sqlx::query_as("SELECT nombre, activo FROM depositos WHERE id=$1 FOR UPDATE")
            .bind(nueva.deposito_id).fetch_optional(&mut *tx).await?
            .ok_or_else(|| AppError::RecursoNoEncontrado("El depósito no existe.".into()))?;
    if !dep_activo { return Err(AppError::Validacion("El depósito está inactivo.".into())); }

    if let Some(cid) = nueva.cliente_id {
        let activo: bool = sqlx::query_scalar("SELECT activo FROM clientes WHERE id=$1")
            .bind(cid).fetch_optional(&mut *tx).await?
            .ok_or_else(|| AppError::RecursoNoEncontrado("El cliente no existe.".into()))?;
        if !activo { return Err(AppError::Validacion("El cliente está inactivo.".into())); }
    }

    let mut ids: Vec<i32> = lineas.iter().map(|l| l.0).collect();
    ids.sort_unstable();
    let filas: Vec<(i32, bool, Decimal)> = sqlx::query_as(
        "SELECT id, activo, precio_venta FROM productos WHERE id = ANY($1) ORDER BY id FOR UPDATE",
    )
    .bind(&ids).fetch_all(&mut *tx).await?;
    if filas.len() != ids.len() {
        return Err(AppError::RecursoNoEncontrado("El producto no existe.".into()));
    }
    if filas.iter().any(|(_, activo, _)| !*activo) {
        return Err(AppError::Validacion("El producto no está disponible para la venta.".into()));
    }

    // Precio congelado del catálogo al confirmar.
    let mut resueltas: Vec<(i32, i32, Decimal)> = Vec::with_capacity(lineas.len());
    let mut subtotal = Decimal::ZERO;
    for (producto_id, cantidad) in &lineas {
        let precio = filas
            .iter()
            .find(|(id, _, _)| id == producto_id)
            .map(|(_, _, p)| *p)
            .ok_or_else(|| AppError::RecursoNoEncontrado("El producto no existe.".into()))?;
        subtotal += precio * Decimal::from(*cantidad);
        resueltas.push((*producto_id, *cantidad, precio));
    }
    if nueva.descuento > subtotal { return Err(AppError::Validacion("El descuento no puede superar el subtotal.".into())); }
    let total = subtotal - nueva.descuento;

    let seq: i64 = sqlx::query_scalar("SELECT nextval('ventas_numero_seq')").fetch_one(&mut *tx).await?;
    let numero = format!("V-{:06}", seq);
    let venta_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO ventas (numero, deposito_id, cliente_id, medio_pago, subtotal, descuento, total, estado, observacion, usuario_id)
           VALUES ($1,$2,$3,$4,$5,$6,$7,'CONFIRMADA',$8,$9) RETURNING id"#,
    )
    .bind(&numero).bind(nueva.deposito_id).bind(nueva.cliente_id).bind(&medio)
    .bind(subtotal).bind(nueva.descuento).bind(total).bind(obs.as_deref()).bind(usuario.id)
    .fetch_one(&mut *tx).await?;

    let movimiento_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id, venta_id)
           VALUES ('SALIDA', $1, 'VENTA', $2, $3, $4) RETURNING id"#,
    )
    .bind(nueva.deposito_id).bind(format!("Venta {}", numero)).bind(usuario.id).bind(venta_id)
    .fetch_one(&mut *tx).await?;

    for (producto_id, cantidad, precio) in &resueltas {
        // Descuento condicional sobre la cantidad agregada: sin saldo no se escribe nada.
        let resultante: Option<i32> = sqlx::query_scalar(
            r#"UPDATE stock_por_deposito
                  SET cantidad = cantidad - $3, actualizado_en = now()
                WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL
                  AND cantidad >= $3
            RETURNING cantidad"#,
        )
        .bind(producto_id).bind(nueva.deposito_id).bind(cantidad)
        .fetch_optional(&mut *tx).await?;
        let Some(resultante) = resultante else {
            let disponibles: i32 = sqlx::query_scalar(
                "SELECT COALESCE((SELECT cantidad FROM stock_por_deposito WHERE producto_id=$1 AND deposito_id=$2 AND ubicacion_id IS NULL),0)",
            )
            .bind(producto_id).bind(nueva.deposito_id).fetch_one(&mut *tx).await?;
            return Err(AppError::StockInsuficiente {
                deposito: dep_nombre,
                disponibles,
                solicitadas: *cantidad,
            });
        };
        let sub = *precio * Decimal::from(*cantidad);
        sqlx::query(
            "INSERT INTO venta_lineas (venta_id, producto_id, cantidad, precio_unitario, subtotal) VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(venta_id).bind(producto_id).bind(cantidad).bind(precio).bind(sub)
        .execute(&mut *tx).await?;
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
    .bind(&ids).execute(&mut *tx).await?;

    auditoria::registrar(&mut tx, "movimientos", movimiento_id, auditoria::ACCION_CREAR, usuario.id).await?;
    auditoria::registrar(&mut tx, "ventas", venta_id, auditoria::ACCION_CREAR, usuario.id).await?;

    tx.commit().await?;
    obtener(pool, venta_id).await
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<VentaDetalle, AppError> {
    let venta = sqlx::query_as::<_, Venta>(
        r#"SELECT v.id, v.numero, v.deposito_id, d.nombre AS deposito_nombre, v.cliente_id, c.nombre AS cliente_nombre,
                  v.medio_pago, v.subtotal, v.descuento, v.total, v.estado, v.observacion, v.usuario_id, u.nombre AS usuario_nombre,
                  v.creado_en, v.anulado_en
             FROM ventas v
             JOIN depositos d ON d.id = v.deposito_id
             LEFT JOIN clientes c ON c.id = v.cliente_id
             JOIN usuarios u ON u.id = v.usuario_id
            WHERE v.id = $1"#,
    )
    .bind(id).fetch_optional(pool).await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("La venta no existe.".into()))?;

    let lineas = sqlx::query_as::<_, VentaLinea>(
        r#"SELECT vl.id, vl.producto_id, p.nombre AS producto_nombre, p.sku, vl.cantidad, vl.precio_unitario, vl.subtotal,
                  (SELECT ml.stock_resultante
                     FROM movimiento_lineas ml
                     JOIN movimientos m ON m.id = ml.movimiento_id
                    WHERE m.venta_id = vl.venta_id AND ml.producto_id = vl.producto_id
                    ORDER BY m.id DESC
                    LIMIT 1) AS stock_resultante
             FROM venta_lineas vl
             JOIN productos p ON p.id = vl.producto_id
            WHERE vl.venta_id = $1
            ORDER BY vl.id"#,
    )
    .bind(id).fetch_all(pool).await?;

    Ok(VentaDetalle { venta, lineas })
}

pub async fn listar(pool: &PgPool, filtros: FiltrosVentas) -> Result<ListaVentas, AppError> {
    let (pagina, por_pagina, offset) = normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;
    let estado = filtros.estado.map(|e| e.trim().to_uppercase());
    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM ventas v
            WHERE ($1::int IS NULL OR v.deposito_id = $1)
              AND ($2::int IS NULL OR v.cliente_id = $2)
              AND ($3::text IS NULL OR v.estado = $3)
              AND ($4::timestamptz IS NULL OR v.creado_en >= $4)
              AND ($5::timestamptz IS NULL OR v.creado_en <= $5)"#,
    )
    .bind(filtros.deposito_id).bind(filtros.cliente_id).bind(estado.as_deref())
    .bind(filtros.desde).bind(filtros.hasta)
    .fetch_one(pool).await?;

    let items = sqlx::query_as::<_, Venta>(
        r#"SELECT v.id, v.numero, v.deposito_id, d.nombre AS deposito_nombre, v.cliente_id, c.nombre AS cliente_nombre,
                  v.medio_pago, v.subtotal, v.descuento, v.total, v.estado, v.observacion, v.usuario_id, u.nombre AS usuario_nombre,
                  v.creado_en, v.anulado_en
             FROM ventas v
             JOIN depositos d ON d.id = v.deposito_id
             LEFT JOIN clientes c ON c.id = v.cliente_id
             JOIN usuarios u ON u.id = v.usuario_id
            WHERE ($1::int IS NULL OR v.deposito_id = $1)
              AND ($2::int IS NULL OR v.cliente_id = $2)
              AND ($3::text IS NULL OR v.estado = $3)
              AND ($4::timestamptz IS NULL OR v.creado_en >= $4)
              AND ($5::timestamptz IS NULL OR v.creado_en <= $5)
            ORDER BY v.creado_en DESC, v.id DESC
            LIMIT $6 OFFSET $7"#,
    )
    .bind(filtros.deposito_id).bind(filtros.cliente_id).bind(estado.as_deref())
    .bind(filtros.desde).bind(filtros.hasta)
    .bind(por_pagina).bind(offset)
    .fetch_all(pool).await?;

    Ok(ListaVentas { items, total, pagina, por_pagina })
}

pub async fn anular(pool: &PgPool, id: i32, usuario: &Usuario) -> Result<VentaDetalle, AppError> {
    let mut tx = pool.begin().await?;
    let estado: String = sqlx::query_scalar("SELECT estado FROM ventas WHERE id=$1 FOR UPDATE")
        .bind(id).fetch_optional(&mut *tx).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("La venta no existe.".into()))?;
    if estado == "ANULADA" { return Err(AppError::VentaYaAnulada); }
    if estado != "CONFIRMADA" { return Err(AppError::Validacion("La venta no está confirmada.".into())); }

    let (deposito_id,): (i32,) = sqlx::query_as("SELECT deposito_id FROM ventas WHERE id=$1").bind(id).fetch_one(&mut *tx).await?;
    let lineas: Vec<(i32, i32)> = sqlx::query_as("SELECT producto_id, cantidad FROM venta_lineas WHERE venta_id=$1 ORDER BY producto_id")
        .bind(id).fetch_all(&mut *tx).await?;
    sqlx::query("SELECT id FROM depositos WHERE id=$1 FOR UPDATE")
        .bind(deposito_id).execute(&mut *tx).await?;
    let bloqueo: Vec<i32> = lineas.iter().map(|l| l.0).collect();
    sqlx::query("SELECT id FROM productos WHERE id = ANY($1) ORDER BY id FOR UPDATE")
        .bind(&bloqueo).execute(&mut *tx).await?;

    let movimiento_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id, venta_id)
           VALUES ('ENTRADA', $1, 'DEVOLUCION', $2, $3, $4) RETURNING id"#,
    )
    .bind(deposito_id).bind(format!("Anulación venta #{}", id)).bind(usuario.id).bind(id)
    .fetch_one(&mut *tx).await?;

    let mut ids: Vec<i32> = lineas.iter().map(|l| l.0).collect();
    ids.sort(); ids.dedup();

    for (producto_id, cantidad) in &lineas {
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
            AppError::Validacion("La cantidad resultante supera el máximo admitido.".into())
        })?;
        sqlx::query("UPDATE stock_por_deposito SET cantidad=$1, actualizado_en=now() WHERE producto_id=$2 AND deposito_id=$3 AND ubicacion_id IS NULL")
            .bind(resultante).bind(producto_id).bind(deposito_id).execute(&mut *tx).await?;

        sqlx::query(
            "INSERT INTO movimiento_lineas (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante) VALUES ($1,$2,NULL,$3,$4)",
        )
        .bind(movimiento_id).bind(producto_id).bind(*cantidad).bind(resultante)
        .execute(&mut *tx).await?;
    }

    sqlx::query(
        r#"UPDATE productos p SET stock_actual = COALESCE((SELECT SUM(s.cantidad) FROM stock_por_deposito s WHERE s.producto_id=p.id),0)::int, actualizado_en=now()
           WHERE p.id = ANY($1)"#,
    )
    .bind(&ids).execute(&mut *tx).await?;

    sqlx::query("UPDATE ventas SET estado='ANULADA', anulado_en=now(), actualizado_en=now() WHERE id=$1")
        .bind(id).execute(&mut *tx).await?;

    auditoria::registrar(&mut tx, "movimientos", movimiento_id, auditoria::ACCION_CREAR, usuario.id).await?;
    auditoria::registrar(&mut tx, "ventas", id, "ANULAR", usuario.id).await?;

    tx.commit().await?;
    obtener(pool, id).await
}
