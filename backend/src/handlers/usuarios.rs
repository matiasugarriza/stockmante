//! Handlers de usuarios (US2, contracts/api.md): CRUD bajo familia GESTION.
//! Todas las rutas exigen sesión vigente + `RequiereGestion` (403 si OPERACION).

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::usuario::Usuario;
use crate::services::usuarios::{self, CambiosUsuario, ListaUsuarios};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/{id}", get(obtener).put(actualizar))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
        .route("/{id}/password-temporal", post(password_temporal))
}

#[derive(Debug, Deserialize)]
pub struct ListarQuery {
    pub activo: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// GET /api/usuarios?activo=&pagina=&por_pagina= → 200 lista paginada.
async fn listar(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    QueryValida(query): QueryValida<ListarQuery>,
) -> Result<Json<ListaUsuarios>, AppError> {
    let activo = super::parsear_activo(query.activo.as_deref(), None)?;
    let lista = usuarios::listar(&estado.pool, activo, query.pagina, query.por_pagina).await?;
    Ok(Json(lista))
}

#[derive(Debug, Deserialize)]
pub struct CrearUsuarioDto {
    pub nombre: String,
    pub email: String,
    pub rol: String,
}

/// POST /api/usuarios → 201 usuario (sin hash ni contraseña) + temporal por correo.
async fn crear(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    JsonValido(dto): JsonValido<CrearUsuarioDto>,
) -> Result<(StatusCode, Json<Usuario>), AppError> {
    let usuario = usuarios::crear(
        &estado.pool,
        estado.correo.as_ref(),
        &dto.nombre,
        &dto.email,
        &dto.rol,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(usuario)))
}

/// GET /api/usuarios/{id} → 200 o 404.
async fn obtener(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Usuario>, AppError> {
    Ok(Json(usuarios::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct ActualizarUsuarioDto {
    pub nombre: Option<String>,
    pub email: Option<String>,
    pub rol: Option<String>,
}

/// PUT /api/usuarios/{id} → 200 usuario actualizado.
async fn actualizar(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ActualizarUsuarioDto>,
) -> Result<Json<Usuario>, AppError> {
    let cambios = CambiosUsuario {
        nombre: dto.nombre,
        email: dto.email,
        rol: dto.rol,
    };
    Ok(Json(usuarios::actualizar(&estado.pool, id, cambios).await?))
}

/// POST /api/usuarios/{id}/desactivar → 204 (revoca sesiones).
async fn desactivar(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    usuarios::desactivar(&estado.pool, estado.reloj.as_ref(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/usuarios/{id}/reactivar → 204.
async fn reactivar(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    usuarios::reactivar(&estado.pool, estado.reloj.as_ref(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/usuarios/{id}/password-temporal → 204 (nueva temporal por correo).
async fn password_temporal(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    usuarios::regenerar_password_temporal(
        &estado.pool,
        estado.reloj.as_ref(),
        estado.correo.as_ref(),
        id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
