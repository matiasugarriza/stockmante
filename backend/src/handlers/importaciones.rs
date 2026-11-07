//! Handler de importación masiva (016). Solo GESTION.

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
use crate::error::{AppError, DetalleImportacion};
use crate::services::importaciones::{self, ResumenImportacion};
use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use utoipa::IntoParams;

/// Axum limita el body a 2 MB; se sube a 16 MB para que el límite de negocio (5 MB) sea
/// alcanzable y responda 422 con su mensaje (D8).
const LIMITE_CUERPO: usize = 16 * 1024 * 1024;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/productos", post(importar_productos))
        .layer(DefaultBodyLimit::max(LIMITE_CUERPO))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ImportarQuery {
    /// `validar` (default, no escribe) o `aplicar` (todo o nada).
    pub modo: Option<String>,
}

fn falta_archivo() -> AppError {
    AppError::ImportacionInvalida(vec![DetalleImportacion {
        fila: 0,
        columna: "archivo".into(),
        mensaje: "Falta el archivo.".into(),
    }])
}

#[utoipa::path(post, path = "/api/importaciones/productos", tag = "importaciones",
    summary = "Importar productos y stock inicial desde CSV (GESTION)",
    description = "`multipart/form-data` con el campo `archivo` (CSV UTF-8, hasta 5.000 filas y 5 MB). `modo=validar` no escribe; `modo=aplicar` es todo o nada.",
    params(ImportarQuery),
    responses(
        (status = 200, description = "Dry-run: resumen con errores por celda", body = ResumenImportacion),
        (status = 201, description = "Aplicado sin errores", body = ResumenImportacion),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "`importacion_invalida` con `detalles`, o `validacion` si `modo` es inválido", body = ErrorRespuesta)
    ))]
pub async fn importar_productos(
    State(e): State<AppEstado>,
    RequiereGestion(u): RequiereGestion,
    axum::extract::Query(q): axum::extract::Query<ImportarQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let aplicar = match q.modo.as_deref().map(str::trim) {
        None | Some("") | Some("validar") => false,
        Some("aplicar") => true,
        Some(_) => return Err(AppError::Validacion("modo debe ser validar o aplicar.".into())),
    };

    let mut archivo: Option<Vec<u8>> = None;
    while let Some(campo) = multipart.next_field().await.map_err(|_| {
        AppError::Validacion("El cuerpo multipart es inválido.".into())
    })? {
        if campo.name() == Some("archivo") {
            let datos = campo.bytes().await.map_err(|_| {
                AppError::ImportacionInvalida(vec![DetalleImportacion {
                    fila: 0,
                    columna: "archivo".into(),
                    mensaje: "No se pudo leer el CSV. Revisá el formato del archivo.".into(),
                }])
            })?;
            archivo = Some(datos.to_vec());
        }
    }
    let bytes = archivo.ok_or_else(falta_archivo)?;

    let resumen = importaciones::importar(&e.pool, &bytes, aplicar, &u).await?;
    let status = if aplicar { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(resumen)))
}
