//! Handlers de auditoría consultable (015).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::QueryValida;
use crate::services::auditoria;
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new().route("/", get(listar))
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListarAuditoriaQuery {
    pub tabla_ref: Option<String>,
    pub usuario_id: Option<i32>,
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditoriaItem {
    pub id: i32,
    pub tabla_ref: String,
    pub registro_id: i32,
    pub accion: String,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub cuando: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListaAuditoria {
    pub items: Vec<AuditoriaItem>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
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
    path = "/api/auditoria",
    tag = "auditoria",
    summary = "Historial de auditoría",
    params(ListarAuditoriaQuery),
    responses(
        (status = 200, description = "Página de auditoría", body = ListaAuditoria),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Sin permisos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    QueryValida(query): QueryValida<ListarAuditoriaQuery>,
) -> Result<Json<ListaAuditoria>, AppError> {
    if usuario.familia_rol != "GESTION" {
        return Err(AppError::SinPermisos);
    }
    let desde = parsear_fecha(query.desde.as_deref(), "desde")?;
    let hasta = parsear_fecha(query.hasta.as_deref(), "hasta")?;
    let (filas, total) = auditoria::listar(
        &estado.pool,
        query.tabla_ref.as_deref(),
        query.usuario_id,
        desde,
        hasta,
        query.pagina,
        query.por_pagina,
    )
    .await?;
    let (pagina, por_pagina, _) =
        crate::services::normalizar_paginacion(query.pagina, query.por_pagina)?;
    let items = filas
        .into_iter()
        .map(|f| AuditoriaItem {
            id: f.id,
            tabla_ref: f.tabla_ref,
            registro_id: f.registro_id,
            accion: f.accion,
            usuario_id: f.usuario_id,
            usuario_nombre: f.usuario_nombre,
            cuando: f.cuando,
        })
        .collect();
    Ok(Json(ListaAuditoria {
        items,
        total,
        pagina,
        por_pagina,
    }))
}