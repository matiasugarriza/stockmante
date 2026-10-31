//! Reportes (014): kardex, stock valorizado, rotación, ventas.

use crate::error::AppError;
use crate::services::normalizar_paginacion;
use chrono::{DateTime, Utc};
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
    pub saldo_inicial: i32,
    pub lineas: Vec<KardexLinea>,
    pub saldo_final: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StockValorizadoItem {
    pub producto_id: i32,
    pub nombre: String,
    pub sku: String,
    pub cantidad: i32,
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
    pub desde: DateTime<Utc>,
    pub hasta: DateTime<Utc>,
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
    pub desde: DateTime<Utc>,
    pub hasta: DateTime<Utc>,
    pub cantidad_ventas: i64,
    #[schema(value_type = f64)]
    pub total: Decimal,
    pub por_dia: Vec<VentaResumenDia>,
    pub por_producto: Vec<VentaResumenProducto>,
}

fn validar_rango(desde: Option<DateTime<Utc>>, hasta: Option<DateTime<Utc>>) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>), AppError> {
    if let (Some(d), Some(h)) = (desde, hasta) {
        if d > h { return Err(AppError::Validacion("La fecha desde no puede ser posterior a la fecha hasta.".into())); }
    }
    Ok((desde, hasta))
}

pub async fn kardex(pool: &PgPool, producto_id: i32, deposito_id: Option<i32>, desde: Option<DateTime<Utc>>, hasta: Option<DateTime<Utc>>) -> Result<KardexReporte, AppError> {
    let (desde, hasta) = validar_rango(desde, hasta)?;
    let producto_nombre: String = sqlx::query_scalar("SELECT nombre FROM productos WHERE id=$1")
        .bind(producto_id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::RecursoNoEncontrado("El producto no existe.".into()))?;

    // saldo inicial: sum of deltas before desde
    let saldo_inicial: i32 = if let Some(d) = desde {
        sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(ml.cantidad_delta),0)::int
                 FROM movimiento_lineas ml
                 JOIN movimientos m ON m.id = ml.movimiento_id
                WHERE ml.producto_id = $1
                  AND ($2::int IS NULL OR m.deposito_id = $2)
                  AND m.creado_en < $3"#,
        )
        .bind(producto_id).bind(deposito_id).bind(d)
        .fetch_one(pool).await?
    } else { 0 };

    // movements in range
    let filas: Vec<(i32, DateTime<Utc>, String, String, i32, String, i32)> = sqlx::query_as(
        r#"SELECT m.id, m.creado_en, m.tipo, m.motivo, m.deposito_id, d.nombre AS deposito_nombre, ml.cantidad_delta
             FROM movimiento_lineas ml
             JOIN movimientos m ON m.id = ml.movimiento_id
             JOIN depositos d ON d.id = m.deposito_id
            WHERE ml.producto_id = $1
              AND ($2::int IS NULL OR m.deposito_id = $2)
              AND ($3::timestamptz IS NULL OR m.creado_en >= $3)
              AND ($4::timestamptz IS NULL OR m.creado_en <= $4)
            ORDER BY m.creado_en ASC, m.id ASC, ml.id ASC"#,
    )
    .bind(producto_id).bind(deposito_id).bind(desde).bind(hasta)
    .fetch_all(pool).await?;

    let mut saldo = saldo_inicial;
    let mut lineas = Vec::new();
    for (mov_id, creado, tipo, motivo, dep_id, dep_nombre, cant) in filas {
        saldo += cant;
        lineas.push(KardexLinea {
            movimiento_id: mov_id, creado_en: creado, tipo, motivo,
            deposito_id: dep_id, deposito_nombre: dep_nombre, cantidad: cant, saldo,
        });
    }
    let saldo_final = saldo;
    Ok(KardexReporte { producto_id, producto_nombre, saldo_inicial, lineas, saldo_final })
}

pub async fn stock_valorizado(pool: &PgPool) -> Result<StockValorizadoReporte, AppError> {
    let filas: Vec<(i32, String, String, i32, Decimal)> = sqlx::query_as(
        r#"SELECT p.id, p.nombre, p.sku, p.stock_actual, p.costo
             FROM productos p
            WHERE p.activo = TRUE
            ORDER BY p.nombre"#,
    )
    .fetch_all(pool).await?;
    let mut items = Vec::new();
    let mut total = Decimal::ZERO;
    for (id, nombre, sku, cant, costo) in filas {
        let valor = costo * Decimal::from(cant);
        total += valor;
        items.push(StockValorizadoItem { producto_id: id, nombre, sku, cantidad: cant, costo_unitario: costo, valor });
    }
    Ok(StockValorizadoReporte { items, total })
}

pub async fn rotacion(pool: &PgPool, desde: Option<DateTime<Utc>>, hasta: Option<DateTime<Utc>>) -> Result<RotacionReporte, AppError> {
    let (desde, hasta) = validar_rango(desde, hasta)?;
    let d = desde.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let h = hasta.unwrap_or_else(Utc::now);

    // For each active product, compute unidades_salidas in range and stock inicial/final
    let productos: Vec<(i32, String)> = sqlx::query_as("SELECT id, nombre FROM productos WHERE activo=TRUE ORDER BY nombre")
        .fetch_all(pool).await?;

    let mut items = Vec::new();
    for (pid, nombre) in productos {
        let salidas: i32 = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(ml.cantidad_delta * -1),0)::int
                 FROM movimiento_lineas ml
                 JOIN movimientos m ON m.id = ml.movimiento_id
                WHERE ml.producto_id = $1
                  AND m.tipo = 'SALIDA'
                  AND m.creado_en >= $2 AND m.creado_en <= $3"#,
        )
        .bind(pid).bind(d).bind(h).fetch_one(pool).await?;

        let stock_inicial: i32 = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(ml.cantidad_delta),0)::int
                 FROM movimiento_lineas ml
                 JOIN movimientos m ON m.id = ml.movimiento_id
                WHERE ml.producto_id = $1
                  AND m.creado_en < $2"#,
        )
        .bind(pid).bind(d).fetch_one(pool).await?;

        let stock_final: i32 = stock_inicial - salidas; // approximate; adjust with entradas in range
        // recalc stock_final as inicial + net deltas in range
        let neto: i32 = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(ml.cantidad_delta),0)::int
                 FROM movimiento_lineas ml
                 JOIN movimientos m ON m.id = ml.movimiento_id
                WHERE ml.producto_id = $1
                  AND m.creado_en >= $2 AND m.creado_en <= $3"#,
        )
        .bind(pid).bind(d).bind(h).fetch_one(pool).await?;
        let stock_final = stock_inicial + neto;

        let promedio = (stock_inicial as f64 + stock_final as f64) / 2.0;
        let indice = if promedio > 0.0 { salidas as f64 / promedio } else { 0.0 };
        if salidas > 0 || stock_inicial > 0 || stock_final > 0 {
            items.push(RotacionItem {
                producto_id: pid, nombre, unidades_salidas: salidas,
                stock_inicial, stock_final, stock_promedio: promedio, indice_rotacion: indice,
            });
        }
    }
    Ok(RotacionReporte { desde: d, hasta: h, items })
}

pub async fn ventas(pool: &PgPool, desde: Option<DateTime<Utc>>, hasta: Option<DateTime<Utc>>) -> Result<VentasReporte, AppError> {
    let (desde, hasta) = validar_rango(desde, hasta)?;
    let d = desde.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let h = hasta.unwrap_or_else(Utc::now);

    let (cantidad_ventas, total): (i64, Decimal) = sqlx::query_as(
        "SELECT count(*)::bigint, COALESCE(SUM(total),0) FROM ventas WHERE estado='CONFIRMADA' AND creado_en >= $1 AND creado_en <= $2",
    )
    .bind(d).bind(h).fetch_one(pool).await?;

    let por_dia_rows: Vec<(String, i64, Decimal)> = sqlx::query_as(
        r#"SELECT to_char(creado_en, 'YYYY-MM-DD') AS dia, count(*)::bigint, COALESCE(SUM(total),0)
             FROM ventas
            WHERE estado='CONFIRMADA' AND creado_en >= $1 AND creado_en <= $2
            GROUP BY 1 ORDER BY 1"#,
    )
    .bind(d).bind(h).fetch_all(pool).await?;

    let por_dia = por_dia_rows.into_iter().map(|(dia, c, t)| VentaResumenDia { dia, cantidad_ventas: c, total: t }).collect();

    let por_producto_rows: Vec<(i32, String, i32, Decimal)> = sqlx::query_as(
        r#"SELECT vl.producto_id, p.nombre, COALESCE(SUM(vl.cantidad),0)::int, COALESCE(SUM(vl.subtotal),0)
             FROM venta_lineas vl
             JOIN ventas v ON v.id = vl.venta_id
             JOIN productos p ON p.id = vl.producto_id
            WHERE v.estado='CONFIRMADA' AND v.creado_en >= $1 AND v.creado_en <= $2
            GROUP BY vl.producto_id, p.nombre
            ORDER BY 4 DESC"#,
    )
    .bind(d).bind(h).fetch_all(pool).await?;

    let por_producto = por_producto_rows.into_iter().map(|(pid, nombre, u, t)| VentaResumenProducto { producto_id: pid, nombre, unidades: u, total: t }).collect();

    Ok(VentasReporte { desde: d, hasta: h, cantidad_ventas, total, por_dia, por_producto })
}
