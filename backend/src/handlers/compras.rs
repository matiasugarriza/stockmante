//! Handlers de compras (012).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::proveedor::{CompraDetalle, ListaCompras};
use crate::services::compras::{self, FiltrosCompras, NuevaCompra, NuevaCompraLinea};
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar_compras).post(crear_compra))
        .route("/{id}", get(obtener_compra).put(actualizar_compra).delete(eliminar_compra))
        .route("/{id}/confirmar", axum::routing::post(confirmar_compra))
        .route("/{id}/anular", axum::routing::post(anular_compra))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompraLineaDto {
    pub producto_id: i32,
    pub cantidad: i32,
    #[schema(value_type = f64)]
    pub costo_unitario: Decimal,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompraDto {
    pub proveedor_id: i32,
    pub deposito_id: i32,
    pub numero: Option<String>,
    pub observacion: Option<String>,
    pub lineas: Vec<CompraLineaDto>,
}

fn dto_a_nueva(dto: CompraDto) -> NuevaCompra {
    NuevaCompra {
        proveedor_id: dto.proveedor_id,
        deposito_id: dto.deposito_id,
        numero: dto.numero,
        observacion: dto.observacion,
        lineas: dto.lineas.into_iter().map(|l| NuevaCompraLinea {
            producto_id: l.producto_id, cantidad: l.cantidad, costo_unitario: l.costo_unitario,
        }).collect(),
    }
}

#[utoipa::path(post, path = "/api/compras", tag = "compras", summary = "Crear compra en borrador",
    request_body = CompraDto,
    responses((status = 201, body = CompraDetalle)))]
pub async fn crear_compra(
    State(e): State<AppEstado>, AuthUsuario(usuario): AuthUsuario, JsonValido(dto): JsonValido<CompraDto>,
) -> Result<(StatusCode, Json<CompraDetalle>), AppError> {
    let c = compras::crear(&e.pool, dto_a_nueva(dto), &usuario).await?;
    Ok((StatusCode::CREATED, Json(c)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarComprasQuery {
    pub proveedor_id: Option<i32>,
    pub estado: Option<String>,
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

fn parsear_fecha(v: Option<&str>) -> Option<DateTime<Utc>> {
    let t = v.map(str::trim).filter(|s| !s.is_empty())?;
    if let Ok(d) = DateTime::parse_from_rfc3339(t) { return Some(d.with_timezone(&Utc)); }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d") {
        return d.and_hms_opt(0,0,0).map(|x| x.and_utc());
    }
    None
}

#[utoipa::path(get, path = "/api/compras", tag = "compras", summary = "Listar compras",
    params(ListarComprasQuery),
    responses((status = 200, body = ListaCompras)))]
pub async fn listar_compras(
    State(e): State<AppEstado>, _u: AuthUsuario, QueryValida(q): QueryValida<ListarComprasQuery>,
) -> Result<Json<ListaCompras>, AppError> {
    let f = FiltrosCompras {
        proveedor_id: q.proveedor_id,
        estado: q.estado,
        desde: parsear_fecha(q.desde.as_deref()),
        hasta: parsear_fecha(q.hasta.as_deref()),
        pagina: q.pagina,
        por_pagina: q.por_pagina,
    };
    Ok(Json(compras::listar(&e.pool, f).await?))
}

#[utoipa::path(get, path = "/api/compras/{id}", tag = "compras", summary = "Detalle de compra",
    params(("id" = i32, Path)),
    responses((status = 200, body = CompraDetalle)))]
pub async fn obtener_compra(
    State(e): State<AppEstado>, _u: AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<CompraDetalle>, AppError> {
    Ok(Json(compras::obtener(&e.pool, id).await?))
}

#[utoipa::path(put, path = "/api/compras/{id}", tag = "compras", summary = "Editar compra en borrador",
    params(("id" = i32, Path)), request_body = CompraDto,
    responses((status = 200, body = CompraDetalle)))]
pub async fn actualizar_compra(
    State(e): State<AppEstado>, AuthUsuario(usuario): AuthUsuario, PathValido(id): PathValido<i32>,
    JsonValido(dto): JsonValido<CompraDto>,
) -> Result<Json<CompraDetalle>, AppError> {
    Ok(Json(compras::actualizar(&e.pool, id, dto_a_nueva(dto), usuario.id).await?))
}

#[utoipa::path(delete, path = "/api/compras/{id}", tag = "compras", summary = "Eliminar compra en borrador",
    params(("id" = i32, Path)),
    responses((status = 204, description = "Eliminada")))]
pub async fn eliminar_compra(
    State(e): State<AppEstado>, _u: AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<StatusCode, AppError> {
    compras::eliminar(&e.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/compras/{id}/confirmar", tag = "compras", summary = "Confirmar recepción",
    params(("id" = i32, Path)),
    responses((status = 200, body = CompraDetalle)))]
pub async fn confirmar_compra(
    State(e): State<AppEstado>, AuthUsuario(usuario): AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<CompraDetalle>, AppError> {
    Ok(Json(compras::confirmar(&e.pool, id, &usuario).await?))
}

#[utoipa::path(post, path = "/api/compras/{id}/anular", tag = "compras", summary = "Anular compra confirmada",
    params(("id" = i32, Path)),
    responses((status = 200, body = CompraDetalle)))]
pub async fn anular_compra(
    State(e): State<AppEstado>, AuthUsuario(usuario): AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<CompraDetalle>, AppError> {
    Ok(Json(compras::anular(&e.pool, id, &usuario).await?))
}
