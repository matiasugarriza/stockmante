//! Handlers de ventas (013).

use crate::app::AppEstado;
use crate::auth::extractors::{AuthUsuario, RequiereGestion};
use crate::error::AppError;
use crate::extractores::{JsonValido, PathValido, QueryValida};
use crate::models::venta::{VentaDetalle, ListaVentas};
use crate::services::ventas::{self, FiltrosVentas, NuevaVenta, NuevaVentaLinea};
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/", get(listar_ventas).post(crear_venta))
        .route("/{id}", get(obtener_venta))
        .route("/{id}/anular", axum::routing::post(anular_venta))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VentaLineaDto {
    pub producto_id: i32,
    pub cantidad: i32,
    /// No se acepta: el precio sale del catálogo. Se deserializa solo para rechazarlo (422).
    #[schema(value_type = Option<f64>)]
    pub precio_unitario: Option<Decimal>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VentaDto {
    pub deposito_id: i32,
    pub cliente_id: Option<i32>,
    pub medio_pago: String,
    #[schema(value_type = f64)]
    pub descuento: Option<Decimal>,
    pub observacion: Option<String>,
    pub lineas: Vec<VentaLineaDto>,
}

#[utoipa::path(post, path = "/api/ventas", tag = "ventas", summary = "Registrar venta",
    description = "OPERACION o GESTION. Las líneas del mismo producto se agregan; el precio sale del catálogo.",
    request_body = VentaDto, responses(
        (status = 201, description = "Venta registrada con `stock_resultante` por línea", body = VentaDetalle),
        (status = 401, description = "Sin sesión", body = crate::docs::ErrorRespuesta),
        (status = 404, description = "Depósito, cliente o producto inexistente", body = crate::docs::ErrorRespuesta),
        (status = 409, description = "`stock_insuficiente` (sobre la cantidad agregada)", body = crate::docs::ErrorRespuesta),
        (status = 422, description = "Precio enviado, producto inactivo, medio de pago, descuento o cantidad inválidos", body = crate::docs::ErrorRespuesta)
    ))]
pub async fn crear_venta(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, JsonValido(dto): JsonValido<VentaDto>,
) -> Result<(StatusCode, Json<VentaDetalle>), AppError> {
    if dto.lineas.iter().any(|l| l.precio_unitario.is_some()) {
        return Err(AppError::Validacion(
            "El precio unitario se toma del catálogo; no se puede enviar en la venta.".into(),
        ));
    }
    let nueva = NuevaVenta {
        deposito_id: dto.deposito_id,
        cliente_id: dto.cliente_id,
        medio_pago: dto.medio_pago,
        descuento: dto.descuento.unwrap_or(Decimal::ZERO),
        observacion: dto.observacion,
        lineas: dto.lineas.into_iter().map(|l| NuevaVentaLinea {
            producto_id: l.producto_id, cantidad: l.cantidad,
        }).collect(),
    };
    Ok((StatusCode::CREATED, Json(ventas::crear(&e.pool, nueva, &u).await?)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListarVentasQuery {
    pub deposito_id: Option<i32>,
    pub cliente_id: Option<i32>,
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

#[utoipa::path(get, path = "/api/ventas", tag = "ventas", summary = "Listar ventas",
    params(ListarVentasQuery), responses((status = 200, body = ListaVentas)))]
pub async fn listar_ventas(
    State(e): State<AppEstado>, _u: AuthUsuario, QueryValida(q): QueryValida<ListarVentasQuery>,
) -> Result<Json<ListaVentas>, AppError> {
    let f = FiltrosVentas {
        deposito_id: q.deposito_id,
        cliente_id: q.cliente_id,
        estado: q.estado,
        desde: parsear_fecha(q.desde.as_deref()),
        hasta: parsear_fecha(q.hasta.as_deref()),
        pagina: q.pagina,
        por_pagina: q.por_pagina,
    };
    Ok(Json(ventas::listar(&e.pool, f).await?))
}

#[utoipa::path(get, path = "/api/ventas/{id}", tag = "ventas", summary = "Detalle de venta",
    params(("id" = i32, Path)), responses((status = 200, body = VentaDetalle)))]
pub async fn obtener_venta(
    State(e): State<AppEstado>, _u: AuthUsuario, PathValido(id): PathValido<i32>,
) -> Result<Json<VentaDetalle>, AppError> {
    Ok(Json(ventas::obtener(&e.pool, id).await?))
}

#[utoipa::path(post, path = "/api/ventas/{id}/anular", tag = "ventas", summary = "Anular venta",
    description = "Solo GESTION. Repone stock con una `ENTRADA` (motivo `DEVOLUCION`).",
    params(("id" = i32, Path)), responses(
        (status = 200, description = "Venta anulada", body = VentaDetalle),
        (status = 403, description = "Requiere familia GESTION (`sin_permisos`)", body = crate::docs::ErrorRespuesta),
        (status = 404, description = "Venta inexistente", body = crate::docs::ErrorRespuesta),
        (status = 409, description = "`venta_ya_anulada`", body = crate::docs::ErrorRespuesta)
    ))]
pub async fn anular_venta(
    State(e): State<AppEstado>, RequiereGestion(u): RequiereGestion, PathValido(id): PathValido<i32>,
) -> Result<Json<VentaDetalle>, AppError> {
    Ok(Json(ventas::anular(&e.pool, id, &u).await?))
}
