//! Handlers de proveedores y compras (012).

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::proveedor::{ListaProveedores, Proveedor};
use crate::models::usuario::Usuario;
use crate::services::proveedores::{self, FiltrosProveedores, NuevoProveedor};
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar_proveedores).post(crear_proveedor))
        .route("/{id}", get(obtener_proveedor).put(actualizar_proveedor))
        .route("/{id}/activar", axum::routing::post(activar_proveedor))
        .route("/{id}/desactivar", axum::routing::post(desactivar_proveedor))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProveedorDto {
    pub nombre: String,
    pub cuit: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub direccion: Option<String>,
}

#[utoipa::path(post, path = "/api/proveedores", tag = "proveedores", summary = "Crear proveedor",
    request_body = ProveedorDto,
    responses((status = 201, body = Proveedor), (status = 409, body = ErrorRespuesta)))]
pub async fn crear_proveedor(
    State(e): State<AppEstado>,
    RequiereGestion(usuario): RequiereGestion,
    JsonValido(dto): JsonValido<ProveedorDto>,
) -> Result<(StatusCode, Json<Proveedor>), AppError> {
    let nuevo = NuevoProveedor {
        nombre: dto.nombre, cuit: dto.cuit, email: dto.email, telefono: dto.telefono, direccion: dto.direccion,
    };
    let p = proveedores::crear(&e.pool, nuevo, usuario.id).await?;
    Ok((StatusCode::CREATED, Json(p)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarProveedoresQuery {
    pub activo: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

#[utoipa::path(get, path = "/api/proveedores", tag = "proveedores", summary = "Listar proveedores",
    params(ListarProveedoresQuery),
    responses((status = 200, body = ListaProveedores)))]
pub async fn listar_proveedores(
    State(e): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(q): QueryValida<ListarProveedoresQuery>,
) -> Result<Json<ListaProveedores>, AppError> {
    let activo = crate::handlers::parsear_activo(q.activo.as_deref(), None)?;
    let f = FiltrosProveedores { activo, pagina: q.pagina, por_pagina: q.por_pagina };
    Ok(Json(proveedores::listar(&e.pool, f).await?))
}

#[utoipa::path(get, path = "/api/proveedores/{id}", tag = "proveedores", summary = "Obtener proveedor",
    params(("id" = i32, Path)),
    responses((status = 200, body = Proveedor), (status = 404, body = ErrorRespuesta)))]
pub async fn obtener_proveedor(
    State(e): State<AppEstado>,
    _gestion: RequiereGestion,
    PathValido(id): PathValido<i32>,
) -> Result<Json<Proveedor>, AppError> {
    Ok(Json(proveedores::obtener(&e.pool, id).await?))
}

#[utoipa::path(put, path = "/api/proveedores/{id}", tag = "proveedores", summary = "Actualizar proveedor",
    params(("id" = i32, Path)), request_body = ProveedorDto,
    responses((status = 200, body = Proveedor)))]
pub async fn actualizar_proveedor(
    State(e): State<AppEstado>,
    RequiereGestion(usuario): RequiereGestion,
    PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ProveedorDto>,
) -> Result<Json<Proveedor>, AppError> {
    let nuevo = NuevoProveedor {
        nombre: dto.nombre, cuit: dto.cuit, email: dto.email, telefono: dto.telefono, direccion: dto.direccion,
    };
    Ok(Json(proveedores::actualizar(&e.pool, id, nuevo, usuario.id).await?))
}

async fn toggle(e: &AppEstado, id: i32, activo: bool, u: i32) -> Result<Proveedor, AppError> {
    proveedores::set_activo(&e.pool, id, activo, u).await
}

#[utoipa::path(post, path = "/api/proveedores/{id}/activar", tag = "proveedores", summary = "Reactivar proveedor",
    params(("id" = i32, Path)), responses((status = 200, body = Proveedor)))]
pub async fn activar_proveedor(
    State(e): State<AppEstado>, RequiereGestion(usuario): RequiereGestion, PathValido(id): PathValido<i32>,
) -> Result<Json<Proveedor>, AppError> {
    Ok(Json(toggle(&e, id, true, usuario.id).await?))
}

#[utoipa::path(post, path = "/api/proveedores/{id}/desactivar", tag = "proveedores", summary = "Desactivar proveedor",
    params(("id" = i32, Path)), responses((status = 200, body = Proveedor)))]
pub async fn desactivar_proveedor(
    State(e): State<AppEstado>, RequiereGestion(usuario): RequiereGestion, PathValido(id): PathValido<i32>,
) -> Result<Json<Proveedor>, AppError> {
    Ok(Json(toggle(&e, id, false, usuario.id).await?))
}
