//! Modelos y reglas puras de clientes y ventas (013).

use crate::error::AppError;
use serde::Serialize;
use utoipa::ToSchema;

pub const NOMBRE_MAX: usize = 120;
pub const DOCUMENTO_MAX: usize = 20;
pub const EMAIL_MAX: usize = 200;
pub const TELEFONO_MAX: usize = 60;
pub const DIRECCION_MAX: usize = 500;
pub const OBSERVACION_MAX: usize = 500;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Cliente {
    pub id: i32,
    pub nombre: String,
    pub documento: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub direccion: Option<String>,
    pub activo: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaClientes {
    pub items: Vec<Cliente>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct VentaLinea {
    pub id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: Option<String>,
    pub cantidad: i32,
    #[schema(value_type = f64)]
    pub precio_unitario: rust_decimal::Decimal,
    #[schema(value_type = f64)]
    pub subtotal: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Venta {
    pub id: i32,
    pub numero: String,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub cliente_id: Option<i32>,
    pub cliente_nombre: Option<String>,
    pub medio_pago: String,
    #[schema(value_type = f64)]
    pub subtotal: rust_decimal::Decimal,
    #[schema(value_type = f64)]
    pub descuento: rust_decimal::Decimal,
    #[schema(value_type = f64)]
    pub total: rust_decimal::Decimal,
    pub estado: String,
    pub observacion: Option<String>,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub creado_en: chrono::DateTime<chrono::Utc>,
    pub anulado_en: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VentaDetalle {
    pub venta: Venta,
    pub lineas: Vec<VentaLinea>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaVentas {
    pub items: Vec<Venta>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

pub fn validar_nombre(valor: &str) -> Result<String, AppError> {
    let n = valor.trim().to_string();
    let l = n.chars().count();
    if l == 0 { return Err(AppError::Validacion("El nombre del cliente es obligatorio.".into())); }
    if l > NOMBRE_MAX { return Err(AppError::Validacion(format!("El nombre no puede superar los {NOMBRE_MAX} caracteres."))); }
    Ok(n)
}
pub fn validar_documento(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match valor.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(c) => {
            if c.chars().count() > DOCUMENTO_MAX { return Err(AppError::Validacion(format!("El documento no puede superar los {DOCUMENTO_MAX} caracteres."))); }
            Ok(Some(c.to_string()))
        }
    }
}
pub fn normalizar_opcional(valor: Option<&str>, max: usize, campo: &str) -> Result<Option<String>, AppError> {
    match valor.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) => {
            if v.chars().count() > max { return Err(AppError::Validacion(format!("{campo} no puede superar los {max} caracteres."))); }
            Ok(Some(v.to_string()))
        }
    }
}
pub fn validar_medio_pago(valor: &str) -> Result<String, AppError> {
    let m = valor.trim().to_uppercase();
    if ["EFECTIVO","TARJETA","TRANSFERENCIA"].contains(&m.as_str()) { Ok(m) } else {
        Err(AppError::Validacion("El medio de pago debe ser EFECTIVO, TARJETA o TRANSFERENCIA.".into()))
    }
}
pub fn validar_observacion(valor: Option<&str>) -> Result<Option<String>, AppError> {
    normalizar_opcional(valor, OBSERVACION_MAX, "La observación")
}
