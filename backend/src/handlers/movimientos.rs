//! Handlers de movimientos de stock (007, contracts/api.md).
//!
//! El alta la puede hacer cualquier familia, salvo `AJUSTE`, que el servicio reserva a
//! `GESTION` (FR-027). El autor siempre sale de la sesión. La consulta está abierta a
//! cualquier sesión vigente.

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::movimiento::LineaNueva;
use crate::models::stock::{ListaMovimientos, MovimientoDetalle};
use crate::services::movimientos::{self, FiltrosMovimientos, NuevoMovimiento};
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

/// Línea del alta: `cantidad_delta` lleva el signo del tipo.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LineaMovimientoDto {
    #[schema(example = 1)]
    pub producto_id: i32,
    /// Opcional; si falta, el stock se imputa al depósito.
    pub ubicacion_id: Option<i32>,
    /// `ENTRADA > 0`, `SALIDA`/`MERMA < 0`, `AJUSTE ≠ 0`.
    #[schema(example = 10)]
    pub cantidad_delta: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearMovimientoDto {
    #[schema(example = "ENTRADA")]
    pub tipo: String,
    #[schema(example = 1)]
    pub deposito_id: i32,
    #[schema(example = "COMPRA")]
    pub motivo: String,
    /// Obligatoria cuando el motivo es `OTRO`.
    pub observacion: Option<String>,
    pub lineas: Vec<LineaMovimientoDto>,
}

/// POST /api/movimientos → 201 con el movimiento, su autor y el stock resultante.
#[utoipa::path(
    post,
    path = "/api/movimientos",
    tag = "movimientos",
    summary = "Registrar un movimiento de stock",
    description = "Una transacción por request: si una línea falla, no se escribe nada. `AJUSTE` requiere GESTION.",
    request_body = CrearMovimientoDto,
    responses(
        (status = 201, description = "Movimiento registrado", body = MovimientoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "AJUSTE requiere GESTION", body = ErrorRespuesta),
        (status = 409, description = "Stock insuficiente o producto inactivo", body = ErrorRespuesta),
        (status = 422, description = "Cuerpo, tipo, motivo o líneas inválidas", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    JsonValido(dto): JsonValido<CrearMovimientoDto>,
) -> Result<(StatusCode, Json<MovimientoDetalle>), AppError> {
    let nuevo = NuevoMovimiento {
        tipo: dto.tipo,
        deposito_id: dto.deposito_id,
        motivo: dto.motivo,
        observacion: dto.observacion,
        lineas: dto
            .lineas
            .into_iter()
            .map(|linea| LineaNueva {
                producto_id: linea.producto_id,
                ubicacion_id: linea.ubicacion_id,
                cantidad_delta: linea.cantidad_delta,
            })
            .collect(),
    };
    let movimiento = movimientos::crear(&estado.pool, nuevo, &usuario).await?;
    Ok((StatusCode::CREATED, Json(movimiento)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarMovimientosQuery {
    /// `ENTRADA`, `SALIDA`, `MERMA` o `AJUSTE`.
    pub tipo: Option<String>,
    pub deposito_id: Option<i32>,
    /// Movimientos que incluyan ese producto.
    pub producto_id: Option<i32>,
    /// ISO 8601 (`2026-10-01` o `2026-10-01T12:00:00Z`).
    pub desde: Option<String>,
    /// ISO 8601, inclusivo.
    pub hasta: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// Acepta `YYYY-MM-DD` y RFC 3339; cualquier otra forma es 422.
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

/// GET /api/movimientos?tipo=&deposito_id=&producto_id=&desde=&hasta=&pagina=&por_pagina=
#[utoipa::path(
    get,
    path = "/api/movimientos",
    tag = "movimientos",
    summary = "Historial de movimientos",
    description = "Orden descendente por fecha. Cada item trae sus líneas con el stock resultante.",
    params(ListarMovimientosQuery),
    responses(
        (status = 200, description = "Página de movimientos", body = ListaMovimientos),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 422, description = "Filtros inválidos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _usuario: AuthUsuario,
    QueryValida(query): QueryValida<ListarMovimientosQuery>,
) -> Result<Json<ListaMovimientos>, AppError> {
    let filtros = FiltrosMovimientos {
        tipo: query.tipo,
        deposito_id: query.deposito_id,
        producto_id: query.producto_id,
        desde: parsear_fecha(query.desde.as_deref(), "desde")?,
        hasta: parsear_fecha(query.hasta.as_deref(), "hasta")?,
        pagina: query.pagina,
        por_pagina: query.por_pagina,
    };
    Ok(Json(movimientos::listar(&estado.pool, filtros).await?))
}

/// GET /api/movimientos/{id} → 200 con el detalle; 404 si no existe.
#[utoipa::path(
    get,
    path = "/api/movimientos/{id}",
    tag = "movimientos",
    summary = "Detalle de un movimiento",
    params(("id" = i32, Path, description = "Identificador del movimiento")),
    responses(
        (status = 200, description = "Movimiento con sus líneas", body = MovimientoDetalle),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 404, description = "No existe", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _usuario: AuthUsuario,
    PathValido(id): PathValido<i32>,
) -> Result<Json<MovimientoDetalle>, AppError> {
    Ok(Json(movimientos::obtener(&estado.pool, id).await?))
}
