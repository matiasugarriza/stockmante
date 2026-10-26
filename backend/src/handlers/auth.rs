//! Handlers de autenticación (contracts/api.md): login, logout, sesión, cambio de password.

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::error::AppError;
use crate::extractores::JsonValido;
use crate::services;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use time::{Duration as Duracion, OffsetDateTime};
use uuid::Uuid;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/sesion", get(sesion))
        .route("/cambiar-password", post(cambiar_password))
}

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CambioPasswordDto {
    pub password_actual: String,
    pub password_nueva: String,
}

/// Usuario tal como lo ve el frontend (sin hash; `debe_cambiar_password` = flag temporal).
#[derive(Debug, Serialize)]
pub struct UsuarioSesionDto {
    pub id: i32,
    pub nombre: String,
    pub email: String,
    pub rol: String,
    pub familia_rol: String,
    pub debe_cambiar_password: bool,
}

#[derive(Debug, Serialize)]
pub struct SesionDto {
    pub usuario: UsuarioSesionDto,
}

fn cookie_sesion(estado: &AppEstado, sesion_id: Uuid) -> Cookie<'static> {
    let horas = estado.config.session_expiry_hours.max(1) as u64;
    Cookie::build((
        estado.config.session_cookie_name.clone(),
        sesion_id.to_string(),
    ))
    .http_only(true)
    .same_site(SameSite::Lax)
    .path("/")
    // FR-002a: sólo viaja por HTTPS en producción; en dev/local no.
    .secure(estado.config.es_produccion())
    .max_age(Duracion::hours(horas as i64))
    .build()
}

fn cookie_sesion_borrada(estado: &AppEstado) -> Cookie<'static> {
    Cookie::build((
        estado.config.session_cookie_name.clone(),
        String::new(),
    ))
    .http_only(true)
    .same_site(SameSite::Lax)
    .path("/")
    .secure(estado.config.es_produccion())
    .expires(OffsetDateTime::now_utc() - Duracion::days(1))
    .build()
}

/// POST /api/auth/login → 204 + Set-Cookie (o 401 genérico).
async fn login(
    State(estado): State<AppEstado>,
    jar: CookieJar,
    JsonValido(dto): JsonValido<LoginDto>,
) -> Result<(CookieJar, StatusCode), AppError> {
    if dto.email.trim().is_empty() || dto.password.is_empty() {
        return Err(AppError::Validacion(
            "Ingresá email y contraseña".into(),
        ));
    }
    let (sesion_id, _) = services::auth::login(
        &estado.pool,
        estado.reloj.as_ref(),
        &dto.email,
        &dto.password,
        estado.config.session_expiry_hours,
    )
    .await?;
    Ok((jar.add(cookie_sesion(&estado, sesion_id)), StatusCode::NO_CONTENT))
}

/// POST /api/auth/logout → 204 (idempotente) + borra la cookie.
async fn logout(
    State(estado): State<AppEstado>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    if let Some(cookie) = jar.get(&estado.config.session_cookie_name) {
        if let Ok(sesion_id) = Uuid::parse_str(cookie.value()) {
            services::auth::logout(&estado.pool, estado.reloj.as_ref(), sesion_id).await?;
        }
    }
    Ok((
        jar.remove(cookie_sesion_borrada(&estado)),
        StatusCode::NO_CONTENT,
    ))
}

/// GET /api/auth/sesion → usuario actual o 401.
async fn sesion(AuthUsuario(usuario): AuthUsuario) -> impl IntoResponse {
    Json(SesionDto {
        usuario: UsuarioSesionDto {
            id: usuario.id,
            nombre: usuario.nombre,
            email: usuario.email,
            rol: usuario.rol,
            familia_rol: usuario.familia_rol,
            debe_cambiar_password: usuario.password_temporal,
        },
    })
}

/// POST /api/auth/cambiar-password → 204; 422 si no cumple política.
async fn cambiar_password(
    State(estado): State<AppEstado>,
    AuthUsuario(_usuario): AuthUsuario,
    jar: CookieJar,
    JsonValido(dto): JsonValido<CambioPasswordDto>,
) -> Result<StatusCode, AppError> {
    let Some(cookie) = jar.get(&estado.config.session_cookie_name) else {
        return Err(AppError::NoAutenticado);
    };
    let sesion_id = Uuid::parse_str(cookie.value())
        .map_err(|_| AppError::NoAutenticado)?;
    services::auth::cambiar_password(
        &estado.pool,
        estado.reloj.as_ref(),
        sesion_id,
        &dto.password_actual,
        &dto.password_nueva,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
