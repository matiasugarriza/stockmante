//! Handlers de ubicaciones jerárquicas (005, contracts/api.md).
//!
//! Crear una ubicación con `deposito_id` inexistente/inactivo o con un padre de
//! otro depósito responde 422 `ubicacion_huerfana`. Mover de depósito solo sin
//! hijos ni asignaciones; el resto es 409 `ubicacion_con_dependencias`.

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido};
use crate::models::deposito::Ubicacion;
use crate::services::depositos::{self, CambiosUbicacion, NuevaUbicacion};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use utoipa::ToSchema;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", post(crear))
        .route("/{id}", get(obtener).put(actualizar).delete(eliminar))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearUbicacionDto {
    pub deposito_id: i32,
    #[serde(default)]
    pub padre_id: Option<i32>,
    #[schema(example = "Estantería A")]
    pub nombre: String,
    #[schema(example = "ESTANTERIA")]
    pub tipo: String,
}

/// POST /api/ubicaciones → 201.
#[utoipa::path(
    post,
    path = "/api/ubicaciones",
    tag = "ubicaciones",
    summary = "Alta de ubicación jerárquica (GESTION)",
    request_body = CrearUbicacionDto,
    responses(
        (status = 201, description = "Ubicación creada", body = Ubicacion),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 409, description = "Nombre repetido en el mismo nivel", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos o ubicación huérfana", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    JsonValido(dto): JsonValido<CrearUbicacionDto>,
) -> Result<(StatusCode, Json<Ubicacion>), AppError> {
    let nueva = NuevaUbicacion {
        deposito_id: dto.deposito_id,
        padre_id: dto.padre_id,
        nombre: dto.nombre,
        tipo: dto.tipo,
    };
    let ubicacion = depositos::crear_ubicacion(&estado.pool, nueva).await?;
    Ok((StatusCode::CREATED, Json(ubicacion)))
}

/// GET /api/ubicaciones/{id}
#[utoipa::path(
    get,
    path = "/api/ubicaciones/{id}",
    tag = "ubicaciones",
    summary = "Detalle de ubicación (GESTION)",
    params(("id" = i32, Path, description = "Id de la ubicación")),
    responses(
        (status = 200, description = "Ubicación", body = Ubicacion),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Ubicación inexistente", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Ubicacion>, AppError> {
    Ok(Json(depositos::obtener_ubicacion(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizarUbicacionDto {
    #[serde(default)]
    pub nombre: Option<String>,
    #[serde(default)]
    pub tipo: Option<String>,
    #[serde(default, deserialize_with = "super::deserializar_presente")]
    #[schema(value_type = Option<i32>)]
    pub padre_id: Option<Option<i32>>,
    #[serde(default)]
    pub deposito_id: Option<i32>,
    /// `activo` no viaja por `PUT`: su sola presencia es 422.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub activo: Option<serde_json::Value>,
}

/// PUT /api/ubicaciones/{id}
#[utoipa::path(
    put,
    path = "/api/ubicaciones/{id}",
    tag = "ubicaciones",
    summary = "Edición parcial de ubicación (GESTION)",
    description = "`padre_id: null` la mueve a la raíz. Cambiar de depósito exige no tener hijos ni asignaciones.",
    params(("id" = i32, Path, description = "Id de la ubicación")),
    request_body = ActualizarUbicacionDto,
    responses(
        (status = 200, description = "Ubicación actualizada", body = Ubicacion),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Ubicación inexistente", body = ErrorRespuesta),
        (status = 409, description = "Nombre repetido o dependencias al mover", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos, ciclo o padre de otro depósito", body = ErrorRespuesta)
    )
)]
pub async fn actualizar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ActualizarUbicacionDto>,
) -> Result<Json<Ubicacion>, AppError> {
    if dto.activo.is_some() {
        return Err(AppError::Validacion(
            "El estado se cambia con /desactivar o /reactivar.".into(),
        ));
    }
    let cambios = CambiosUbicacion {
        nombre: dto.nombre,
        tipo: dto.tipo,
        padre_id: dto.padre_id,
        deposito_id: dto.deposito_id,
    };
    Ok(Json(
        depositos::actualizar_ubicacion(&estado.pool, id, cambios).await?,
    ))
}

/// DELETE /api/ubicaciones/{id} → 204 solo sin hijos ni asignaciones.
#[utoipa::path(
    delete,
    path = "/api/ubicaciones/{id}",
    tag = "ubicaciones",
    summary = "Borrado físico de ubicación sin dependencias (GESTION)",
    params(("id" = i32, Path, description = "Id de la ubicación")),
    responses(
        (status = 204, description = "Ubicación borrada"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Ubicación inexistente", body = ErrorRespuesta),
        (status = 409, description = "Tiene hijos o productos asignados", body = ErrorRespuesta)
    )
)]
pub async fn eliminar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    depositos::eliminar_ubicacion(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/ubicaciones/{id}/desactivar → 204.
#[utoipa::path(
    post,
    path = "/api/ubicaciones/{id}/desactivar",
    tag = "ubicaciones",
    summary = "Baja lógica de ubicación (GESTION)",
    params(("id" = i32, Path, description = "Id de la ubicación")),
    responses(
        (status = 204, description = "Ubicación desactivada (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Ubicación inexistente", body = ErrorRespuesta)
    )
)]
pub async fn desactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    depositos::desactivar_ubicacion(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/ubicaciones/{id}/reactivar → 204.
#[utoipa::path(
    post,
    path = "/api/ubicaciones/{id}/reactivar",
    tag = "ubicaciones",
    summary = "Reactiva una ubicación (GESTION)",
    params(("id" = i32, Path, description = "Id de la ubicación")),
    responses(
        (status = 204, description = "Ubicación reactivada (idempotente)"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Ubicación inexistente", body = ErrorRespuesta)
    )
)]
pub async fn reactivar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    depositos::reactivar_ubicacion(&estado.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
