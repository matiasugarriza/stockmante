//! Modelos y reglas puras de clientes (013, FR-049).

use crate::error::AppError;
use serde::Serialize;
use utoipa::ToSchema;

pub const NOMBRE_MAX: usize = 120;
pub const DOCUMENTO_MAX: usize = 20;
pub const EMAIL_MAX: usize = 200;
pub const TELEFONO_MAX: usize = 60;
pub const DIRECCION_MAX: usize = 500;

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
