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
