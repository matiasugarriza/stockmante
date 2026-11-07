//! Handler del asistente de inventario (017).
//!
//! Cualquier sesión vigente puede conversar. La sesión se toma de la cookie y se
//! reenvía al servicio por header; nunca se lee del texto del usuario (17.5).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::JsonValido;
use crate::services::asistente::{MensajeHistorial, RespuestaAsistente};
use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use utoipa::ToSchema;

pub fn router() -> Router<AppEstado> {
    Router::new().route("/consulta", post(consulta))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConsultaDto {
    /// Pregunta o pedido en lenguaje natural (1–2000 caracteres).
    #[schema(example = "¿Cuánto stock hay de yerba?")]
    pub mensaje: String,
    /// Mensajes previos de la conversación en curso; se reenvían los 10 más recientes.
    #[serde(default)]
    pub historial: Vec<MensajeHistorial>,
}

/// POST /api/asistente/consulta → respuesta redactada y, si corresponde, una propuesta.
#[utoipa::path(
    post,
    path = "/api/asistente/consulta",
    tag = "asistente",
    summary = "Consultar al asistente de inventario",
    description = "Lecturas con los permisos del usuario. Nunca escribe: una `accion_propuesta` se ejecuta solo si el usuario la confirma con `POST /api/movimientos`. Límite: 20 consultas por minuto por usuario; 30 s por consulta.",
    request_body = ConsultaDto,
    responses(
        (status = 200, description = "Respuesta del asistente", body = RespuestaAsistente),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 422, description = "Mensaje vacío o demasiado largo", body = ErrorRespuesta),
        (status = 429, description = "Límite de consultas por minuto excedido", body = ErrorRespuesta),
        (status = 503, description = "Asistente no configurado o sin respuesta", body = ErrorRespuesta),
        (status = 504, description = "El asistente superó el tiempo máximo", body = ErrorRespuesta)
    )
)]
pub async fn consulta(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    jar: CookieJar,
    JsonValido(dto): JsonValido<ConsultaDto>,
) -> Result<Json<RespuestaAsistente>, AppError> {
    // `AuthUsuario` ya validó esta cookie; se reenvía tal cual para que las tools
    // actúen con la misma sesión (FR-068).
    let cookie = jar
        .get(&estado.config.session_cookie_name)
        .map(|c| format!("{}={}", c.name(), c.value()))
        .ok_or(AppError::NoAutenticado)?;
    let respuesta = estado
        .asistente
        .consultar(estado.reloj.ahora(), &usuario, &cookie, dto.mensaje, dto.historial)
        .await?;
    Ok(Json(respuesta))
}
