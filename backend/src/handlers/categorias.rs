//! Handlers de categorías (003, contracts/api.md).
//!
//! Lectura: cualquier sesión válida. Escritura: familia GESTION.
//! No se registra `DELETE`: Axum responde 405 con `Allow: GET, PUT`.

use crate::app::AppEstado;
use crate::auth::extractors::{AuthUsuario, RequiereGestion};
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

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/{id}", get(obtener).put(actualizar))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
}

#[derive(Debug, Deserialize)]
pub struct ListarQuery {
    pub activo: Option<String>,
}

/// GET /api/categorias?activo=true|false|all (default: solo activas).
async fn listar(
    State(estado): State<AppEstado>,
    _sesion: AuthUsuario,
    QueryValida(query): QueryValida<ListarQuery>,
) -> Result<Json<Vec<Categoria>>, AppError> {
    let activo = super::parsear_activo(query.activo.as_deref(), Some(true))?;
    Ok(Json(categorias::listar(&estado.pool, activo).await?))
}

#[derive(Debug, Deserialize)]
pub struct CrearCategoriaDto {
    pub nombre: String,
    #[serde(default)]
    pub descripcion: Option<String>,
}

/// POST /api/categorias → 201 categoría.
async fn crear(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    JsonValido(dto): JsonValido<CrearCategoriaDto>,
) -> Result<(StatusCode, Json<Categoria>), AppError> {
    let categoria =
        categorias::crear(&estado.pool, &dto.nombre, dto.descripcion.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(categoria)))
}

/// GET /api/categorias/{id} → 200 o 404 (también para inactivas).
async fn obtener(
    State(estado): State<AppEstado>,
    _sesion: AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Categoria>, AppError> {
    Ok(Json(categorias::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct ActualizarCategoriaDto {
    #[serde(default)]
    pub nombre: Option<String>,
    #[serde(default, deserialize_with = "super::deserializar_presente")]
    pub descripcion: Option<Option<String>>,
    /// `activo` no viaja por `PUT`: su sola presencia es 422.
    #[serde(default)]
    pub activo: Option<serde_json::Value>,
}

/// PUT /api/categorias/{id} → 200 categoría actualizada.
async fn actualizar(
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
async fn desactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    categorias::desactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/categorias/{id}/reactivar → 204 (idempotente).
async fn reactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    categorias::reactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
