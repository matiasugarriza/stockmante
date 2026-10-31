//! Handlers de reportes (014).

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::QueryValida;
use crate::services::reportes;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::IntoParams;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/kardex", get(kardex))
        .route("/stock-valorizado", get(stock_valorizado))
        .route("/rotacion", get(rotacion))
        .route("/ventas", get(ventas))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct KardexQuery {
    pub producto_id: i32,
    pub deposito_id: Option<i32>,
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub formato: Option<String>,
}

fn parsear_fecha(v: Option<&str>) -> Option<DateTime<Utc>> {
    let t = v.map(str::trim).filter(|s| !s.is_empty())?;
    if let Ok(d) = DateTime::parse_from_rfc3339(t) { return Some(d.with_timezone(&Utc)); }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d") {
        return d.and_hms_opt(0,0,0).map(|x| x.and_utc());
    }
    None
}

#[utoipa::path(get, path = "/api/reportes/kardex", tag = "reportes", summary = "Kardex de producto",
    params(KardexQuery),
    responses((status = 200, body = reportes::KardexReporte)))]
pub async fn kardex(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, QueryValida(q): QueryValida<KardexQuery>,
) -> Result<impl IntoResponse, AppError> {
    let desde = parsear_fecha(q.desde.as_deref());
    let hasta = parsear_fecha(q.hasta.as_deref());
    let rep = reportes::kardex(&e.pool, q.producto_id, q.deposito_id, desde, hasta).await?;
    if q.formato.as_deref() == Some("csv") {
        let mut csv = String::from("fecha,tipo,motivo,deposito,cantidad,saldo\n");
        for l in &rep.lineas {
            csv.push_str(&format!("{},{},{},{},{},{}\n", l.creado_en, l.tipo, l.motivo, l.deposito_nombre, l.cantidad, l.saldo));
        }
        return Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/csv; charset=utf-8")], csv).into_response());
    }
    Ok(Json(rep).into_response())
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RangoQuery {
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub formato: Option<String>,
}

#[utoipa::path(get, path = "/api/reportes/stock-valorizado", tag = "reportes", summary = "Stock valorizado",
    responses((status = 200, body = reportes::StockValorizadoReporte)))]
pub async fn stock_valorizado(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario,
) -> Result<Json<reportes::StockValorizadoReporte>, AppError> {
    if u.familia_rol != "GESTION" { return Err(AppError::SinPermisos); }
    Ok(Json(reportes::stock_valorizado(&e.pool).await?))
}

#[utoipa::path(get, path = "/api/reportes/rotacion", tag = "reportes", summary = "Rotación por producto",
    params(RangoQuery),
    responses((status = 200, body = reportes::RotacionReporte)))]
pub async fn rotacion(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, QueryValida(q): QueryValida<RangoQuery>,
) -> Result<impl IntoResponse, AppError> {
    let desde = parsear_fecha(q.desde.as_deref());
    let hasta = parsear_fecha(q.hasta.as_deref());
    let rep = reportes::rotacion(&e.pool, desde, hasta).await?;
    if q.formato.as_deref() == Some("csv") {
        let mut csv = String::from("producto,unidades_salidas,stock_inicial,stock_final,stock_promedio,indice_rotacion\n");
        for i in &rep.items {
            csv.push_str(&format!("{},{},{},{},{:.2},{:.4}\n", i.nombre, i.unidades_salidas, i.stock_inicial, i.stock_final, i.stock_promedio, i.indice_rotacion));
        }
        return Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/csv; charset=utf-8")], csv).into_response());
    }
    Ok(Json(rep).into_response())
}

#[utoipa::path(get, path = "/api/reportes/ventas", tag = "reportes", summary = "Resumen de ventas",
    params(RangoQuery),
    responses((status = 200, body = reportes::VentasReporte)))]
pub async fn ventas(
    State(e): State<AppEstado>, AuthUsuario(u): AuthUsuario, QueryValida(q): QueryValida<RangoQuery>,
) -> Result<impl IntoResponse, AppError> {
    let desde = parsear_fecha(q.desde.as_deref());
    let hasta = parsear_fecha(q.hasta.as_deref());
    let rep = reportes::ventas(&e.pool, desde, hasta).await?;
    if q.formato.as_deref() == Some("csv") {
        let mut csv = String::from("dia,cantidad_ventas,total\n");
        for d in &rep.por_dia {
            csv.push_str(&format!("{},{},{}\n", d.dia, d.cantidad_ventas, d.total));
        }
        return Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/csv; charset=utf-8")], csv).into_response());
    }
    Ok(Json(rep).into_response())
}
