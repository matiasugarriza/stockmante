//! Modelos y reglas puras de ventas (013). Los clientes viven en `models::cliente`.

use crate::error::AppError;
use serde::Serialize;
use utoipa::ToSchema;

pub const OBSERVACION_MAX: usize = 500;

pub use crate::models::cliente::normalizar_opcional;

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
    /// Saldo del producto en el depósito tras el movimiento más reciente de la venta
    /// (la venta o su anulación). `None` solo si no hay movimiento asociado.
    pub stock_resultante: Option<i32>,
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

pub fn validar_medio_pago(valor: &str) -> Result<String, AppError> {
    let m = valor.trim().to_uppercase();
    if ["EFECTIVO","TARJETA","TRANSFERENCIA"].contains(&m.as_str()) { Ok(m) } else {
        Err(AppError::Validacion("El medio de pago debe ser EFECTIVO, TARJETA o TRANSFERENCIA.".into()))
    }
}
pub fn validar_observacion(valor: Option<&str>) -> Result<Option<String>, AppError> {
    normalizar_opcional(valor, OBSERVACION_MAX, "La observación")
}
