//! Handlers de reportes (014). Toda la superficie exige GESTION (SC-031).
//!
//! `formato=json|csv` (default json) y, en ventas, `agrupar=dia|producto`; otro valor es
//! 422 con un mensaje que nombra el parámetro (R8). El CSV usa `services::reportes::csv`.

use crate::app::AppEstado;
use crate::auth::extractors::RequiereGestion;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::QueryValida;
use crate::services::reportes::{self, csv::a_csv, rango};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{FixedOffset, NaiveDate};
use serde::Deserialize;
use utoipa::IntoParams;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/kardex", get(kardex))
        .route("/stock-valorizado", get(stock_valorizado))
        .route("/rotacion", get(rotacion))
        .route("/ventas", get(ventas))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Formato {
    Json,
    Csv,
}

fn parsear_formato(valor: Option<&str>) -> Result<Formato, AppError> {
    match valor.map(|v| v.trim().to_lowercase()).as_deref() {
        None | Some("") | Some("json") => Ok(Formato::Json),
        Some("csv") => Ok(Formato::Csv),
        Some(_) => Err(AppError::Validacion(
            "El parámetro formato acepta json o csv.".into(),
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agrupar {
    Dia,
    Producto,
}

fn parsear_agrupar(valor: Option<&str>) -> Result<Agrupar, AppError> {
    match valor.map(|v| v.trim().to_lowercase()).as_deref() {
        None | Some("") | Some("dia") => Ok(Agrupar::Dia),
        Some("producto") => Ok(Agrupar::Producto),
        Some(_) => Err(AppError::Validacion(
            "El parámetro agrupar acepta dia o producto.".into(),
        )),
    }
}

fn nombre_archivo(reporte: &str, desde: Option<NaiveDate>, hasta: Option<NaiveDate>) -> String {
    match (desde, hasta) {
        (Some(d), Some(h)) => format!("{reporte}_{d}_{h}.csv"),
        (Some(d), None) => format!("{reporte}_{d}.csv"),
        (None, Some(h)) => format!("{reporte}_{h}.csv"),
        (None, None) => format!("{reporte}.csv"),
    }
}

fn respuesta_csv(archivo: String, cuerpo: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{archivo}\""),
            ),
        ],
        cuerpo,
    )
        .into_response()
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct KardexQuery {
    pub producto_id: i32,
    /// Opcional: limita saldo inicial y líneas a ese depósito.
    pub deposito_id: Option<i32>,
    /// Día local `AAAA-MM-DD` (inclusivo).
    pub desde: Option<String>,
    /// Día local `AAAA-MM-DD` (inclusivo).
    pub hasta: Option<String>,
    /// `json` (default) o `csv`.
    pub formato: Option<String>,
}

#[utoipa::path(get, path = "/api/reportes/kardex", tag = "reportes", summary = "Kardex de producto (GESTION)",
    params(KardexQuery),
    responses(
        (status = 200, description = "Kardex en JSON, o `text/csv` con `formato=csv`", body = reportes::KardexReporte),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 404, description = "Producto inexistente", body = ErrorRespuesta),
        (status = 422, description = "Fechas o formato inválidos", body = ErrorRespuesta)
    ))]
pub async fn kardex(
    State(e): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(q): QueryValida<KardexQuery>,
) -> Result<Response, AppError> {
    let formato = parsear_formato(q.formato.as_deref())?;
    let desde = rango::parsear_dia(q.desde.as_deref(), "desde")?;
    let hasta = rango::parsear_dia(q.hasta.as_deref(), "hasta")?;
    let rep = reportes::kardex(&e.pool, q.producto_id, q.deposito_id, desde, hasta).await?;
    if formato == Formato::Json {
        return Ok(Json(rep).into_response());
    }
    let local = FixedOffset::west_opt(3 * 3600);
    let filas = rep.lineas.iter().map(|l| {
        let fecha = match local {
            Some(o) => l.creado_en.with_timezone(&o).to_rfc3339(),
            None => l.creado_en.to_rfc3339(),
        };
        vec![
            fecha,
            l.tipo.clone(),
            l.motivo.clone(),
            l.deposito_nombre.clone(),
            l.cantidad.to_string(),
            l.saldo.to_string(),
        ]
    });
    let cuerpo = a_csv(&["fecha", "tipo", "motivo", "deposito", "cantidad", "saldo"], filas);
    Ok(respuesta_csv(nombre_archivo("kardex", desde, hasta), cuerpo))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct FormatoQuery {
    /// `json` (default) o `csv`.
    pub formato: Option<String>,
}

#[utoipa::path(get, path = "/api/reportes/stock-valorizado", tag = "reportes", summary = "Stock valorizado (GESTION)",
    params(FormatoQuery),
    responses(
        (status = 200, description = "Stock valorizado en JSON o `text/csv`", body = reportes::StockValorizadoReporte),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Formato inválido", body = ErrorRespuesta)
    ))]
pub async fn stock_valorizado(
    State(e): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(q): QueryValida<FormatoQuery>,
) -> Result<Response, AppError> {
    let formato = parsear_formato(q.formato.as_deref())?;
    let rep = reportes::stock_valorizado(&e.pool).await?;
    if formato == Formato::Json {
        return Ok(Json(rep).into_response());
    }
    let filas = rep.items.iter().map(|i| {
        vec![
            i.nombre.clone(),
            i.sku.clone(),
            i.cantidad.to_string(),
            i.costo_unitario.to_string(),
            i.valor.to_string(),
        ]
    });
    let cuerpo = a_csv(&["producto", "sku", "cantidad", "costo_unitario", "valor"], filas);
    Ok(respuesta_csv("stock-valorizado.csv".into(), cuerpo))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RangoQuery {
    /// Día local `AAAA-MM-DD` (obligatorio).
    pub desde: Option<String>,
    /// Día local `AAAA-MM-DD` (obligatorio).
    pub hasta: Option<String>,
    /// `json` (default) o `csv`.
    pub formato: Option<String>,
    /// Solo ventas: `dia` (default) o `producto` (afecta el CSV).
    pub agrupar: Option<String>,
}

#[utoipa::path(get, path = "/api/reportes/rotacion", tag = "reportes", summary = "Rotación por producto (GESTION)",
    params(RangoQuery),
    responses(
        (status = 200, description = "Rotación en JSON o `text/csv`", body = reportes::RotacionReporte),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Falta el rango, fechas o formato inválidos", body = ErrorRespuesta)
    ))]
pub async fn rotacion(
    State(e): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(q): QueryValida<RangoQuery>,
) -> Result<Response, AppError> {
    let formato = parsear_formato(q.formato.as_deref())?;
    let desde = rango::parsear_dia(q.desde.as_deref(), "desde")?;
    let hasta = rango::parsear_dia(q.hasta.as_deref(), "hasta")?;
    let rep = reportes::rotacion(&e.pool, desde, hasta).await?;
    if formato == Formato::Json {
        return Ok(Json(rep).into_response());
    }
    let filas = rep.items.iter().map(|i| {
        vec![
            i.nombre.clone(),
            i.unidades_salidas.to_string(),
            i.stock_inicial.to_string(),
            i.stock_final.to_string(),
            format!("{:.2}", i.stock_promedio),
            format!("{:.4}", i.indice_rotacion),
        ]
    });
    let cuerpo = a_csv(
        &["producto", "unidades_salidas", "stock_inicial", "stock_final", "stock_promedio", "indice_rotacion"],
        filas,
    );
    Ok(respuesta_csv(nombre_archivo("rotacion", Some(rep.desde), Some(rep.hasta)), cuerpo))
}

#[utoipa::path(get, path = "/api/reportes/ventas", tag = "reportes", summary = "Resumen de ventas (GESTION)",
    params(RangoQuery),
    responses(
        (status = 200, description = "Ventas en JSON o `text/csv` (por día o por producto)", body = reportes::VentasReporte),
        (status = 403, description = "Requiere familia GESTION", body = ErrorRespuesta),
        (status = 422, description = "Falta el rango, fechas, formato o agrupar inválidos", body = ErrorRespuesta)
    ))]
pub async fn ventas(
    State(e): State<AppEstado>,
    _gestion: RequiereGestion,
    QueryValida(q): QueryValida<RangoQuery>,
) -> Result<Response, AppError> {
    let formato = parsear_formato(q.formato.as_deref())?;
    let agrupar = parsear_agrupar(q.agrupar.as_deref())?;
    let desde = rango::parsear_dia(q.desde.as_deref(), "desde")?;
    let hasta = rango::parsear_dia(q.hasta.as_deref(), "hasta")?;
    let rep = reportes::ventas(&e.pool, desde, hasta).await?;
    if formato == Formato::Json {
        return Ok(Json(rep).into_response());
    }
    let archivo = nombre_archivo("ventas", Some(rep.desde), Some(rep.hasta));
    let cuerpo = match agrupar {
        Agrupar::Dia => a_csv(
            &["dia", "cantidad_ventas", "total"],
            rep.por_dia
                .iter()
                .map(|d| vec![d.dia.clone(), d.cantidad_ventas.to_string(), d.total.to_string()]),
        ),
        Agrupar::Producto => a_csv(
            &["producto", "unidades", "total"],
            rep.por_producto
                .iter()
                .map(|p| vec![p.nombre.clone(), p.unidades.to_string(), p.total.to_string()]),
        ),
    };
    Ok(respuesta_csv(archivo, cuerpo))
}
