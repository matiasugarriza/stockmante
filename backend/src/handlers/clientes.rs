//! Handlers de clientes (013).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::venta::{Cliente, ListaClientes};
use crate::services::clientes::{self, FiltrosClientes, NuevoCliente};
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar_clientes).post(crear_cliente))
        .route("/{id}", get(obtener_cliente).put(actualizar_cliente))
        .route("/{id}/activar", axum::routing::post(activar_cliente))
        .route("/{id}/desactivar", axum::routing::post(desactivar_cliente))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClienteDto {
    pub nombre: String,
    pub documento: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub direccion: Option<String>,
}

#[utoipa::path(post, path = "/api/clientes", tag = "clientes", summary = "Crear cliente",
    request_body = ClienteDto, responses((status = 201, body = Cliente)))]
pub async fn crear_cliente(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, JsonValido(dto): JsonValido<ClienteDto>,
) -> Result<(StatusCode, Json<Cliente>), AppError> {
    let nuevo = NuevoCliente {
        nombre: dto.nombre, documento: dto.documento, email: dto.email, telefono: dto.telefono, direccion: dto.direccion,
    };
    Ok((StatusCode::CREATED, Json(clientes::crear(&e.pool, nuevo, u.id).await?)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarClientesQuery {
    pub activo: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

#[utoipa::path(get, path = "/api/clientes", tag = "clientes", summary = "Listar clientes",
    params(ListarClientesQuery), responses((status = 200, body = ListaClientes)))]
pub async fn listar_clientes(
    State(e): State<AppEstado>, _u: AuthUsuario, QueryValida(q): QueryValida<ListarClientesQuery>,
) -> Result<Json<ListaClientes>, AppError> {
    let activo = crate::handlers::parsear_activo(q.activo.as_deref(), None)?;
    let f = FiltrosClientes { activo, pagina: q.pagina, por_pagina: q.por_pagina };
    Ok(Json(clientes::listar(&e.pool, f).await?))
}

#[utoipa::path(get, path = "/api/clientes/{id}", tag = "clientes", summary = "Obtener cliente",
    params(("id" = i32, Path)), responses((status = 200, body = Cliente)))]
pub async fn obtener_cliente(
    State(e): State<AppEstado>, _u: AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<Cliente>, AppError> {
    Ok(Json(clientes::obtener(&e.pool, id).await?))
}

#[utoipa::path(put, path = "/api/clientes/{id}", tag = "clientes", summary = "Actualizar cliente",
    params(("id" = i32, Path)), request_body = ClienteDto,
    responses((status = 200, body = Cliente)))]
pub async fn actualizar_cliente(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<ClienteDto>,
) -> Result<Json<Cliente>, AppError> {
    let nuevo = NuevoCliente {
        nombre: dto.nombre, documento: dto.documento, email: dto.email, telefono: dto.telefono, direccion: dto.direccion,
    };
    Ok(Json(clientes::actualizar(&e.pool, id, nuevo, u.id).await?))
}

async fn toggle(e: &AppEstado, id: i32, activo: bool, u: i32) -> Result<Cliente, AppError> {
    clientes::set_activo(&e.pool, id, activo, u).await
}

#[utoipa::path(post, path = "/api/clientes/{id}/activar", tag = "clientes", summary = "Reactivar cliente",
    params(("id" = i32, Path)), responses((status = 200, body = Cliente)))]
pub async fn activar_cliente(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<Cliente>, AppError> {
    Ok(Json(toggle(&e, id, true, u.id).await?))
}

#[utoipa::path(post, path = "/api/clientes/{id}/desactivar", tag = "clientes", summary = "Desactivar cliente",
    params(("id" = i32, Path)), responses((status = 200, body = Cliente)))]
pub async fn desactivar_cliente(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<Cliente>, AppError> {
    Ok(Json(toggle(&e, id, false, u.id).await?))
}
