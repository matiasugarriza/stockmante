//! Modelos y reglas puras de proveedores y compras (012).

use crate::error::AppError;
use serde::Serialize;
use utoipa::ToSchema;

pub const NOMBRE_MAX: usize = 120;
pub const EMAIL_MAX: usize = 200;
pub const TELEFONO_MAX: usize = 60;
pub const DIRECCION_MAX: usize = 500;
pub const OBSERVACION_MAX: usize = 500;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Proveedor {
    pub id: i32,
    pub nombre: String,
    pub cuit: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub direccion: Option<String>,
    pub activo: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaProveedores {
    pub items: Vec<Proveedor>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct CompraLinea {
    pub id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: Option<String>,
    pub cantidad: i32,
    #[schema(value_type = f64)]
    pub costo_unitario: rust_decimal::Decimal,
    #[schema(value_type = f64)]
    pub subtotal: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Compra {
    pub id: i32,
    pub proveedor_id: i32,
    pub proveedor_nombre: String,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub numero: Option<String>,
    pub estado: String,
    pub observacion: Option<String>,
    #[schema(value_type = f64)]
    pub total_costo: rust_decimal::Decimal,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub creado_en: chrono::DateTime<chrono::Utc>,
    pub confirmado_en: Option<chrono::DateTime<chrono::Utc>>,
    pub anulado_en: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompraDetalle {
    pub compra: Compra,
    pub lineas: Vec<CompraLinea>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaCompras {
    pub items: Vec<Compra>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

pub fn normalizar_nombre(valor: &str) -> String { valor.trim().to_string() }
pub fn validar_nombre(valor: &str) -> Result<String, AppError> {
    let n = normalizar_nombre(valor);
    let l = n.chars().count();
    if l == 0 { return Err(AppError::Validacion("El nombre del proveedor es obligatorio.".into())); }
    if l > NOMBRE_MAX { return Err(AppError::Validacion(format!("El nombre no puede superar los {NOMBRE_MAX} caracteres."))); }
    Ok(n)
}
pub fn validar_cuit(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match valor.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(c) => {
            if c.chars().count() != 11 || !c.chars().all(|ch| ch.is_ascii_digit()) {
                return Err(AppError::Validacion("El CUIT debe tener 11 dígitos.".into()));
            }
            Ok(Some(c.to_string()))
        }
    }
}
pub fn normalizar_opcional(valor: Option<&str>, max: usize, campo: &str) -> Result<Option<String>, AppError> {
    match valor.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) => {
            if v.chars().count() > max {
                return Err(AppError::Validacion(format!("{campo} no puede superar los {max} caracteres.")));
            }
            Ok(Some(v.to_string()))
        }
    }
}
pub fn validar_observacion(valor: Option<&str>) -> Result<Option<String>, AppError> {
    normalizar_opcional(valor, OBSERVACION_MAX, "La observación")
}

/// Tope de `compras.numero` (VARCHAR(40)).
pub const NUMERO_MAX: usize = 40;

/// Nº de factura o remito del proveedor: opcional, hasta 40 caracteres.
pub fn validar_numero(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match valor.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) if v.chars().count() > NUMERO_MAX => Err(AppError::Validacion(format!(
            "El número de factura o remito no puede superar los {NUMERO_MAX} caracteres."
        ))),
        Some(v) => Ok(Some(v.to_string())),
    }
}

/// `costo_unitario` entre 0 y el máximo de `NUMERIC(12,2)`.
pub fn validar_costo(valor: rust_decimal::Decimal) -> Result<(), AppError> {
    let maximo = rust_decimal::Decimal::new(999_999_999_999, 2);
    if valor < rust_decimal::Decimal::ZERO {
        return Err(AppError::Validacion(
            "El costo unitario no puede ser negativo.".into(),
        ));
    }
    if valor > maximo {
        return Err(AppError::Validacion(
            "El costo unitario no puede superar 9.999.999.999,99.".into(),
        ));
    }
    Ok(())
}

/// Filtro `estado` del listado: vacío = todos; otro valor → 422.
pub fn validar_estado(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match valor.map(|v| v.trim().to_uppercase()).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(e) if ["BORRADOR", "CONFIRMADA", "ANULADA"].contains(&e.as_str()) => Ok(Some(e)),
        Some(_) => Err(AppError::Validacion(
            "El estado debe ser BORRADOR, CONFIRMADA o ANULADA.".into(),
        )),
    }
}

/// Fecha de filtro (`AAAA-MM-DD` o RFC 3339); el error nombra el parámetro.
pub fn parsear_fecha(
    nombre: &str,
    valor: Option<&str>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, AppError> {
    let Some(texto) = valor.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if let Ok(fecha) = chrono::DateTime::parse_from_rfc3339(texto) {
        return Ok(Some(fecha.with_timezone(&chrono::Utc)));
    }
    chrono::NaiveDate::parse_from_str(texto, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|f| Some(f.and_utc()))
        .ok_or_else(|| {
            AppError::Validacion(format!(
                "El parámetro '{nombre}' debe tener formato AAAA-MM-DD."
            ))
        })
}

/// Respuesta de confirmar y anular: la compra y el movimiento generado (R2).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResultadoCompra {
    pub compra: Compra,
    pub lineas: Vec<CompraLinea>,
    pub movimiento: crate::models::stock::MovimientoDetalle,
}
