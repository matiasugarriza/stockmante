//! Extractores que normalizan los rechazos de Axum al sobre
//! `{ error: { codigo, mensaje } }` de `contracts/api.md`.
//!
//! Axum responde por defecto con texto plano y status 400/415/422. El contrato
//! del Módulo 1 exige `codigo: "validacion"` y status 422 para entradas mal
//! formadas, sin importar qué extractor las detectó.

use crate::error::AppError;
use axum::extract::{FromRequest, FromRequestParts, Json, Path, Query};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;

const MENSAJE_JSON: &str = "El cuerpo de la solicitud no es un JSON válido";
const MENSAJE_PATH: &str = "El parámetro de ruta no es válido";
const MENSAJE_QUERY: &str = "Los parámetros de la consulta no son válidos";

/// `Json<T>` con rechazos en el sobre de error del contrato.
pub struct JsonValido<T>(pub T);

impl<S, T> FromRequest<S> for JsonValido<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(valor)) => Ok(JsonValido(valor)),
            Err(rechazo) => {
                tracing::debug!(error = %rechazo.body_text(), "JSON rechazado");
                Err(AppError::Validacion(MENSAJE_JSON.into()))
            }
        }
    }
}

/// `Path<T>` con rechazos en el sobre de error del contrato.
pub struct PathValido<T>(pub T);

impl<S, T> FromRequestParts<S> for PathValido<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::<T>::from_request_parts(parts, state).await {
            Ok(Path(valor)) => Ok(PathValido(valor)),
            Err(rechazo) => {
                tracing::debug!(error = %rechazo.body_text(), "path rechazado");
                Err(AppError::Validacion(MENSAJE_PATH.into()))
            }
        }
    }
}

/// `Query<T>` con rechazos en el sobre de error del contrato.
pub struct QueryValida<T>(pub T);

impl<S, T> FromRequestParts<S> for QueryValida<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(valor)) => Ok(QueryValida(valor)),
            Err(rechazo) => {
                tracing::debug!(error = %rechazo.body_text(), "query rechazada");
                Err(AppError::Validacion(MENSAJE_QUERY.into()))
            }
        }
    }
}
