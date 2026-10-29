//! Handlers de consulta de stock (007, FR-029).
//!
//! Solo lectura: cualquier sesión vigente puede consultar. El listado usa el cache
//! `productos.stock_actual`; el detalle agrega el desglose real por depósito y ubicación.

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{PathValido, QueryValida};
use crate::models::stock::{ListaStock, StockProducto};
use crate::services::stock::{self, FiltrosStock};
use axum::{extract::State, routing::get, Json, Router};
use serde::Deserialize;
use utoipa::IntoParams;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar))
        .route("/{producto_id}", get(detalle))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarStockQuery {
    /// Subcadena sobre nombre o SKU.
    pub q: Option<String>,
    /// Agrega `stock_deposito` a cada item.
    pub deposito_id: Option<i32>,
    pub categoria_id: Option<i32>,
    /// Solo productos por debajo de su stock mínimo.
    pub bajo_minimo: Option<bool>,
    /// `true`, `false` o `all` (ausente = solo activos).
    pub activo: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// GET /api/stock?q=&deposito_id=&categoria_id=&bajo_minimo=&activo=&pagina=&por_pagina=
#[utoipa::path(
    get,
    path = "/api/stock",
    tag = "stock",
    summary = "Listado de stock por producto",
    description = "`stock_actual` es el total del producto; con `deposito_id` se agrega `stock_deposito`.",
    params(ListarStockQuery),
    responses(
        (status = 200, description = "Página de stock", body = ListaStock),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 422, description = "Filtros inválidos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _usuario: AuthUsuario,
    QueryValida(query): QueryValida<ListarStockQuery>,
) -> Result<Json<ListaStock>, AppError> {
    let filtros = FiltrosStock {
        q: query.q,
        deposito_id: query.deposito_id,
        categoria_id: query.categoria_id,
        bajo_minimo: query.bajo_minimo.unwrap_or(false),
        activo: super::parsear_activo(query.activo.as_deref(), Some(true))?,
        pagina: query.pagina,
        por_pagina: query.por_pagina,
    };
    Ok(Json(stock::listar(&estado.pool, filtros).await?))
}

/// GET /api/stock/{producto_id} → desglose por depósito y ubicación.
#[utoipa::path(
    get,
    path = "/api/stock/{producto_id}",
    tag = "stock",
    summary = "Detalle de stock de un producto",
    params(("producto_id" = i32, Path, description = "Identificador del producto")),
    responses(
        (status = 200, description = "Stock y desglose", body = StockProducto),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta)
    )
)]
pub async fn detalle(
    State(estado): State<AppEstado>,
    _usuario: AuthUsuario,
    PathValido(producto_id): PathValido<i32>,
) -> Result<Json<StockProducto>, AppError> {
    Ok(Json(stock::detalle(&estado.pool, producto_id).await?))
}
