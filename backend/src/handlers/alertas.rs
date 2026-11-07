//! Handlers de alertas de stock mínimo (011). Toda la superficie exige GESTION.

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::QueryValida;
use crate::models::alertas::{ListaAlertas, RespuestaCiclo};
use crate::services::alertas::{self, AlcanceCiclo};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use utoipa::IntoParams;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar))
        .route("/enviar", post(enviar))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarAlertasQuery {
    /// Página (≥ 1, default 1).
    pub pagina: Option<i64>,
    /// Tamaño de página (1–100, default 50).
    pub por_pagina: Option<i64>,
}

/// GET /api/alertas → productos activos bajo su mínimo.
#[utoipa::path(
    get,
    path = "/api/alertas",
    tag = "alertas",
    summary = "Productos bajo el stock mínimo (GESTION)",
    params(ListarAlertasQuery),
    responses(
        (status = 200, description = "Página de alertas", body = ListaAlertas),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Paginación inválida", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(query): QueryValida<ListarAlertasQuery>,
) -> Result<Json<ListaAlertas>, AppError> {
    Ok(Json(
        alertas::detectar(&estado.pool, query.pagina, query.por_pagina).await?,
    ))
}

/// POST /api/alertas/enviar → corre el ciclo de digest en el momento (idempotente en el día).
#[utoipa::path(
    post,
    path = "/api/alertas/enviar",
    tag = "alertas",
    summary = "Disparar el ciclo de alertas por correo (GESTION)",
    description = "Un correo por gestor activo con sus productos pendientes del día. Un fallo de transporte responde 200 con `fallidos > 0`.",
    responses(
        (status = 200, description = "Resultado del ciclo", body = RespuestaCiclo),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta)
    )
)]
pub async fn enviar(
    State(estado): State<AppEstado>,
    _gestion: RequiereGestion,
) -> Result<Json<RespuestaCiclo>, AppError> {
    Ok(Json(
        alertas::ciclo(
            &estado.pool,
            estado.correo.as_ref(),
            estado.reloj.as_ref(),
            &AlcanceCiclo::default(),
        )
        .await?,
    ))
}
