//! Transferencias entre depósitos (008): alta transaccional, historial y detalle.
//!
//! Toda escritura ocurre en una única transacción SQLx. El orden de bloqueo es siempre
//! `depositos` (por id ascendente) → `productos` → `stock_por_deposito`.

use crate::error::AppError;
use crate::models::stock::{
    DepositoBreve, ListaTransferencias, TransferenciaDetalle, TransferenciaFila, UsuarioBreve,
};
use crate::models::transferencia::{self, TransferenciaNueva};
use crate::models::usuario::Usuario;
use crate::services::{auditoria, normalizar_paginacion};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;

/// Alta de transferencia ya deserializada.
#[derive(Debug, Clone)]
pub struct NuevaTransferencia {
    pub producto_id: i32,
    pub deposito_origen_id: i32,
    pub deposito_destino_id: i32,
    pub cantidad: i32,
    pub ubicacion_origen_id: Option<i32>,
    pub ubicacion_destino_id: Option<i32>,
    pub observacion: Option<String>,
}

/// Filtros del historial.
#[derive(Debug, Clone, Default)]
pub struct FiltrosTransferencias {
    pub producto_id: Option<i32>,
    pub deposito_origen_id: Option<i32>,
    pub deposito_destino_id: Option<i32>,
    pub desde: Option<DateTime<Utc>>,
    pub hasta: Option<DateTime<Utc>>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct DepositoFila {
    nombre: String,
    activo: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct ProductoFila {
    nombre: String,
    activo: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct UbicacionFila {
    deposito_id: i32,
    activo: bool,
}

/// Crea una transferencia atómica y devuelve el detalle con stock resultante.
pub async fn crear(
    pool: &PgPool,
    nueva: NuevaTransferencia,
    usuario: &Usuario,
) -> Result<TransferenciaDetalle, AppError> {
    let observacion = transferencia::validar_transferencia(&TransferenciaNueva {
        producto_id: nueva.producto_id,
        deposito_origen_id: nueva.deposito_origen_id,
        deposito_destino_id: nueva.deposito_destino_id,
        cantidad: nueva.cantidad,
        observacion: nueva.observacion.clone(),
    })?;

    let mut tx = pool.begin().await?;

    // Bloquear depósitos por id ascendente.
    let mut ids = vec![nueva.deposito_origen_id, nueva.deposito_destino_id];
    ids.sort_unstable();
    ids.dedup();
    let mut depositos: HashMap<i32, DepositoFila> = HashMap::new();
    for id in ids {
        let fila = sqlx::query_as::<_, DepositoFila>(
            "SELECT nombre, activo FROM depositos WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validacion("El depósito indicado no existe.".into()))?;
        depositos.insert(id, fila);
    }

    let dep_origen = depositos
        .get(&nueva.deposito_origen_id)
        .expect("origen bloqueado");
    let dep_destino = depositos
        .get(&nueva.deposito_destino_id)
        .expect("destino bloqueado");

    if !dep_origen.activo {
        return Err(AppError::Validacion("El depósito de origen está inactivo.".into()));
    }
    if !dep_destino.activo {
        return Err(AppError::Validacion("El depósito de destino está inactivo.".into()));
    }

    // Bloquear producto.
    let producto = sqlx::query_as::<_, ProductoFila>(
        "SELECT nombre, activo FROM productos WHERE id = $1 FOR UPDATE",
    )
    .bind(nueva.producto_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Validacion("El producto indicado no existe.".into()))?;

    if !producto.activo {
        return Err(AppError::ProductoInactivo);
    }

    let nombre_producto = producto.nombre.clone();

    // Validar ubicación origen.
    if let Some(uo) = nueva.ubicacion_origen_id {
        let ub = sqlx::query_as::<_, UbicacionFila>(
            "SELECT deposito_id, activo FROM ubicaciones WHERE id = $1",
        )
        .bind(uo)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validacion("La ubicación indicada no existe.".into()))?;
        if ub.deposito_id != nueva.deposito_origen_id {
            return Err(AppError::Validacion(
                "La ubicación de origen no pertenece al depósito de origen.".into(),
            ));
        }
        if !ub.activo {
            return Err(AppError::Validacion("La ubicación de origen está inactiva.".into()));
        }
        let permitida: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM producto_ubicacion_permitida WHERE producto_id = $1 AND ubicacion_id = $2)",
        )
        .bind(nueva.producto_id)
        .bind(uo)
        .fetch_one(&mut *tx)
        .await?;
        if !permitida {
            return Err(AppError::Validacion(format!(
                "La ubicación no está permitida para {nombre_producto}."
            )));
        }
    }

    // Validar ubicación destino.
    if let Some(ud) = nueva.ubicacion_destino_id {
        let ub = sqlx::query_as::<_, UbicacionFila>(
            "SELECT deposito_id, activo FROM ubicaciones WHERE id = $1",
        )
        .bind(ud)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validacion("La ubicación indicada no existe.".into()))?;
        if ub.deposito_id != nueva.deposito_destino_id {
            return Err(AppError::Validacion(
                "La ubicación de destino no pertenece al depósito de destino.".into(),
            ));
        }
        if !ub.activo {
            return Err(AppError::Validacion("La ubicación de destino está inactiva.".into()));
        }
        let permitida: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM producto_ubicacion_permitida WHERE producto_id = $1 AND ubicacion_id = $2)",
        )
        .bind(nueva.producto_id)
        .bind(ud)
        .fetch_one(&mut *tx)
        .await?;
        if !permitida {
            return Err(AppError::Validacion(format!(
                "La ubicación no está permitida para {nombre_producto}."
            )));
        }
    }

    // Asegurar filas de stock (cantidad 0) en origen y destino.
    for (dep_id, ub_id) in [
        (nueva.deposito_origen_id, nueva.ubicacion_origen_id),
        (nueva.deposito_destino_id, nueva.ubicacion_destino_id),
    ] {
        sqlx::query(
            r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
               VALUES ($1, $2, $3, 0)
               ON CONFLICT ON CONSTRAINT uq_stock_producto_deposito_ubicacion DO NOTHING"#,
        )
        .bind(nueva.producto_id)
        .bind(dep_id)
        .bind(ub_id)
        .execute(&mut *tx)
        .await?;
    }

    // Bloquear y leer stock origen.
    let stock_origen_actual: i32 = sqlx::query_scalar(
        r#"SELECT cantidad FROM stock_por_deposito
            WHERE producto_id = $1 AND deposito_id = $2
              AND ubicacion_id IS NOT DISTINCT FROM $3
            FOR UPDATE"#,
    )
    .bind(nueva.producto_id)
    .bind(nueva.deposito_origen_id)
    .bind(nueva.ubicacion_origen_id)
    .fetch_one(&mut *tx)
    .await?;

    if stock_origen_actual < nueva.cantidad {
        return Err(AppError::StockInsuficiente {
            deposito: dep_origen.nombre.clone(),
            disponibles: stock_origen_actual,
            solicitadas: nueva.cantidad,
        });
    }

    // Bloquear y leer stock destino.
    let stock_destino_actual: i32 = sqlx::query_scalar(
        r#"SELECT cantidad FROM stock_por_deposito
            WHERE producto_id = $1 AND deposito_id = $2
              AND ubicacion_id IS NOT DISTINCT FROM $3
            FOR UPDATE"#,
    )
    .bind(nueva.producto_id)
    .bind(nueva.deposito_destino_id)
    .bind(nueva.ubicacion_destino_id)
    .fetch_one(&mut *tx)
    .await?;

    let stock_origen_resultante = stock_origen_actual - nueva.cantidad;
    let stock_destino_resultante = stock_destino_actual + nueva.cantidad;

    // Actualizar stock.
    sqlx::query(
        r#"UPDATE stock_por_deposito SET cantidad = $1, actualizado_en = now()
           WHERE producto_id = $2 AND deposito_id = $3
             AND ubicacion_id IS NOT DISTINCT FROM $4"#,
    )
    .bind(stock_origen_resultante)
    .bind(nueva.producto_id)
    .bind(nueva.deposito_origen_id)
    .bind(nueva.ubicacion_origen_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"UPDATE stock_por_deposito SET cantidad = $1, actualizado_en = now()
           WHERE producto_id = $2 AND deposito_id = $3
             AND ubicacion_id IS NOT DISTINCT FROM $4"#,
    )
    .bind(stock_destino_resultante)
    .bind(nueva.producto_id)
    .bind(nueva.deposito_destino_id)
    .bind(nueva.ubicacion_destino_id)
    .execute(&mut *tx)
    .await?;

    // Insertar movimiento SALIDA.
    let movimiento_salida_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id)
           VALUES ('SALIDA', $1, 'TRANSFERENCIA', $2, $3)
           RETURNING id"#,
    )
    .bind(nueva.deposito_origen_id)
    .bind(observacion.as_deref())
    .bind(usuario.id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"INSERT INTO movimiento_lineas
               (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(movimiento_salida_id)
    .bind(nueva.producto_id)
    .bind(nueva.ubicacion_origen_id)
    .bind(-nueva.cantidad)
    .bind(stock_origen_resultante)
    .execute(&mut *tx)
    .await?;

    // Insertar movimiento ENTRADA.
    let movimiento_entrada_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id)
           VALUES ('ENTRADA', $1, 'TRANSFERENCIA', $2, $3)
           RETURNING id"#,
    )
    .bind(nueva.deposito_destino_id)
    .bind(observacion.as_deref())
    .bind(usuario.id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"INSERT INTO movimiento_lineas
               (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(movimiento_entrada_id)
    .bind(nueva.producto_id)
    .bind(nueva.ubicacion_destino_id)
    .bind(nueva.cantidad)
    .bind(stock_destino_resultante)
    .execute(&mut *tx)
    .await?;

    // Insertar transferencia.
    let transferencia_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO transferencias
               (producto_id, deposito_origen_id, deposito_destino_id, cantidad,
                ubicacion_origen_id, ubicacion_destino_id, usuario_id, observacion,
                movimiento_salida_id, movimiento_entrada_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           RETURNING id"#,
    )
    .bind(nueva.producto_id)
    .bind(nueva.deposito_origen_id)
    .bind(nueva.deposito_destino_id)
    .bind(nueva.cantidad)
    .bind(nueva.ubicacion_origen_id)
    .bind(nueva.ubicacion_destino_id)
    .bind(usuario.id)
    .bind(observacion.as_deref())
    .bind(movimiento_salida_id)
    .bind(movimiento_entrada_id)
    .fetch_one(&mut *tx)
    .await?;

    // Vincular movimientos a la transferencia.
    sqlx::query(
        "UPDATE movimientos SET transferencia_id = $1 WHERE id = ANY($2)",
    )
    .bind(transferencia_id)
    .bind(&[movimiento_salida_id, movimiento_entrada_id])
    .execute(&mut *tx)
    .await?;

    // Recalcular cache del producto.
    sqlx::query(
        r#"UPDATE productos p
              SET stock_actual = COALESCE(
                      (SELECT SUM(s.cantidad) FROM stock_por_deposito s WHERE s.producto_id = p.id),
                      0
                  )::int,
                  actualizado_en = now()
            WHERE p.id = $1"#,
    )
    .bind(nueva.producto_id)
    .execute(&mut *tx)
    .await?;

    // Auditoría.
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_TRANSFERENCIAS,
        transferencia_id,
        auditoria::ACCION_CREAR,
        usuario.id,
    )
    .await?;

    tx.commit().await?;

    obtener(pool, transferencia_id).await
}

/// Historial paginado.
pub async fn listar(
    pool: &PgPool,
    filtros: FiltrosTransferencias,
) -> Result<ListaTransferencias, AppError> {
    let (pagina, por_pagina, offset) =
        normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*)
             FROM transferencias t
            WHERE ($1::int IS NULL OR t.producto_id = $1)
              AND ($2::int IS NULL OR t.deposito_origen_id = $2)
              AND ($3::int IS NULL OR t.deposito_destino_id = $3)
              AND ($4::timestamptz IS NULL OR t.creado_en >= $4)
              AND ($5::timestamptz IS NULL OR t.creado_en <= $5)"#,
    )
    .bind(filtros.producto_id)
    .bind(filtros.deposito_origen_id)
    .bind(filtros.deposito_destino_id)
    .bind(filtros.desde)
    .bind(filtros.hasta)
    .fetch_one(pool)
    .await?;

    let filas = sqlx::query_as::<_, TransferenciaFila>(
        r#"SELECT t.id, t.producto_id, p.nombre AS producto_nombre, p.sku,
                  t.deposito_origen_id, do.nombre AS deposito_origen_nombre,
                  t.deposito_destino_id, dd.nombre AS deposito_destino_nombre,
                  t.cantidad, t.stock_origen_resultante, t.stock_destino_resultante,
                  t.usuario_id, u.nombre AS usuario_nombre, t.creado_en,
                  t.movimiento_salida_id, t.movimiento_entrada_id, t.observacion
             FROM transferencias t
             JOIN productos p ON p.id = t.producto_id
             JOIN depositos do ON do.id = t.deposito_origen_id
             JOIN depositos dd ON dd.id = t.deposito_destino_id
             JOIN usuarios u ON u.id = t.usuario_id
            WHERE ($1::int IS NULL OR t.producto_id = $1)
              AND ($2::int IS NULL OR t.deposito_origen_id = $2)
              AND ($3::int IS NULL OR t.deposito_destino_id = $3)
              AND ($4::timestamptz IS NULL OR t.creado_en >= $4)
              AND ($5::timestamptz IS NULL OR t.creado_en <= $5)
            ORDER BY t.creado_en DESC, t.id DESC
            LIMIT $6 OFFSET $7"#,
    )
    .bind(filtros.producto_id)
    .bind(filtros.deposito_origen_id)
    .bind(filtros.deposito_destino_id)
    .bind(filtros.desde)
    .bind(filtros.hasta)
    .bind(por_pagina)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let items: Vec<TransferenciaDetalle> = filas
        .into_iter()
        .map(|f| TransferenciaDetalle {
            id: f.id,
            producto_id: f.producto_id,
            producto_nombre: f.producto_nombre,
            sku: f.sku,
            deposito_origen: DepositoBreve {
                id: f.deposito_origen_id,
                nombre: f.deposito_origen_nombre,
            },
            deposito_destino: DepositoBreve {
                id: f.deposito_destino_id,
                nombre: f.deposito_destino_nombre,
            },
            cantidad: f.cantidad,
            stock_origen_resultante: f.stock_origen_resultante,
            stock_destino_resultante: f.stock_destino_resultante,
            usuario: UsuarioBreve {
                id: f.usuario_id,
                nombre: f.usuario_nombre,
            },
            creado_en: f.creado_en,
            movimiento_salida_id: f.movimiento_salida_id,
            movimiento_entrada_id: f.movimiento_entrada_id,
            observacion: f.observacion,
        })
        .collect();

    Ok(ListaTransferencias {
        items,
        total,
        pagina,
        por_pagina,
    })
}

/// Detalle por id; 404 si no existe.
pub async fn obtener(pool: &PgPool, id: i32) -> Result<TransferenciaDetalle, AppError> {
    let fila = sqlx::query_as::<_, TransferenciaFila>(
        r#"SELECT t.id, t.producto_id, p.nombre AS producto_nombre, p.sku,
                  t.deposito_origen_id, do.nombre AS deposito_origen_nombre,
                  t.deposito_destino_id, dd.nombre AS deposito_destino_nombre,
                  t.cantidad, t.stock_origen_resultante, t.stock_destino_resultante,
                  t.usuario_id, u.nombre AS usuario_nombre, t.creado_en,
                  t.movimiento_salida_id, t.movimiento_entrada_id, t.observacion
             FROM transferencias t
             JOIN productos p ON p.id = t.producto_id
             JOIN depositos do ON do.id = t.deposito_origen_id
             JOIN depositos dd ON dd.id = t.deposito_destino_id
             JOIN usuarios u ON u.id = t.usuario_id
            WHERE t.id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("No existe la transferencia.".into()))?;

    Ok(TransferenciaDetalle {
        id: fila.id,
        producto_id: fila.producto_id,
        producto_nombre: fila.producto_nombre,
        sku: fila.sku,
        deposito_origen: DepositoBreve {
            id: fila.deposito_origen_id,
            nombre: fila.deposito_origen_nombre,
        },
        deposito_destino: DepositoBreve {
            id: fila.deposito_destino_id,
            nombre: fila.deposito_destino_nombre,
        },
        cantidad: fila.cantidad,
        stock_origen_resultante: fila.stock_origen_resultante,
        stock_destino_resultante: fila.stock_destino_resultante,
        usuario: UsuarioBreve {
            id: fila.usuario_id,
            nombre: fila.usuario_nombre,
        },
        creado_en: fila.creado_en,
        movimiento_salida_id: fila.movimiento_salida_id,
        movimiento_entrada_id: fila.movimiento_entrada_id,
        observacion: fila.observacion,
    })
}
