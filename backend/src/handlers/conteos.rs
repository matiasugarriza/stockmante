//! Handlers de conteos de inventario (009, contracts/api.md).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::stock::{ConteoDetalle, ListaConteos};
use crate::services::conteos::{self, FiltrosConteos, LineaContadaInput, NuevoConteo};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/{id}", get(obtener))
        .route("/{id}/lineas", patch(cargar_lineas))
        .route("/{id}/cerrar", post(cerrar))
        .route("/{id}/cancelar", post(cancelar))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearConteoDto {
    pub deposito_id: i32,
    pub observacion: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LineaContadaDto {
    pub linea_id: i32,
    pub cantidad_contada: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CargarLineasDto {
    pub lineas: Vec<LineaContadaDto>,
}

fn exigir_gestion(usuario: &crate::models::usuario::Usuario) -> Result<(), AppError> {
    if usuario.familia_rol != "GESTION" {
        return Err(AppError::SinPermisos);
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/conteos",
    tag = "conteos",
    summary = "Abrir un conteo de inventario",
    request_body = CrearConteoDto,
    responses(
        (status = 201, description = "Conteo abierto", body = ConteoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 422, description = "Cuerpo inválido", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    JsonValido(dto): JsonValido<CrearConteoDto>,
) -> Result<(StatusCode, Json<ConteoDetalle>), AppError> {
    exigir_gestion(&usuario)?;
    let nuevo = NuevoConteo {
        deposito_id: dto.deposito_id,
        observacion: dto.observacion,
    };
    let conteo = conteos::crear(&estado.pool, nuevo, &usuario).await?;
    Ok((StatusCode::CREATED, Json(conteo)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarConteosQuery {
    pub deposito_id: Option<i32>,
    pub estado: Option<String>,
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

fn parsear_fecha(valor: Option<&str>, campo: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    let Some(texto) = valor.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if let Ok(fecha) = DateTime::parse_from_rfc3339(texto) {
        return Ok(Some(fecha.with_timezone(&Utc)));
    }
    if let Ok(dia) = NaiveDate::parse_from_str(texto, "%Y-%m-%d") {
        return Ok(dia.and_hms_opt(0, 0, 0).map(|fecha| fecha.and_utc()));
    }
    Err(AppError::Validacion(format!(
        "El parámetro {campo} debe estar en formato ISO 8601."
    )))
}

#[utoipa::path(
    get,
    path = "/api/conteos",
    tag = "conteos",
    summary = "Historial de conteos",
    params(ListarConteosQuery),
    responses(
        (status = 200, description = "Página de conteos", body = ListaConteos),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 422, description = "Filtros inválidos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    QueryValida(query): QueryValida<ListarConteosQuery>,
) -> Result<Json<ListaConteos>, AppError> {
    exigir_gestion(&usuario)?;
    let filtros = FiltrosConteos {
        deposito_id: query.deposito_id,
        estado: query.estado,
        desde: parsear_fecha(query.desde.as_deref(), "desde")?,
        hasta: parsear_fecha(query.hasta.as_deref(), "hasta")?,
        pagina: query.pagina,
        por_pagina: query.por_pagina,
    };
    Ok(Json(conteos::listar(&estado.pool, filtros).await?))
}

#[utoipa::path(
    get,
    path = "/api/conteos/{id}",
    tag = "conteos",
    summary = "Detalle de un conteo",
    params(("id" = i32, Path, description = "Identificador del conteo")),
    responses(
        (status = 200, description = "Conteo con sus líneas", body = ConteoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 404, description = "No existe", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<ConteoDetalle>, AppError> {
    exigir_gestion(&usuario)?;
    Ok(Json(conteos::obtener(&estado.pool, id).await?))
}

#[utoipa::path(
    patch,
    path = "/api/conteos/{id}/lineas",
    tag = "conteos",
    summary = "Cargar cantidades contadas",
    request_body = CargarLineasDto,
    params(("id" = i32, Path, description = "Identificador del conteo")),
    responses(
        (status = 200, description = "Conteo actualizado", body = ConteoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 404, description = "No existe", body = ErrorRespuesta),
        (status = 409, description = "Conteo cerrado o cancelado", body = ErrorRespuesta),
        (status = 422, description = "Cuerpo inválido", body = ErrorRespuesta)
    )
)]
pub async fn cargar_lineas(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<CargarLineasDto>,
) -> Result<Json<ConteoDetalle>, AppError> {
    exigir_gestion(&usuario)?;
    let lineas: Vec<LineaContadaInput> = dto
        .lineas
        .into_iter()
        .map(|l| LineaContadaInput {
            linea_id: l.linea_id,
            cantidad_contada: l.cantidad_contada,
        })
        .collect();
    Ok(Json(
        conteos::cargar_lineas(&estado.pool, id, &lineas, &usuario).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/conteos/{id}/cerrar",
    tag = "conteos",
    summary = "Cerrar un conteo y ajustar diferencias",
    params(("id" = i32, Path, description = "Identificador del conteo")),
    responses(
        (status = 200, description = "Conteo cerrado", body = ConteoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 404, description = "No existe", body = ErrorRespuesta),
        (status = 409, description = "Conteo cerrado o cancelado", body = ErrorRespuesta),
        (status = 422, description = "Líneas sin contar", body = ErrorRespuesta)
    )
)]
pub async fn cerrar(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<ConteoDetalle>, AppError> {
    exigir_gestion(&usuario)?;
    Ok(Json(conteos::cerrar(&estado.pool, id, &usuario).await?))
}

#[utoipa::path(
    post,
    path = "/api/conteos/{id}/cancelar",
    tag = "conteos",
    summary = "Cancelar un conteo abierto",
    params(("id" = i32, Path, description = "Identificador del conteo")),
    responses(
        (status = 200, description = "Conteo cancelado", body = ConteoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 404, description = "No existe", body = ErrorRespuesta),
        (status = 409, description = "Conteo cerrado o cancelado", body = ErrorRespuesta)
    )
)]
pub async fn cancelar(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<ConteoDetalle>, AppError> {
    exigir_gestion(&usuario)?;
    Ok(Json(conteos::cancelar(&estado.pool, id, &usuario).await?))
}