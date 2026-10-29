//! Handlers de depósitos (005, contracts/api.md).
//!
//! Toda la superficie exige familia GESTION. El borrado físico responde 409
//! `deposito_con_dependencias`; la baja lógica es la alternativa.

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::deposito::{Deposito, ListaUbicaciones};
use crate::services::depositos::{self, CambiosDeposito, NuevoDeposito};
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
        .route(
            "/{id}",
            get(obtener).put(actualizar).delete(eliminar),
        )
        .route("/{id}/ubicaciones", get(listar_ubicaciones))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarDepositosQuery {
    /// `true`, `false` o `all` (ausente = solo activos).
    pub activo: Option<String>,
}

/// GET /api/depositos?activo=
#[utoipa::path(
    get,
    path = "/api/depositos",
    tag = "depositos",
    summary = "Listado de depósitos (GESTION)",
    params(ListarDepositosQuery),
    responses(
        (status = 200, description = "Depósitos ordenados por nombre", body = Vec<Deposito>),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Filtro activo inválido", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(query): QueryValida<ListarDepositosQuery>,
) -> Result<Json<Vec<Deposito>>, AppError> {
    let activo = super::parsear_activo(query.activo.as_deref(), Some(true))?;
    Ok(Json(depositos::listar(&estado.pool, activo).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearDepositoDto {
    #[schema(example = "Depósito Central")]
    pub nombre: String,
    #[schema(example = "DEP-CENTRAL")]
    pub identificador: String,
    #[serde(default)]
    #[schema(example = "Av. Siempre Viva 742")]
    pub direccion: Option<String>,
}

/// POST /api/depositos → 201.
#[utoipa::path(
    post,
    path = "/api/depositos",
    tag = "depositos",
    summary = "Alta de depósito (GESTION)",
    request_body = CrearDepositoDto,
    responses(
        (status = 201, description = "Depósito creado", body = Deposito),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 409, description = "Nombre o identificador duplicado", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    JsonValido(dto): JsonValido<CrearDepositoDto>,
) -> Result<(StatusCode, Json<Deposito>), AppError> {
    let nuevo = NuevoDeposito {
        nombre: dto.nombre,
        identificador: dto.identificador,
        direccion: dto.direccion,
    };
    let deposito = depositos::crear(&estado.pool, nuevo).await?;
    Ok((StatusCode::CREATED, Json(deposito)))
}

/// GET /api/depositos/{id}
#[utoipa::path(
    get,
    path = "/api/depositos/{id}",
    tag = "depositos",
    summary = "Detalle de depósito (GESTION)",
    params(("id" = i32, Path, description = "Id del depósito")),
    responses(
        (status = 200, description = "Depósito", body = Deposito),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Depósito inexistente", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Deposito>, AppError> {
    Ok(Json(depositos::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizarDepositoDto {
    #[serde(default)]
    pub nombre: Option<String>,
    #[serde(default)]
    pub identificador: Option<String>,
    #[serde(default, deserialize_with = "super::deserializar_presente")]
    #[schema(value_type = Option<String>)]
    pub direccion: Option<Option<String>>,
    /// `activo` no viaja por `PUT`: su sola presencia es 422.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub activo: Option<serde_json::Value>,
}

/// PUT /api/depositos/{id}
#[utoipa::path(
    put,
    path = "/api/depositos/{id}",
    tag = "depositos",
    summary = "Edición parcial de depósito (GESTION)",
    description = "Omitir un campo lo conserva; `direccion: \"\"` la limpia.",
    params(("id" = i32, Path, description = "Id del depósito")),
    request_body = ActualizarDepositoDto,
    responses(
        (status = 200, description = "Depósito actualizado", body = Deposito),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Depósito inexistente", body = ErrorRespuesta),
        (status = 409, description = "Nombre o identificador duplicado", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos", body = ErrorRespuesta)
    )
)]
pub async fn actualizar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ActualizarDepositoDto>,
) -> Result<Json<Deposito>, AppError> {
    if dto.activo.is_some() {
        return Err(AppError::Validacion(
            "El estado se cambia con /desactivar o /reactivar.".into(),
        ));
    }
    let cambios = CambiosDeposito {
        nombre: dto.nombre,
        identificador: dto.identificador,
        direccion: dto.direccion,
    };
    Ok(Json(depositos::actualizar(&estado.pool, id, cambios).await?))
}

/// DELETE /api/depositos/{id} → 204 solo sin dependencias.
#[utoipa::path(
    delete,
    path = "/api/depositos/{id}",
    tag = "depositos",
    summary = "Borrado físico de depósito sin dependencias (GESTION)",
    params(("id" = i32, Path, description = "Id del depósito")),
    responses(
        (status = 204, description = "Depósito borrado"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Depósito inexistente", body = ErrorRespuesta),
        (status = 409, description = "Tiene ubicaciones o asignaciones", body = ErrorRespuesta)
    )
)]
pub async fn eliminar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    depositos::eliminar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/depositos/{id}/ubicaciones → lista plana con `padre_id`.
#[utoipa::path(
    get,
    path = "/api/depositos/{id}/ubicaciones",
    tag = "depositos",
    summary = "Ubicaciones de un depósito (GESTION)",
    params(("id" = i32, Path, description = "Id del depósito")),
    responses(
        (status = 200, description = "Ubicaciones del depósito", body = ListaUbicaciones),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Depósito inexistente", body = ErrorRespuesta)
    )
)]
pub async fn listar_ubicaciones(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<ListaUbicaciones>, AppError> {
    let items = depositos::listar_ubicaciones(&estado.pool, id).await?;
    Ok(Json(ListaUbicaciones { items }))
}

/// POST /api/depositos/{id}/desactivar → 204.
#[utoipa::path(
    post,
    path = "/api/depositos/{id}/desactivar",
    tag = "depositos",
    summary = "Baja lógica de depósito (GESTION)",
    params(("id" = i32, Path, description = "Id del depósito")),
    responses(
        (status = 204, description = "Depósito desactivado (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Depósito inexistente", body = ErrorRespuesta)
    )
)]
pub async fn desactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    depositos::desactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/depositos/{id}/reactivar → 204.
#[utoipa::path(
    post,
    path = "/api/depositos/{id}/reactivar",
    tag = "depositos",
    summary = "Reactiva un depósito (GESTION)",
    params(("id" = i32, Path, description = "Id del depósito")),
    responses(
        (status = 204, description = "Depósito reactivado (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Depósito inexistente", body = ErrorRespuesta)
    )
)]
pub async fn reactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    depositos::reactivar(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
