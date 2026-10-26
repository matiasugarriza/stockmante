//! Handlers de productos (003, contracts/api.md).
//!
//! Listado, detalle, alta y edición. Escritura: familia GESTION; el listado de
//! gestión también (es la pantalla de administración); el detalle lo puede leer
//! cualquier sesión válida.
//! No se registra `DELETE`: Axum responde 405 con `Allow: GET, PUT`.

use crate::app::AppEstado;
use crate::auth::extractors::{AuthUsuario, RequiereGestion};
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::deposito::ListaUbicaciones;
use crate::models::producto::{ListaBusqueda, ListaProductos, Producto};
use crate::services::depositos;
use crate::services::productos::{
    self, BusquedaProductos, CampoBusqueda, CambiosProducto, FiltrosProductos, NuevoProducto,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/buscar", get(buscar))
        .route("/{id}", get(obtener).put(actualizar))
        .route(
            "/{id}/ubicaciones",
            get(ubicaciones).put(reemplazar_ubicaciones),
        )
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarQuery {
    /// Subcadena sobre nombre o SKU.
    pub q: Option<String>,
    /// Filtra por categoría.
    pub categoria_id: Option<i32>,
    /// `true`, `false` o `all` (ausente = solo activos).
    pub activo: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// GET /api/productos?q=&categoria_id=&activo=&pagina=&por_pagina=
/// (default: solo activos, orden por nombre).
#[utoipa::path(
    get,
    path = "/api/productos",
    tag = "productos",
    summary = "Listado de gestión del catálogo (GESTION)",
    params(ListarQuery),
    responses(
        (status = 200, description = "Página de productos", body = ListaProductos),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Filtros o paginación inválidos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
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

#[derive(Debug, Deserialize, IntoParams)]
pub struct BuscarQuery {
    /// Término de búsqueda (1–160). Puede faltar si viene `categoria_id`.
    pub q: Option<String>,
    /// `sku`, `codigo`, `nombre`, `categoria` o `todos` (default).
    pub campo: Option<String>,
    /// `true` solo para familia GESTION.
    pub incluir_inactivos: Option<bool>,
    /// Filtra por categoría; también permite listar sin `q`.
    pub categoria_id: Option<i32>,
    /// Cantidad máxima de resultados (1–50, default 20).
    pub limite: Option<i64>,
}

/// GET /api/productos/buscar?q=&campo=&incluir_inactivos=&categoria_id=&limite=
/// Búsqueda de caja: cualquier sesión; inactivos solo GESTION.
#[utoipa::path(
    get,
    path = "/api/productos/buscar",
    tag = "productos",
    summary = "Búsqueda rápida de caja y almacén",
    description = "Exacta por SKU/código y por trigramas para nombre/categoría. Excluye inactivos salvo `incluir_inactivos=true` (GESTION).",
    params(BuscarQuery),
    responses(
        (status = 200, description = "Coincidencias ordenadas", body = ListaBusqueda),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Inactivos requieren GESTION", body = ErrorRespuesta),
        (status = 422, description = "Parámetros inválidos", body = ErrorRespuesta)
    )
)]
pub async fn buscar(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    QueryValida(query): QueryValida<BuscarQuery>,
) -> Result<Json<ListaBusqueda>, AppError> {
    let incluir_inactivos = query.incluir_inactivos.unwrap_or(false);
    if incluir_inactivos && usuario.familia_rol != "GESTION" {
        return Err(AppError::SinPermisos);
    }
    let busqueda = BusquedaProductos {
        q: query.q.unwrap_or_default(),
        campo: CampoBusqueda::parsear(query.campo.as_deref())?,
        incluir_inactivos,
        categoria_id: query.categoria_id,
        limite: query.limite,
    };
    Ok(Json(productos::buscar(&estado.pool, busqueda).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearProductoDto {
    #[schema(example = "Yerba 1 kg")]
    pub nombre: String,
    #[schema(example = "YERBA-1K")]
    pub sku: String,
    #[serde(default)]
    pub codigo_barras: Option<String>,
    pub categoria_id: i32,
    #[schema(example = "UNIDAD")]
    pub unidad_medida: String,
    #[schema(value_type = f64, example = 1200.5)]
    pub costo: rust_decimal::Decimal,
    #[schema(value_type = f64, example = 1800.0)]
    pub precio_venta: rust_decimal::Decimal,
    #[serde(default)]
    pub stock_minimo: Option<i32>,
}

/// POST /api/productos → 201 producto.
#[utoipa::path(
    post,
    path = "/api/productos",
    tag = "productos",
    summary = "Alta de producto (GESTION)",
    request_body = CrearProductoDto,
    responses(
        (status = 201, description = "Producto creado", body = Producto),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 409, description = "SKU o código de barras duplicado", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos o categoría inexistente/inactiva", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    gestion: RequiereGestion,
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
    let producto = productos::crear(&estado.pool, nuevo, gestion.0.id).await?;
    Ok((StatusCode::CREATED, Json(producto)))
}

/// GET /api/productos/{id} → 200 o 404 (también para inactivos).
#[utoipa::path(
    get,
    path = "/api/productos/{id}",
    tag = "productos",
    summary = "Detalle de producto",
    params(("id" = i32, Path, description = "Id del producto")),
    responses(
        (status = 200, description = "Producto (también inactivo)", body = Producto),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _sesion: AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Producto>, AppError> {
    Ok(Json(productos::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizarProductoDto {
    #[serde(default)]
    pub nombre: Option<String>,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default, deserialize_with = "super::deserializar_presente")]
    #[schema(value_type = Option<String>)]
    pub codigo_barras: Option<Option<String>>,
    #[serde(default)]
    pub categoria_id: Option<i32>,
    #[serde(default)]
    pub unidad_medida: Option<String>,
    #[serde(default)]
    #[schema(value_type = Option<f64>)]
    pub costo: Option<rust_decimal::Decimal>,
    #[serde(default)]
    #[schema(value_type = Option<f64>)]
    pub precio_venta: Option<rust_decimal::Decimal>,
    #[serde(default)]
    pub stock_minimo: Option<i32>,
    /// `activo` no viaja por `PUT`: su sola presencia es 422.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub activo: Option<serde_json::Value>,
}

/// PUT /api/productos/{id} → 200 producto actualizado.
#[utoipa::path(
    put,
    path = "/api/productos/{id}",
    tag = "productos",
    summary = "Edición parcial de producto (GESTION)",
    description = "Omitir un campo lo conserva; `codigo_barras: \"\"` lo limpia. `activo` en el cuerpo es 422.",
    params(("id" = i32, Path, description = "Id del producto")),
    request_body = ActualizarProductoDto,
    responses(
        (status = 200, description = "Producto actualizado", body = Producto),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta),
        (status = 409, description = "SKU o código de barras duplicado", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos", body = ErrorRespuesta)
    )
)]
pub async fn actualizar(
    State(estado): State<AppEstado>,
    gestion: RequiereGestion,
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
    Ok(Json(
        productos::actualizar(&estado.pool, id, cambios, gestion.0.id).await?,
    ))
}

/// POST /api/productos/{id}/desactivar → 204 (idempotente).
#[utoipa::path(
    post,
    path = "/api/productos/{id}/desactivar",
    tag = "productos",
    summary = "Baja lógica de producto (GESTION)",
    params(("id" = i32, Path, description = "Id del producto")),
    responses(
        (status = 204, description = "Producto desactivado (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta)
    )
)]
pub async fn desactivar(
    State(estado): State<AppEstado>,
    gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    productos::desactivar(&estado.pool, id, gestion.0.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/productos/{id}/reactivar → 204 (idempotente).
#[utoipa::path(
    post,
    path = "/api/productos/{id}/reactivar",
    tag = "productos",
    summary = "Reactiva un producto (GESTION)",
    params(("id" = i32, Path, description = "Id del producto")),
    responses(
        (status = 204, description = "Producto reactivado (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta)
    )
)]
pub async fn reactivar(
    State(estado): State<AppEstado>,
    gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    productos::reactivar(&estado.pool, id, gestion.0.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AsignarUbicacionesDto {
    /// Conjunto completo; `[]` limpia las ubicaciones permitidas.
    pub ubicacion_ids: Vec<i32>,
}

/// GET /api/productos/{id}/ubicaciones → ubicaciones permitidas.
#[utoipa::path(
    get,
    path = "/api/productos/{id}/ubicaciones",
    tag = "productos",
    summary = "Ubicaciones permitidas de un producto (GESTION)",
    params(("id" = i32, Path, description = "Id del producto")),
    responses(
        (status = 200, description = "Ubicaciones asignadas", body = ListaUbicaciones),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta)
    )
)]
pub async fn ubicaciones(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<ListaUbicaciones>, AppError> {
    let items = depositos::ubicaciones_de_producto(&estado.pool, id).await?;
    Ok(Json(ListaUbicaciones { items }))
}

/// PUT /api/productos/{id}/ubicaciones → 204 (reemplaza el conjunto).
#[utoipa::path(
    put,
    path = "/api/productos/{id}/ubicaciones",
    tag = "productos",
    summary = "Asigna ubicaciones permitidas a un producto (GESTION)",
    description = "Reemplaza el conjunto completo; las ubicaciones deben existir con depósito activo.",
    params(("id" = i32, Path, description = "Id del producto")),
    request_body = AsignarUbicacionesDto,
    responses(
        (status = 204, description = "Asignaciones reemplazadas"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Producto o ubicación inexistente/inactiva", body = ErrorRespuesta),
        (status = 422, description = "Ids inválidos", body = ErrorRespuesta)
    )
)]
pub async fn reemplazar_ubicaciones(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<AsignarUbicacionesDto>,
) -> Result<StatusCode, AppError> {
    depositos::reemplazar_ubicaciones_producto(&estado.pool, id, dto.ubicacion_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}
