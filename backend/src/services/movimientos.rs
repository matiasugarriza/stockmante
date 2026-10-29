//! Movimientos de stock (007): alta transaccional, historial y detalle.
//!
//! Toda escritura ocurre en una única transacción SQLx. El orden de bloqueo es siempre
//! `depositos` → `productos` (por id ascendente) → `stock_por_deposito`: así dos movimientos
//! concurrentes no se piden los locks en orden inverso y no hay deadlocks.

use crate::error::AppError;
use crate::models::movimiento::{self, LineaNueva};
use crate::models::stock::{
    DepositoBreve, LineaFila, LineaMovimiento, ListaMovimientos, MovimientoCabecera,
    MovimientoDetalle, UsuarioBreve,
};
use crate::models::usuario::Usuario;
use crate::services::{auditoria, normalizar_paginacion};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;

/// Alta de movimiento ya deserializada.
#[derive(Debug, Clone)]
pub struct NuevoMovimiento {
    pub tipo: String,
    pub deposito_id: i32,
    pub motivo: String,
    pub observacion: Option<String>,
    pub lineas: Vec<LineaNueva>,
}

/// Filtros del historial.
#[derive(Debug, Clone, Default)]
pub struct FiltrosMovimientos {
    pub tipo: Option<String>,
    pub deposito_id: Option<i32>,
    pub producto_id: Option<i32>,
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
    id: i32,
    nombre: String,
    activo: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct UbicacionFila {
    deposito_id: i32,
    activo: bool,
}

/// Devuelve el movimiento recién creado con sus líneas y saldos resultantes.
///
/// Pasos, todos dentro de la misma transacción:
/// 1. valida tipo, motivo, observación y líneas (reglas puras);
/// 2. bloquea y valida el depósito (activo) y los productos (existentes y activos);
/// 3. valida cada ubicación (mismo depósito, activa y permitida para el producto);
/// 4. inserta la cabecera;
/// 5. por línea: asegura la fila de stock, la bloquea, valida el saldo y la actualiza;
/// 6. recalcula `productos.stock_actual` como suma de `stock_por_deposito`;
/// 7. audita y confirma.
pub async fn crear(
    pool: &PgPool,
    nuevo: NuevoMovimiento,
    usuario: &Usuario,
) -> Result<MovimientoDetalle, AppError> {
    let tipo = movimiento::validar_tipo(&nuevo.tipo)?;
    let motivo = movimiento::validar_motivo(&nuevo.motivo)?;
    let observacion = movimiento::validar_observacion(nuevo.observacion.as_deref(), &motivo)?;
    movimiento::validar_lineas(&tipo, &nuevo.lineas)?;

    if tipo == "AJUSTE" && usuario.familia_rol != "GESTION" {
        return Err(AppError::SinPermisos);
    }

    let mut tx = pool.begin().await?;

    let deposito = sqlx::query_as::<_, DepositoFila>(
        "SELECT nombre, activo FROM depositos WHERE id = $1 FOR UPDATE",
    )
    .bind(nuevo.deposito_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Validacion("El depósito indicado no existe.".into()))?;

    if !deposito.activo {
        return Err(AppError::Validacion("El depósito está inactivo.".into()));
    }

    let mut productos: Vec<i32> = nuevo.lineas.iter().map(|l| l.producto_id).collect();
    productos.sort_unstable();
    productos.dedup();

    let filas = sqlx::query_as::<_, ProductoFila>(
        "SELECT id, nombre, activo FROM productos WHERE id = ANY($1) ORDER BY id FOR UPDATE",
    )
    .bind(&productos)
    .fetch_all(&mut *tx)
    .await?;

    if filas.len() != productos.len() {
        return Err(AppError::Validacion(
            "Alguno de los productos indicados no existe.".into(),
        ));
    }
    if let Some(inactivo) = filas.iter().find(|fila| !fila.activo) {
        tracing::debug!(producto_id = inactivo.id, "movimiento rechazado por producto inactivo");
        return Err(AppError::ProductoInactivo);
    }

    let nombres: HashMap<i32, String> =
        filas.iter().map(|fila| (fila.id, fila.nombre.clone())).collect();

    for linea in &nuevo.lineas {
        let Some(ubicacion_id) = linea.ubicacion_id else {
            continue;
        };
        let ubicacion = sqlx::query_as::<_, UbicacionFila>(
            "SELECT deposito_id, activo FROM ubicaciones WHERE id = $1",
        )
        .bind(ubicacion_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validacion("La ubicación indicada no existe.".into()))?;

        if ubicacion.deposito_id != nuevo.deposito_id {
            return Err(AppError::Validacion(
                "La ubicación no pertenece al depósito del movimiento.".into(),
            ));
        }
        if !ubicacion.activo {
            return Err(AppError::Validacion("La ubicación está inactiva.".into()));
        }

        let permitida: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM producto_ubicacion_permitida
                             WHERE producto_id = $1 AND ubicacion_id = $2)",
        )
        .bind(linea.producto_id)
        .bind(ubicacion_id)
        .fetch_one(&mut *tx)
        .await?;

        if !permitida {
            let nombre = nombres
                .get(&linea.producto_id)
                .map(String::as_str)
                .unwrap_or("el producto");
            return Err(AppError::Validacion(format!(
                "La ubicación no está permitida para {nombre}."
            )));
        }
    }

    let movimiento_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(&tipo)
    .bind(nuevo.deposito_id)
    .bind(&motivo)
    .bind(observacion.as_deref())
    .bind(usuario.id)
    .fetch_one(&mut *tx)
    .await?;

    for linea in &nuevo.lineas {
        // Asegura la fila (cantidad 0) sin pisar una existente ni chocar en concurrencia.
        sqlx::query(
            r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
               VALUES ($1, $2, $3, 0)
               ON CONFLICT ON CONSTRAINT uq_stock_producto_deposito_ubicacion DO NOTHING"#,
        )
        .bind(linea.producto_id)
        .bind(nuevo.deposito_id)
        .bind(linea.ubicacion_id)
        .execute(&mut *tx)
        .await?;

        let actual: i32 = sqlx::query_scalar(
            r#"SELECT cantidad FROM stock_por_deposito
                WHERE producto_id = $1 AND deposito_id = $2
                  AND ubicacion_id IS NOT DISTINCT FROM $3
                FOR UPDATE"#,
        )
        .bind(linea.producto_id)
        .bind(nuevo.deposito_id)
        .bind(linea.ubicacion_id)
        .fetch_one(&mut *tx)
        .await?;

        let resultante = actual.checked_add(linea.cantidad_delta).ok_or_else(|| {
            AppError::Validacion("La cantidad resultante es demasiado grande.".into())
        })?;

        if resultante < 0 {
            return Err(AppError::StockInsuficiente {
                deposito: deposito.nombre.clone(),
                disponibles: actual,
                solicitadas: linea.cantidad_delta.unsigned_abs() as i32,
            });
        }

        sqlx::query(
            r#"UPDATE stock_por_deposito
                  SET cantidad = $1, actualizado_en = now()
                WHERE producto_id = $2 AND deposito_id = $3
                  AND ubicacion_id IS NOT DISTINCT FROM $4"#,
        )
        .bind(resultante)
        .bind(linea.producto_id)
        .bind(nuevo.deposito_id)
        .bind(linea.ubicacion_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO movimiento_lineas
                   (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(movimiento_id)
        .bind(linea.producto_id)
        .bind(linea.ubicacion_id)
        .bind(linea.cantidad_delta)
        .bind(resultante)
        .execute(&mut *tx)
        .await?;
    }

    // El cache se recalcula, no se incrementa: así no puede derivar de la tabla puente.
    sqlx::query(
        r#"UPDATE productos p
              SET stock_actual = COALESCE(
                      (SELECT SUM(s.cantidad) FROM stock_por_deposito s WHERE s.producto_id = p.id),
                      0
                  )::int,
                  actualizado_en = now()
            WHERE p.id = ANY($1)"#,
    )
    .bind(&productos)
    .execute(&mut *tx)
    .await?;

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_MOVIMIENTOS,
        movimiento_id,
        auditoria::ACCION_CREAR,
        usuario.id,
    )
    .await?;

    tx.commit().await?;

    obtener(pool, movimiento_id).await
}

/// Historial paginado, con las líneas de cada movimiento.
pub async fn listar(
    pool: &PgPool,
    filtros: FiltrosMovimientos,
) -> Result<ListaMovimientos, AppError> {
    let (pagina, por_pagina, offset) =
        normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;

    let tipo = match filtros.tipo {
        Some(valor) => Some(movimiento::validar_tipo(&valor)?),
        None => None,
    };

    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*)
             FROM movimientos m
            WHERE ($1::text IS NULL OR m.tipo = $1)
              AND ($2::int IS NULL OR m.deposito_id = $2)
              AND ($3::int IS NULL OR EXISTS (
                    SELECT 1 FROM movimiento_lineas l
                     WHERE l.movimiento_id = m.id AND l.producto_id = $3))
              AND ($4::timestamptz IS NULL OR m.creado_en >= $4)
              AND ($5::timestamptz IS NULL OR m.creado_en <= $5)"#,
    )
    .bind(tipo.as_deref())
    .bind(filtros.deposito_id)
    .bind(filtros.producto_id)
    .bind(filtros.desde)
    .bind(filtros.hasta)
    .fetch_one(pool)
    .await?;

    let cabeceras = sqlx::query_as::<_, MovimientoCabecera>(
        r#"SELECT m.id, m.tipo, m.deposito_id, d.nombre AS deposito_nombre,
                  m.motivo, m.observacion, m.usuario_id, u.nombre AS usuario_nombre,
                  m.creado_en
             FROM movimientos m
             JOIN depositos d ON d.id = m.deposito_id
             JOIN usuarios u ON u.id = m.usuario_id
            WHERE ($1::text IS NULL OR m.tipo = $1)
              AND ($2::int IS NULL OR m.deposito_id = $2)
              AND ($3::int IS NULL OR EXISTS (
                    SELECT 1 FROM movimiento_lineas l
                     WHERE l.movimiento_id = m.id AND l.producto_id = $3))
              AND ($4::timestamptz IS NULL OR m.creado_en >= $4)
              AND ($5::timestamptz IS NULL OR m.creado_en <= $5)
            ORDER BY m.creado_en DESC, m.id DESC
            LIMIT $6 OFFSET $7"#,
    )
    .bind(tipo.as_deref())
    .bind(filtros.deposito_id)
    .bind(filtros.producto_id)
    .bind(filtros.desde)
    .bind(filtros.hasta)
    .bind(por_pagina)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let ids: Vec<i32> = cabeceras.iter().map(|c| c.id).collect();
    let mut por_movimiento = lineas_de(pool, &ids).await?;

    let items = cabeceras
        .into_iter()
        .map(|c| MovimientoDetalle {
            id: c.id,
            tipo: c.tipo,
            deposito: DepositoBreve {
                id: c.deposito_id,
                nombre: c.deposito_nombre,
            },
            motivo: c.motivo,
            observacion: c.observacion,
            usuario: UsuarioBreve {
                id: c.usuario_id,
                nombre: c.usuario_nombre,
            },
            creado_en: c.creado_en,
            lineas: por_movimiento.remove(&c.id).unwrap_or_default(),
        })
        .collect();

    Ok(ListaMovimientos {
        items,
        total,
        pagina,
        por_pagina,
    })
}

/// Detalle de un movimiento; 404 si no existe.
pub async fn obtener(pool: &PgPool, id: i32) -> Result<MovimientoDetalle, AppError> {
    let cabecera = sqlx::query_as::<_, MovimientoCabecera>(
        r#"SELECT m.id, m.tipo, m.deposito_id, d.nombre AS deposito_nombre,
                  m.motivo, m.observacion, m.usuario_id, u.nombre AS usuario_nombre,
                  m.creado_en
             FROM movimientos m
             JOIN depositos d ON d.id = m.deposito_id
             JOIN usuarios u ON u.id = m.usuario_id
            WHERE m.id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("El movimiento no existe.".into()))?;

    let lineas = lineas_de(pool, &[cabecera.id])
        .await?
        .remove(&cabecera.id)
        .unwrap_or_default();

    Ok(MovimientoDetalle {
        id: cabecera.id,
        tipo: cabecera.tipo,
        deposito: DepositoBreve {
            id: cabecera.deposito_id,
            nombre: cabecera.deposito_nombre,
        },
        motivo: cabecera.motivo,
        observacion: cabecera.observacion,
        usuario: UsuarioBreve {
            id: cabecera.usuario_id,
            nombre: cabecera.usuario_nombre,
        },
        creado_en: cabecera.creado_en,
        lineas,
    })
}

/// Líneas de varios movimientos agrupadas por `movimiento_id`, en una sola consulta.
async fn lineas_de(
    pool: &PgPool,
    ids: &[i32],
) -> Result<HashMap<i32, Vec<LineaMovimiento>>, AppError> {
    let mut agrupadas: HashMap<i32, Vec<LineaMovimiento>> = HashMap::new();
    if ids.is_empty() {
        return Ok(agrupadas);
    }

    let filas = sqlx::query_as::<_, LineaFila>(
        r#"SELECT l.movimiento_id, l.producto_id, p.nombre AS producto_nombre, p.sku,
                  l.ubicacion_id, u.nombre AS ubicacion_nombre,
                  l.cantidad_delta, l.stock_resultante
             FROM movimiento_lineas l
             JOIN productos p ON p.id = l.producto_id
             LEFT JOIN ubicaciones u ON u.id = l.ubicacion_id
            WHERE l.movimiento_id = ANY($1)
            ORDER BY l.id"#,
    )
    .bind(ids)
    .fetch_all(pool)
    .await?;

    for fila in filas {
        agrupadas
            .entry(fila.movimiento_id)
            .or_default()
            .push(fila.into());
    }
    Ok(agrupadas)
}
