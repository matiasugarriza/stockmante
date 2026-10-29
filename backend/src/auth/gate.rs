//! Gate global de contraseña temporal (FR-008a / T028).
//!
//! Toda la superficie autenticada pasa por acá antes de los extractores de
//! autorización, así el orden es siempre temporal → familia: una cuenta con
//! contraseña temporal vigente sólo alcanza `GET /auth/sesion` y
//! `POST /auth/cambiar-password` (esas rutas no están detrás de este gate).

use crate::app::AppEstado;
use crate::error::AppError;
use crate::services;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

/// 403 `cambio_password_requerido` si la sesión tiene `password_temporal`.
///
/// Sin cookie, con cookie ilegible o con sesión inexistente/expirada deja pasar
/// la request: el extractor del handler responde el 401 que corresponde.
pub async fn exigir_password_cambiada(
    State(estado): State<AppEstado>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let jar = CookieJar::from_headers(req.headers());
    let Some(cookie) = jar.get(&estado.config.session_cookie_name) else {
        return Ok(next.run(req).await);
    };
    let Ok(sesion_id) = Uuid::parse_str(cookie.value()) else {
        return Ok(next.run(req).await);
    };
    let usuario =
        services::auth::sesion_actual(&estado.pool, estado.reloj.as_ref(), sesion_id).await?;
    if usuario.password_temporal {
        return Err(AppError::DebeCambiarPassword);
    }
    Ok(next.run(req).await)
}
