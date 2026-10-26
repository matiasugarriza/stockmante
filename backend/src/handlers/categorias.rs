//! Handlers de categorías (003, contracts/api.md).
//!
//! Lectura: cualquier sesión válida. Escritura: familia GESTION.
//! No se registra `DELETE`: Axum responde 405 con `Allow: GET, PUT`.

use crate::app::AppEstado;
use crate::auth::extractors::{AuthUsuario, RequiereGestion};
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::producto::Categoria;
use crate::services::categorias::{self, CambiosCategoria};
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
        .route("/{id}", get(obtener).put(actualizar))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarQuery {
    /// `true`, `false` o `all` (ausente = solo activas).
    pub activo: Option<String>,
}

/// GET /api/categorias?activo=true|false|all (default: solo activas).
#[utoipa::path(
    get,
    path = "/api/categorias",
    tag = "categorias",
    summary = "Lista categorías",
    params(ListarQuery),
    responses(
        (status = 200, description = "Categorías ordenadas por nombre", body = Vec<Categoria>),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 422, description = "Filtro inválido", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _sesion: AuthUsuario,
    QueryValida(query): QueryValida<ListarQuery>,
) -> Result<Json<Vec<Categoria>>, AppError> {
    let activo = super::parsear_activo(query.activo.as_deref(), Some(true))?;
    Ok(Json(categorias::listar(&estado.pool, activo).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearCategoriaDto {
    #[schema(example = "Almacén")]
    pub nombre: String,
    pub descripcion: Option<String>,
}

/// POST /api/categorias → 201 categoría.
#[utoipa::path(
    post,
    path = "/api/categorias",
    tag = "categorias",
    summary = "Alta de categoría (GESTION)",
    request_body = CrearCategoriaDto,
    responses(
        (status = 201, description = "Categoría creada", body = Categoria),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 409, description = "Nombre duplicado", body = ErrorRespuesta),
        (status = 422, description = "Nombre inválido", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    JsonValido(dto): JsonValido<CrearCategoriaDto>,
) -> Result<(StatusCode, Json<Categoria>), AppError> {
    let categoria =
        categorias::crear(&estado.pool, &dto.nombre, dto.descripcion.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(categoria)))
}

/// GET /api/categorias/{id} → 200 o 404 (también para inactivas).
#[utoipa::path(
    get,
    path = "/api/categorias/{id}",
    tag = "categorias",
    summary = "Detalle de categoría",
    params(("id" = i32, Path, description = "Id de la categoría")),
    responses(
        (status = 200, description = "Categoría (también inactiva)", body = Categoria),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 404, description = "Categoría inexistente", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _sesion: AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Categoria>, AppError> {
    Ok(Json(categorias::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizarCategoriaDto {
    #[serde(default)]
    pub nombre: Option<String>,
    #[serde(default, deserialize_with = "super::deserializar_presente")]
    #[schema(value_type = Option<String>)]
    pub descripcion: Option<Option<String>>,
    /// `activo` no viaja por `PUT`: su sola presencia es 422.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub activo: Option<serde_json::Value>,
}

/// PUT /api/categorias/{id} → 200 categoría actualizada.
#[utoipa::path(
    put,
    path = "/api/categorias/{id}",
    tag = "categorias",
    summary = "Edición parcial de categoría (GESTION)",
    description = "Omitir un campo lo conserva; `descripcion: \"\"` la limpia. `activo` en el cuerpo es 422.",
    params(("id" = i32, Path, description = "Id de la categoría")),
    request_body = ActualizarCategoriaDto,
    responses(
        (status = 200, description = "Categoría actualizada", body = Categoria),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Categoría inexistente", body = ErrorRespuesta),
        (status = 409, description = "Nombre duplicado", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos", body = ErrorRespuesta)
    )
)]
pub async fn actualizar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ActualizarCategoriaDto>,
) -> Result<Json<Categoria>, AppError> {
    if dto.activo.is_some() {
        return Err(AppError::Validacion(
            "El estado se cambia con /desactivar o /reactivar.".into(),
        ));
    }
    let cambios = CambiosCategoria {
        nombre: dto.nombre,
        descripcion: dto.descripcion,
    };
    Ok(Json(categorias::actualizar(&estado.pool, id, cambios).await?))
}

/// POST /api/categorias/{id}/desactivar → 204 (idempotente).
#[utoipa::path(
    post,
    path = "/api/categorias/{id}/desactivar",
    tag = "categorias",
    summary = "Baja lógica de categoría (GESTION)",
    params(("id" = i32, Path, description = "Id de la categoría")),
    responses(
        (status = 204, description = "Categoría desactivada (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Categoría inexistente", body = ErrorRespuesta)
    )
)]
pub async fn desactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    categorias::desactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/categorias/{id}/reactivar → 204 (idempotente).
#[utoipa::path(
    post,
    path = "/api/categorias/{id}/reactivar",
    tag = "categorias",
    summary = "Reactiva una categoría (GESTION)",
    params(("id" = i32, Path, description = "Id de la categoría")),
    responses(
        (status = 204, description = "Categoría reactivada (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Categoría inexistente", body = ErrorRespuesta)
    )
)]
pub async fn reactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    categorias::reactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
