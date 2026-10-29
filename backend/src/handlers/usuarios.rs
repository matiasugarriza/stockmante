//! Handlers de usuarios (US2, contracts/api.md): CRUD bajo familia GESTION.
//! Todas las rutas exigen sesión vigente + `RequiereGestion` (403 si OPERACION).

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
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
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar).post(crear))
        .route("/{id}", get(obtener).put(actualizar))
        .route("/{id}/desactivar", post(desactivar))
        .route("/{id}/reactivar", post(reactivar))
        .route("/{id}/password-temporal", post(password_temporal))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarQuery {
    /// `true`, `false` o `all` (ausente = todas).
    pub activo: Option<String>,
    /// Página (desde 1).
    pub pagina: Option<i64>,
    /// Tamaño de página (1–100).
    pub por_pagina: Option<i64>,
}

/// GET /api/usuarios?activo=&pagina=&por_pagina= → 200 lista paginada.
#[utoipa::path(
    get,
    path = "/api/usuarios",
    tag = "usuarios",
    summary = "Lista usuarios (GESTION)",
    params(ListarQuery),
    responses(
        (status = 200, description = "Página de usuarios", body = ListaUsuarios),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Filtros inválidos", body = ErrorRespuesta)
    )
)]
pub async fn listar(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    QueryValida(query): QueryValida<ListarQuery>,
) -> Result<Json<ListaUsuarios>, AppError> {
    let activo = super::parsear_activo(query.activo.as_deref(), None)?;
    let lista = usuarios::listar(&estado.pool, activo, query.pagina, query.por_pagina).await?;
    Ok(Json(lista))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CrearUsuarioDto {
    pub nombre: String,
    #[schema(example = "operario@stockmante.local")]
    pub email: String,
    #[schema(example = "OPERARIO")]
    pub rol: String,
}

/// POST /api/usuarios → 201 usuario (sin hash ni contraseña) + temporal por correo.
#[utoipa::path(
    post,
    path = "/api/usuarios",
    tag = "usuarios",
    summary = "Alta de usuario (GESTION)",
    description = "Genera una contraseña temporal y la envía por correo dentro de la transacción.",
    request_body = CrearUsuarioDto,
    responses(
        (status = 201, description = "Usuario creado", body = Usuario),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 409, description = "Email duplicado", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos", body = ErrorRespuesta)
    )
)]
pub async fn crear(
    State(estado): State<AppEstado>,
    admin: RequiereGestion,
    JsonValido(dto): JsonValido<CrearUsuarioDto>,
) -> Result<(StatusCode, Json<Usuario>), AppError> {
    let usuario = usuarios::crear(
        &estado.pool,
        estado.correo.as_ref(),
        admin.0.id,
        &dto.nombre,
        &dto.email,
        &dto.rol,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(usuario)))
}

/// GET /api/usuarios/{id} → 200 o 404.
#[utoipa::path(
    get,
    path = "/api/usuarios/{id}",
    tag = "usuarios",
    summary = "Detalle de usuario (GESTION)",
    params(("id" = i32, Path, description = "Id del usuario")),
    responses(
        (status = 200, description = "Usuario", body = Usuario),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Usuario inexistente", body = ErrorRespuesta)
    )
)]
pub async fn obtener(
    State(estado): State<AppEstado>,
    _admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Usuario>, AppError> {
    Ok(Json(usuarios::obtener(&estado.pool, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizarUsuarioDto {
    pub nombre: Option<String>,
    pub email: Option<String>,
    pub rol: Option<String>,
}

/// PUT /api/usuarios/{id} → 200 usuario actualizado.
#[utoipa::path(
    put,
    path = "/api/usuarios/{id}",
    tag = "usuarios",
    summary = "Edición parcial de usuario (GESTION)",
    description = "Cambiar el rol recalcula la familia; degradar al último gestor activo es 409.",
    params(("id" = i32, Path, description = "Id del usuario")),
    request_body = ActualizarUsuarioDto,
    responses(
        (status = 200, description = "Usuario actualizado", body = Usuario),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Usuario inexistente", body = ErrorRespuesta),
        (status = 409, description = "Email duplicado o último gestor", body = ErrorRespuesta),
        (status = 422, description = "Datos inválidos", body = ErrorRespuesta)
    )
)]
pub async fn actualizar(
    State(estado): State<AppEstado>,
    admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ActualizarUsuarioDto>,
) -> Result<Json<Usuario>, AppError> {
    let cambios = CambiosUsuario {
        nombre: dto.nombre,
        email: dto.email,
        rol: dto.rol,
    };
    Ok(Json(
        usuarios::actualizar(&estado.pool, id, cambios, admin.0.id).await?,
    ))
}

/// POST /api/usuarios/{id}/desactivar → 204 (revoca sesiones).
#[utoipa::path(
    post,
    path = "/api/usuarios/{id}/desactivar",
    tag = "usuarios",
    summary = "Baja lógica de usuario (GESTION)",
    description = "Desactiva la cuenta y revoca sus sesiones. No se puede desactivar al último gestor.",
    params(("id" = i32, Path, description = "Id del usuario")),
    responses(
        (status = 204, description = "Usuario desactivado"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Usuario inexistente", body = ErrorRespuesta),
        (status = 409, description = "Último gestor activo", body = ErrorRespuesta)
    )
)]
pub async fn desactivar(
    State(estado): State<AppEstado>,
    admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    usuarios::desactivar(&estado.pool, estado.reloj.as_ref(), id, admin.0.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/usuarios/{id}/reactivar → 204.
#[utoipa::path(
    post,
    path = "/api/usuarios/{id}/reactivar",
    tag = "usuarios",
    summary = "Reactiva un usuario (GESTION)",
    params(("id" = i32, Path, description = "Id del usuario")),
    responses(
        (status = 204, description = "Usuario reactivado"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Usuario inexistente", body = ErrorRespuesta)
    )
)]
pub async fn reactivar(
    State(estado): State<AppEstado>,
    admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    usuarios::reactivar(&estado.pool, estado.reloj.as_ref(), id, admin.0.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/usuarios/{id}/password-temporal → 204 (nueva temporal por correo).
#[utoipa::path(
    post,
    path = "/api/usuarios/{id}/password-temporal",
    tag = "usuarios",
    summary = "Regenera la contraseña temporal (GESTION)",
    description = "Actualiza el hash, marca cambio obligatorio y revoca las sesiones antes de enviar el correo.",
    params(("id" = i32, Path, description = "Id del usuario")),
    responses(
        (status = 204, description = "Contraseña temporal regenerada y enviada"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Usuario inexistente", body = ErrorRespuesta),
        (status = 500, description = "No se pudo enviar el correo", body = ErrorRespuesta)
    )
)]
pub async fn password_temporal(
    State(estado): State<AppEstado>,
    admin: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    usuarios::regenerar_password_temporal(
        &estado.pool,
        estado.reloj.as_ref(),
        estado.correo.as_ref(),
        id,
        admin.0.id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
