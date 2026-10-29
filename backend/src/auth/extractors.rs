//! Extractores de autenticación y autorización (M1-A).

use crate::app::AppEstado;
use crate::error::AppError;
use crate::models::usuario::Usuario;
use crate::services;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

/// Usuario autenticado a partir de la cookie de sesión.
/// 401 `no_autenticado` / `sesion_expirada` si no hay sesión vigente.
pub struct AuthUsuario(pub Usuario);

impl FromRequestParts<AppEstado> for AuthUsuario {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppEstado,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let Some(cookie) = jar.get(&state.config.session_cookie_name) else {
            return Err(AppError::NoAutenticado);
        };
        let Ok(sesion_id) = Uuid::parse_str(cookie.value()) else {
            return Err(AppError::NoAutenticado);
        };
        let usuario =
            services::auth::sesion_actual(&state.pool, state.reloj.as_ref(), sesion_id).await?;
        Ok(AuthUsuario(usuario))
    }
}

/// Requiere sesión válida + familia `GESTION` (403 `sin_permisos` si es OPERACION).
/// El orden es temporal → familia: con `password_temporal` vigente responde 403
/// `cambio_password_requerido`, sin importar el rol (FR-008a). El gate global de
/// `auth::gate` aplica la misma regla a toda la superficie autenticada; éste es
/// el respaldo del extractor cuando se usa fuera de ese grupo.
pub struct RequiereGestion(pub Usuario);

impl FromRequestParts<AppEstado> for RequiereGestion {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppEstado,
    ) -> Result<Self, Self::Rejection> {
        let AuthUsuario(usuario) = AuthUsuario::from_request_parts(parts, state).await?;
        if usuario.password_temporal {
            return Err(AppError::DebeCambiarPassword);
        }
        if usuario.familia_rol != "GESTION" {
            return Err(AppError::SinPermisos);
        }
        Ok(RequiereGestion(usuario))
    }
}
