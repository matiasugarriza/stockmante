//! Reportes (014): kardex, stock valorizado, rotación y ventas. Solo lectura.
//!
//! Las fechas son días locales UTC−03:00 (inclusivos) convertidos a límites UTC
//! semiabiertos por `rango`; ningún servicio usa el reloj. Los cálculos puros viven en
//! `calculo` y el CSV en `csv`.

pub mod calculo;
pub mod csv;
pub mod rango;

use crate::error::AppError;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KardexLinea {
    pub movimiento_id: i32,
    pub creado_en: DateTime<Utc>,
    pub tipo: String,
    pub motivo: String,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub cantidad: i32,
    pub saldo: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KardexReporte {
    pub producto_id: i32,
    pub producto_nombre: String,
    /// Filtro aplicado; `null` = todos los depósitos.
    pub deposito_id: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub desde: Option<NaiveDate>,
    #[schema(value_type = Option<String>)]
    pub hasta: Option<NaiveDate>,
    pub saldo_inicial: i32,
    pub lineas: Vec<KardexLinea>,
    pub saldo_final: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StockValorizadoItem {
    pub producto_id: i32,
    pub nombre: String,
    pub sku: String,
    /// Σ `stock_por_deposito.cantidad` (bigint: la suma de varios depósitos puede superar i32).
    pub cantidad: i64,
    #[schema(value_type = f64)]
    pub costo_unitario: Decimal,
    #[schema(value_type = f64)]
    pub valor: Decimal,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StockValorizadoReporte {
    pub items: Vec<StockValorizadoItem>,
    #[schema(value_type = f64)]
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RotacionItem {
    pub producto_id: i32,
    pub nombre: String,
    pub unidades_salidas: i32,
    pub stock_inicial: i32,
    pub stock_final: i32,
    pub stock_promedio: f64,
    pub indice_rotacion: f64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RotacionReporte {
    #[schema(value_type = String)]
    pub desde: NaiveDate,
    #[schema(value_type = String)]
    pub hasta: NaiveDate,
    pub items: Vec<RotacionItem>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VentaResumenDia {
    pub dia: String,
    pub cantidad_ventas: i64,
    #[schema(value_type = f64)]
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VentaResumenProducto {
    pub producto_id: i32,
    pub nombre: String,
    pub unidades: i32,
    #[schema(value_type = f64)]
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VentasReporte {
    #[schema(value_type = String)]
    pub desde: NaiveDate,
    #[schema(value_type = String)]
    pub hasta: NaiveDate,
    pub cantidad_ventas: i64,
    #[schema(value_type = f64)]
    pub total: Decimal,
    pub por_dia: Vec<VentaResumenDia>,
    pub por_producto: Vec<VentaResumenProducto>,
}

/// FR-056, R5: saldo inicial (antes de `desde`), líneas en el rango y saldo corrido.
pub async fn kardex(
    pool: &PgPool,
    producto_id: i32,
    deposito_id: Option<i32>,
    desde: Option<NaiveDate>,
    hasta: Option<NaiveDate>,
) -> Result<KardexReporte, AppError> {
    let (inicio, fin) = rango::limites_utc(desde, hasta)?;
    let producto_nombre: String = sqlx::query_scalar("SELECT nombre FROM productos WHERE id=$1")
        .bind(producto_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("El producto no existe.".into()))?;

    let saldo_inicial: i32 = match inicio {
        Some(inicio) => {
            sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(ml.cantidad_delta), 0)::int
                     FROM movimiento_lineas ml
                     JOIN movimientos m ON m.id = ml.movimiento_id
                    WHERE ml.producto_id = $1
                      AND ($2::int IS NULL OR m.deposito_id = $2)
                      AND m.creado_en < $3"#,
            )
            .bind(producto_id)
            .bind(deposito_id)
            .bind(inicio)
            .fetch_one(pool)
            .await?
        }
        None => 0,
    };

    let filas: Vec<(i32, DateTime<Utc>, String, String, i32, String, i32)> = sqlx::query_as(
        r#"SELECT m.id, m.creado_en, m.tipo, m.motivo, m.deposito_id, d.nombre, ml.cantidad_delta
             FROM movimiento_lineas ml
             JOIN movimientos m ON m.id = ml.movimiento_id
             JOIN depositos d ON d.id = m.deposito_id
            WHERE ml.producto_id = $1
              AND ($2::int IS NULL OR m.deposito_id = $2)
              AND ($3::timestamptz IS NULL OR m.creado_en >= $3)
              AND ($4::timestamptz IS NULL OR m.creado_en < $4)
            ORDER BY m.creado_en ASC, m.id ASC, ml.id ASC"#,
    )
    .bind(producto_id)
    .bind(deposito_id)
    .bind(inicio)
    .bind(fin)
    .fetch_all(pool)
    .await?;

    let deltas: Vec<i32> = filas.iter().map(|f| f.6).collect();
    let saldos = calculo::acumular_saldos(saldo_inicial, &deltas);
    let saldo_final = saldos.last().copied().unwrap_or(saldo_inicial);
    let lineas = filas
        .into_iter()
        .zip(saldos)
        .map(
            |((movimiento_id, creado_en, tipo, motivo, deposito_id, deposito_nombre, cantidad), saldo)| {
                KardexLinea {
                    movimiento_id,
                    creado_en,
                    tipo,
                    motivo,
                    deposito_id,
                    deposito_nombre,
                    cantidad,
                    saldo,
                }
            },
        )
        .collect();

    Ok(KardexReporte {
        producto_id,
        producto_nombre,
        deposito_id,
        desde,
        hasta,
        saldo_inicial,
        lineas,
        saldo_final,
    })
}

/// FR-057, R4: desde `stock_por_deposito` (fuente de verdad), activos o no, solo con stock.
pub async fn stock_valorizado(pool: &PgPool) -> Result<StockValorizadoReporte, AppError> {
    let filas: Vec<(i32, String, String, i64, Decimal)> = sqlx::query_as(
        r#"SELECT p.id, p.nombre, p.sku, SUM(s.cantidad)::bigint AS cantidad, p.costo
             FROM stock_por_deposito s
             JOIN productos p ON p.id = s.producto_id
            GROUP BY p.id, p.nombre, p.sku, p.costo
           HAVING SUM(s.cantidad) > 0
            ORDER BY p.nombre, p.id"#,
    )
    .fetch_all(pool)
    .await?;
    let mut total = Decimal::ZERO;
    let items = filas
        .into_iter()
        .map(|(producto_id, nombre, sku, cantidad, costo)| {
            let valor = costo * Decimal::from(cantidad);
            total += valor;
            StockValorizadoItem {
                producto_id,
                nombre,
                sku,
                cantidad,
                costo_unitario: costo,
                valor,
            }
        })
        .collect();
    Ok(StockValorizadoReporte { items, total })
}

/// FR-058, R3: una sola consulta agregada; salidas `SALIDA` sin `TRANSFERENCIA`.
pub async fn rotacion(
    pool: &PgPool,
    desde: Option<NaiveDate>,
    hasta: Option<NaiveDate>,
) -> Result<RotacionReporte, AppError> {
    let (desde, hasta) = rango::exigir_rango(desde, hasta)?;
    let (inicio, fin) = rango::limites_utc(Some(desde), Some(hasta))?;
    let filas: Vec<(i32, String, i32, i32, i32)> = sqlx::query_as(
        r#"SELECT ml.producto_id, p.nombre,
                  COALESCE(SUM(-ml.cantidad_delta) FILTER (
                      WHERE m.creado_en >= $1 AND m.creado_en < $2
                        AND m.tipo = 'SALIDA' AND m.motivo <> 'TRANSFERENCIA'), 0)::int AS salidas,
                  COALESCE(SUM(ml.cantidad_delta) FILTER (WHERE m.creado_en < $1), 0)::int AS inicial,
                  COALESCE(SUM(ml.cantidad_delta) FILTER (
                      WHERE m.creado_en >= $1 AND m.creado_en < $2), 0)::int AS neto
             FROM movimiento_lineas ml
             JOIN movimientos m ON m.id = ml.movimiento_id
             JOIN productos p ON p.id = ml.producto_id
            WHERE m.creado_en < $2
            GROUP BY ml.producto_id, p.nombre
            ORDER BY p.nombre, ml.producto_id"#,
    )
    .bind(inicio)
    .bind(fin)
    .fetch_all(pool)
    .await?;
    let items = filas
        .into_iter()
        .filter_map(|(producto_id, nombre, unidades_salidas, stock_inicial, neto)| {
            let stock_final = stock_inicial.saturating_add(neto);
            if unidades_salidas == 0 && stock_inicial <= 0 && stock_final <= 0 {
                return None;
            }
            let (stock_promedio, indice_rotacion) =
                calculo::calcular_rotacion(unidades_salidas, stock_inicial, stock_final);
            Some(RotacionItem {
                producto_id,
                nombre,
                unidades_salidas,
                stock_inicial,
                stock_final,
                stock_promedio,
                indice_rotacion,
            })
        })
        .collect();
    Ok(RotacionReporte { desde, hasta, items })
}

/// FR-058, R6: ventas `CONFIRMADA` del período, por día local y por producto.
pub async fn ventas(
    pool: &PgPool,
    desde: Option<NaiveDate>,
    hasta: Option<NaiveDate>,
) -> Result<VentasReporte, AppError> {
    let (desde, hasta) = rango::exigir_rango(desde, hasta)?;
    let (inicio, fin) = rango::limites_utc(Some(desde), Some(hasta))?;

    let (cantidad_ventas, total): (i64, Decimal) = sqlx::query_as(
        r#"SELECT count(*)::bigint, COALESCE(SUM(total), 0)
             FROM ventas
            WHERE estado = 'CONFIRMADA' AND creado_en >= $1 AND creado_en < $2"#,
    )
    .bind(inicio)
    .bind(fin)
    .fetch_one(pool)
    .await?;

    let por_dia = sqlx::query_as::<_, (String, i64, Decimal)>(
        r#"SELECT to_char(creado_en AT TIME ZONE 'UTC' - INTERVAL '3 hours', 'YYYY-MM-DD') AS dia,
                  count(*)::bigint, COALESCE(SUM(total), 0)
             FROM ventas
            WHERE estado = 'CONFIRMADA' AND creado_en >= $1 AND creado_en < $2
            GROUP BY 1
            ORDER BY 1"#,
    )
    .bind(inicio)
    .bind(fin)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(dia, cantidad_ventas, total)| VentaResumenDia {
        dia,
        cantidad_ventas,
        total,
    })
    .collect();

    let por_producto = sqlx::query_as::<_, (i32, String, i32, Decimal)>(
        r#"SELECT vl.producto_id, p.nombre, COALESCE(SUM(vl.cantidad), 0)::int, COALESCE(SUM(vl.subtotal), 0)
             FROM venta_lineas vl
             JOIN ventas v ON v.id = vl.venta_id
             JOIN productos p ON p.id = vl.producto_id
            WHERE v.estado = 'CONFIRMADA' AND v.creado_en >= $1 AND v.creado_en < $2
            GROUP BY vl.producto_id, p.nombre
            ORDER BY 4 DESC, p.nombre"#,
    )
    .bind(inicio)
    .bind(fin)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(producto_id, nombre, unidades, total)| VentaResumenProducto {
        producto_id,
        nombre,
        unidades,
        total,
    })
    .collect();

    Ok(VentasReporte {
        desde,
        hasta,
        cantidad_ventas,
        total,
        por_dia,
        por_producto,
    })
}
