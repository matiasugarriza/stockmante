//! Modelos y reglas puras de depósitos y ubicaciones (005).
//!
//! La jerarquía vive en `ubicaciones.padre_id`; `deposito_id` es obligatorio
//! (la migración 0003 ya impide huérfanas). Las reglas de mismo depósito,
//! ciclos y dependencias se validan en el servicio, dentro de una transacción.

use crate::error::AppError;
use serde::Serialize;
use utoipa::ToSchema;

/// Tipos admitidos de ubicación (enum simple, spec 005).
pub const TIPOS_UBICACION: [&str; 4] = ["PASILLO", "ESTANTERIA", "NIVEL", "POSICION"];

/// Largo máximo de la dirección libre.
pub const DIRECCION_MAX: usize = 500;

/// Depósito tal como lo devuelve la API.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Deposito {
    pub id: i32,
    pub nombre: String,
    pub identificador: String,
    pub direccion: Option<String>,
    pub activo: bool,
}

/// Ubicación jerárquica dentro de un depósito.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Ubicacion {
    pub id: i32,
    pub deposito_id: i32,
    pub padre_id: Option<i32>,
    pub nombre: String,
    pub tipo: String,
    pub activo: bool,
}

/// Lista plana de ubicaciones; el cliente arma el árbol con `padre_id`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaUbicaciones {
    pub items: Vec<Ubicacion>,
}

/// `trim` del nombre; la unicidad exacta la impone la base.
pub fn normalizar_nombre_deposito(valor: &str) -> String {
    valor.trim().to_string()
}

/// 1–120 caracteres tras normalizar.
pub fn validar_nombre_deposito(valor: &str) -> Result<String, AppError> {
    let nombre = normalizar_nombre_deposito(valor);
    let largo = nombre.chars().count();
    if largo == 0 {
        Err(AppError::Validacion(
            "El nombre del depósito es obligatorio.".into(),
        ))
    } else if largo > 120 {
        Err(AppError::Validacion(
            "El nombre del depósito no puede superar los 120 caracteres.".into(),
        ))
    } else {
        Ok(nombre)
    }
}

/// 1–40 caracteres tras normalizar.
pub fn validar_identificador(valor: &str) -> Result<String, AppError> {
    let identificador = valor.trim().to_string();
    let largo = identificador.chars().count();
    if largo == 0 {
        Err(AppError::Validacion(
            "El identificador del depósito es obligatorio.".into(),
        ))
    } else if largo > 40 {
        Err(AppError::Validacion(
            "El identificador del depósito no puede superar los 40 caracteres.".into(),
        ))
    } else {
        Ok(identificador)
    }
}

/// `trim`; vacío → `None`.
pub fn normalizar_direccion(valor: Option<&str>) -> Option<String> {
    valor
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// Hasta 500 caracteres; vacío → `None`.
pub fn validar_direccion(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match normalizar_direccion(valor) {
        Some(direccion) if direccion.chars().count() > DIRECCION_MAX => {
            Err(AppError::Validacion(format!(
                "La dirección no puede superar los {DIRECCION_MAX} caracteres."
            )))
        }
        otro => Ok(otro),
    }
}

/// 1–80 caracteres tras normalizar.
pub fn validar_nombre_ubicacion(valor: &str) -> Result<String, AppError> {
    let nombre = valor.trim().to_string();
    let largo = nombre.chars().count();
    if largo == 0 {
        Err(AppError::Validacion(
            "El nombre de la ubicación es obligatorio.".into(),
        ))
    } else if largo > 80 {
        Err(AppError::Validacion(
            "El nombre de la ubicación no puede superar los 80 caracteres.".into(),
        ))
    } else {
        Ok(nombre)
    }
}

/// Lista cerrada; normaliza a mayúsculas para aceptar `estanteria`.
pub fn validar_tipo_ubicacion(valor: &str) -> Result<String, AppError> {
    let tipo = valor.trim().to_uppercase();
    if TIPOS_UBICACION.contains(&tipo.as_str()) {
        Ok(tipo)
    } else {
        Err(AppError::Validacion(
            "El tipo de ubicación debe ser PASILLO, ESTANTERIA, NIVEL o POSICION.".into(),
        ))
    }
}
