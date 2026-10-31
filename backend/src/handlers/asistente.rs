//! Asistente de inventario (017): endpoint mínimo.

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::error::AppError;
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppEstado> {
    Router::new().route("/consulta", post(consulta))
}

#[derive(Deserialize)]
pub struct ConsultaDto {
    pub mensaje: String,
}

#[derive(Serialize)]
pub struct ConsultaRespuesta {
    pub respuesta: String,
}

pub async fn consulta(
    State(_e): State<AppEstado>,
    AuthUsuario(_u): AuthUsuario,
    Json(_dto): Json<ConsultaDto>,
) -> Result<Json<ConsultaRespuesta>, AppError> {
    if std::env::var("LLM_API_KEY").map(|v| v.trim().is_empty()).unwrap_or(true) {
        return Err(AppError::AsistenteNoDisponible);
    }
    // Implementación completa del asistente (LangChain + tools) fuera de alcance de este entorno.
    Ok(Json(ConsultaRespuesta {
        respuesta: "El asistente está configurado pero la integración LLM completa no está implementada en este entorno.".into(),
    }))
}
