//! Handlers de productos (003, contracts/api.md).
//!
//! Listado, detalle, alta y edición. Escritura: familia GESTION; el listado de
//! gestión también (es la pantalla de administración); el detalle lo puede leer
//! cualquier sesión válida.
//! No se registra `DELETE`: Axum responde 405 con `Allow: GET, PUT`.

use crate::app::AppEstado;
use crate::auth::extractors::{AuthUsuario, RequiereGestion};
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::producto::{ListaProductos, Producto};
use crate::services::productos::{
    self, CambiosProducto, FiltrosProductos, NuevoProducto,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/{id}", get(obtener).put(actualizar))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
}

#[derive(Debug, Deserialize)]
pub struct ListarQuery {
    pub q: Option<String>,
    pub categoria_id: Option<i32>,
    pub activo: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// GET /api/productos?q=&categoria_id=&activo=&pagina=&por_pagina=
/// (default: solo activos, orden por nombre).
async fn listar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(query): QueryValida<ListarQuery>,
) -> Result<Json<ListaProductos>, AppError> {
    let filtros = FiltrosProductos {
        q: query.q,
        categoria_id: query.categoria_id,
        activo: super::parsear_activo(query.activo.as_deref(), Some(true))?,
        pagina: query.pagina,
        por_pagina: query.por_pagina,
    };
    Ok(Json(productos::listar(&estado.pool, filtros).await?))
}

#[derive(Debug, Deserialize)]
pub struct CrearProductoDto {
    pub nombre: String,
    pub sku: String,
    #[serde(default)]
    pub codigo_barras: Option<String>,
    pub categoria_id: i32,
    pub unidad_medida: String,
    pub costo: rust_decimal::Decimal,
    pub precio_venta: rust_decimal::Decimal,
    #[serde(default)]
    pub stock_minimo: Option<i32>,
}

/// POST /api/productos → 201 producto.
async fn crear(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    JsonValido(dto): JsonValido<CrearProductoDto>,
) -> Result<(StatusCode, Json<Producto>), AppError> {
    let nuevo = NuevoProducto {
        nombre: dto.nombre,
        sku: dto.sku,
        codigo_barras: dto.codigo_barras,
        categoria_id: dto.categoria_id,
        unidad_medida: dto.unidad_medida,
        costo: dto.costo,
        precio_venta: dto.precio_venta,
        stock_minimo: dto.stock_minimo,
    };
    let producto = productos::crear(&estado.pool, nuevo).await?;
    Ok((StatusCode::CREATED, Json(producto)))
}

/// GET /api/productos/{id} → 200 o 404 (también para inactivos).
async fn obtener(
    State(estado): State<AppEstado>,
    _sesion: AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Producto>, AppError> {
    Ok(Json(productos::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct ActualizarProductoDto {
    #[serde(default)]
    pub nombre: Option<String>,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default, deserialize_with = "super::deserializar_presente")]
    pub codigo_barras: Option<Option<String>>,
    #[serde(default)]
    pub categoria_id: Option<i32>,
    #[serde(default)]
    pub unidad_medida: Option<String>,
    #[serde(default)]
    pub costo: Option<rust_decimal::Decimal>,
    #[serde(default)]
    pub precio_venta: Option<rust_decimal::Decimal>,
    #[serde(default)]
    pub stock_minimo: Option<i32>,
    /// `activo` no viaja por `PUT`: su sola presencia es 422.
    #[serde(default)]
    pub activo: Option<serde_json::Value>,
}

/// PUT /api/productos/{id} → 200 producto actualizado.
async fn actualizar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ActualizarProductoDto>,
) -> Result<Json<Producto>, AppError> {
    if dto.activo.is_some() {
        return Err(AppError::Validacion(
            "El estado se cambia con /desactivar o /reactivar.".into(),
        ));
    }
    let cambios = CambiosProducto {
        nombre: dto.nombre,
        sku: dto.sku,
        codigo_barras: dto.codigo_barras,
        categoria_id: dto.categoria_id,
        unidad_medida: dto.unidad_medida,
        costo: dto.costo,
        precio_venta: dto.precio_venta,
        stock_minimo: dto.stock_minimo,
    };
    Ok(Json(productos::actualizar(&estado.pool, id, cambios).await?))
}

/// POST /api/productos/{id}/desactivar → 204 (idempotente).
async fn desactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    productos::desactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/productos/{id}/reactivar → 204 (idempotente).
async fn reactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    productos::reactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
