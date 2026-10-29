//! Handlers de transferencias entre depósitos (008, contracts/api.md).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::stock::{ListaTransferencias, TransferenciaDetalle};
use crate::services::transferencias::{self, FiltrosTransferencias, NuevaTransferencia};
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/{id}", get(obtener))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearTransferenciaDto {
    pub producto_id: i32,
    pub deposito_origen_id: i32,
    pub deposito_destino_id: i32,
    pub cantidad: i32,
    pub ubicacion_origen_id: Option<i32>,
    pub ubicacion_destino_id: Option<i32>,
    pub observacion: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/transferencias",
    tag = "transferencias",
    summary = "Registrar una transferencia entre depósitos",
    description = "Una transacción por request: descuenta origen, incrementa destino, escribe dos movimientos enlazados.",
    request_body = CrearTransferenciaDto,
    responses(
        (status = 201, description = "Transferencia registrada", body = TransferenciaDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta),
        (status = 409, description = "Stock insuficiente o producto inactivo", body = ErrorRespuesta),
        (status = 422, description = "Cuerpo inválido o mismo depósito", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    JsonValido(dto): JsonValido<CrearTransferenciaDto>,
) -> Result<(StatusCode, Json<TransferenciaDetalle>), AppError> {
    let nueva = NuevaTransferencia {
        producto_id: dto.producto_id,
        deposito_origen_id: dto.deposito_origen_id,
        deposito_destino_id: dto.deposito_destino_id,
        cantidad: dto.cantidad,
        ubicacion_origen_id: dto.ubicacion_origen_id,
        ubicacion_destino_id: dto.ubicacion_destino_id,
        observacion: dto.observacion,
    };
    let transferencia = transferencias::crear(&estado.pool, nueva, &usuario).await?;
    Ok((StatusCode::CREATED, Json(transferencia)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarTransferenciasQuery {
    pub producto_id: Option<i32>,
    pub deposito_origen_id: Option<i32>,
    pub deposito_destino_id: Option<i32>,
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
    path = "/api/transferencias",
    tag = "transferencias",
    summary = "Historial de transferencias",
    params(ListarTransferenciasQuery),
    responses(
        (status = 200, description = "Página de transferencias", body = ListaTransferencias),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 422, description = "Filtros inválidos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _usuario: AuthUsuario,
    QueryValida(query): QueryValida<ListarTransferenciasQuery>,
) -> Result<Json<ListaTransferencias>, AppError> {
    let filtros = FiltrosTransferencias {
        producto_id: query.producto_id,
        deposito_origen_id: query.deposito_origen_id,
        deposito_destino_id: query.deposito_destino_id,
        desde: parsear_fecha(query.desde.as_deref(), "desde")?,
        hasta: parsear_fecha(query.hasta.as_deref(), "hasta")?,
        pagina: query.pagina,
        por_pagina: query.por_pagina,
    };
    Ok(Json(transferencias::listar(&estado.pool, filtros).await?))
}

#[utoipa::path(
    get,
    path = "/api/transferencias/{id}",
    tag = "transferencias",
    summary = "Detalle de una transferencia",
    params(("id" = i32, Path, description = "Identificador de la transferencia")),
    responses(
        (status = 200, description = "Transferencia con sus movimientos", body = TransferenciaDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 404, description = "No existe", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _usuario: AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<TransferenciaDetalle>, AppError> {
    Ok(Json(transferencias::obtener(&estado.pool, id).await?))
}
