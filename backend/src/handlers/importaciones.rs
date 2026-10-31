//! Handler de importación masiva (016).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::services::importaciones;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use utoipa::ToSchema;

pub fn router() -> Router<AppEstado> {
    Router::new().route("/productos", post(importar_productos))
}

#[derive(serde::Deserialize)]
pub struct ImportarQuery {
    pub modo: Option<String>,
}

#[utoipa::path(post, path = "/api/importaciones/productos", tag = "importaciones", summary = "Importar productos CSV",
    responses((status = 200, description = "Validación"), (status = 201, description = "Aplicado")))]
pub async fn importar_productos(
    State(e): State<AppEstado>,
    AuthUsuario(u): AuthUsuario,
    axum::extract::Query(q): axum::extract::Query<ImportarQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    if u.familia_rol != "GESTION" { return Err(AppError::SinPermisos); }
    let modo = q.modo.clone().unwrap_or_else(|| "validar".to_string());
    if modo != "validar" && modo != "aplicar" {
        return Err(AppError::Validacion("modo debe ser validar o aplicar.".into()));
    }

    let mut bytes: Vec<u8> = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::Validacion("Multipart inválido.".into()))? {
        if field.name() == Some("archivo") {
            let data = field.bytes().await.map_err(|_| AppError::Validacion("No se pudo leer el archivo.".into()))?;
            bytes = data.to_vec();
        }
    }
    if bytes.is_empty() {
        return Err(AppError::Validacion("Falta el archivo.".into()));
    }

    let resumen = importaciones::importar(&e.pool, &bytes, &modo, &u).await?;
    if modo == "aplicar" {
        if !resumen.errores.is_empty() {
            return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(resumen)).into_response());
        }
        return Ok((StatusCode::CREATED, Json(resumen)).into_response());
    }
    Ok((StatusCode::OK, Json(resumen)).into_response())
}
