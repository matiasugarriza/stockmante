//! Consulta de stock (007, FR-029).
//!
//! `productos.stock_actual` es el cache global y aquí es lo que se lista. El desglose
//! sale de `stock_por_deposito` (fuente de verdad) y siempre suma exactamente el cache:
//! esa es la invariante que verifican los tests (SC-014).

use crate::error::AppError;
use crate::models::stock::{
    DesgloseStock, ListaStock, StockFila, StockItem, StockProducto,
};
use crate::services::normalizar_paginacion;
use sqlx::PgPool;

/// Filtros del listado de stock.
#[derive(Debug, Clone, Default)]
pub struct FiltrosStock {
    /// Subcadena sobre nombre o SKU.
    pub q: Option<String>,
    /// Agrega la columna `stock_deposito` y opcionalmente restringe a ese depósito.
    pub deposito_id: Option<i32>,
    pub categoria_id: Option<i32>,
    /// Solo productos con `stock_actual < stock_minimo` (y mínimo definido).
    pub bajo_minimo: bool,
    /// `None` = todos; el handler decide el default (solo activos).
    pub activo: Option<bool>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// Fila del detalle de producto (cabecera del desglose).
#[derive(Debug, sqlx::FromRow)]
struct ProductoStockFila {
    producto_id: i32,
    nombre: String,
    sku: String,
    unidad_medida: String,
    activo: bool,
    stock_actual: i64,
    stock_minimo: i32,
}

/// Listado paginado de stock por producto.
///
/// Con `deposito_id` el `WHERE` no descarta productos sin existencias en ese depósito:
/// devuelve el catálogo filtrado y `stock_deposito` en cero, que es lo que necesita la
/// pantalla para detectar faltantes.
pub async fn listar(pool: &PgPool, filtros: FiltrosStock) -> Result<ListaStock, AppError> {
    let (pagina, por_pagina, offset) =
        normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;

    let q = filtros
        .q
        .as_deref()
        .map(str::trim)
        .filter(|texto| !texto.is_empty())
        .map(str::to_owned);

    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*)
             FROM productos p
            WHERE ($1::text IS NULL OR p.nombre ILIKE '%' || $1 || '%' OR p.sku ILIKE '%' || $1 || '%')
              AND ($2::int IS NULL OR p.categoria_id = $2)
              AND ($3::bool IS NULL OR p.activo = $3)
              AND ($4::bool IS NOT TRUE
                   OR (p.stock_minimo > 0 AND p.stock_actual < p.stock_minimo))"#,
    )
    .bind(q.as_deref())
    .bind(filtros.categoria_id)
    .bind(filtros.activo)
    .bind(filtros.bajo_minimo)
    .fetch_one(pool)
    .await?;

    let filas = sqlx::query_as::<_, StockFila>(
        r#"SELECT p.id AS producto_id, p.nombre, p.sku,
                  c.id AS categoria_id, c.nombre AS categoria_nombre,
                  p.unidad_medida, p.activo,
                  p.stock_actual::bigint AS stock_actual,
                  p.stock_minimo,
                  CASE WHEN $5::int IS NULL THEN NULL
                       ELSE COALESCE(
                           (SELECT SUM(s.cantidad) FROM stock_por_deposito s
                             WHERE s.producto_id = p.id AND s.deposito_id = $5),
                           0
                       )::bigint
                  END AS stock_deposito
             FROM productos p
             JOIN categorias c ON c.id = p.categoria_id
            WHERE ($1::text IS NULL OR p.nombre ILIKE '%' || $1 || '%' OR p.sku ILIKE '%' || $1 || '%')
              AND ($2::int IS NULL OR p.categoria_id = $2)
              AND ($3::bool IS NULL OR p.activo = $3)
              AND ($4::bool IS NOT TRUE
                   OR (p.stock_minimo > 0 AND p.stock_actual < p.stock_minimo))
            ORDER BY p.nombre, p.id
            LIMIT $6 OFFSET $7"#,
    )
    .bind(q.as_deref())
    .bind(filtros.categoria_id)
    .bind(filtros.activo)
    .bind(filtros.bajo_minimo)
    .bind(filtros.deposito_id)
    .bind(por_pagina)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let items = filas.into_iter().map(StockItem::from).collect();

    Ok(ListaStock {
        items,
        total,
        pagina,
        por_pagina,
    })
}

/// Detalle de stock de un producto con el desglose por depósito y ubicación.
pub async fn detalle(pool: &PgPool, producto_id: i32) -> Result<StockProducto, AppError> {
    let producto = sqlx::query_as::<_, ProductoStockFila>(
        r#"SELECT p.id AS producto_id, p.nombre, p.sku, p.unidad_medida, p.activo,
                  p.stock_actual::bigint AS stock_actual, p.stock_minimo
             FROM productos p
            WHERE p.id = $1"#,
    )
    .bind(producto_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::RecursoNoEncontrado("Producto no encontrado.".into()))?;

    let desglose = sqlx::query_as::<_, DesgloseStock>(
        r#"SELECT s.deposito_id, d.nombre AS deposito_nombre,
                  s.ubicacion_id, u.nombre AS ubicacion_nombre,
                  s.cantidad::bigint AS cantidad
             FROM stock_por_deposito s
             JOIN depositos d ON d.id = s.deposito_id
             LEFT JOIN ubicaciones u ON u.id = s.ubicacion_id
            WHERE s.producto_id = $1
            ORDER BY d.nombre, u.nombre NULLS FIRST"#,
    )
    .bind(producto_id)
    .fetch_all(pool)
    .await?;

    let bajo_minimo =
        producto.stock_minimo > 0 && producto.stock_actual < i64::from(producto.stock_minimo);

    Ok(StockProducto {
        producto_id: producto.producto_id,
        nombre: producto.nombre,
        sku: producto.sku,
        unidad_medida: producto.unidad_medida,
        activo: producto.activo,
        stock_actual: producto.stock_actual,
        stock_minimo: producto.stock_minimo,
        bajo_minimo,
        desglose,
    })
}
