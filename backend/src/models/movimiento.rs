//! Modelos y reglas puras de movimientos de stock (007).
//!
//! El signo vive en la línea (`cantidad_delta`): `ENTRADA > 0`, `SALIDA` y `MERMA < 0`,
//! `AJUSTE ≠ 0`. Toda la validación devuelve `Result<_, AppError>`; no hay `unwrap` ni
//! `expect` en rutas de producción.

use crate::error::AppError;

/// Tipos admitidos (lista cerrada, FR-024).
pub const TIPOS_MOVIMIENTO: [&str; 4] = ["ENTRADA", "SALIDA", "MERMA", "AJUSTE"];

/// Motivos admitidos (lista cerrada, FR-024).
pub const MOTIVOS: [&str; 9] = [
    "COMPRA",
    "VENTA",
    "MERMA",
    "AJUSTE",
    "DEVOLUCION",
    "TRANSFERENCIA",
    "INVENTARIO",
    "IMPORTACION",
    "OTRO",
];

/// Tope de líneas por movimiento (evita requests abusivos).
pub const LINEAS_MAX: usize = 200;

/// Tope de la observación libre.
pub const OBSERVACION_MAX: usize = 500;

/// Línea ya validada, lista para aplicar.
#[derive(Debug, Clone)]
pub struct LineaNueva {
    pub producto_id: i32,
    pub ubicacion_id: Option<i32>,
    pub cantidad_delta: i32,
}

/// `trim` + mayúsculas para aceptar `entrada`.
pub fn validar_tipo(valor: &str) -> Result<String, AppError> {
    let tipo = valor.trim().to_uppercase();
    if TIPOS_MOVIMIENTO.contains(&tipo.as_str()) {
        Ok(tipo)
    } else {
        Err(AppError::Validacion(
            "El tipo de movimiento debe ser ENTRADA, SALIDA, MERMA o AJUSTE.".into(),
        ))
    }
}

/// `trim` + mayúsculas para aceptar `compra`.
pub fn validar_motivo(valor: &str) -> Result<String, AppError> {
    let motivo = valor.trim().to_uppercase();
    if MOTIVOS.contains(&motivo.as_str()) {
        Ok(motivo)
    } else {
        Err(AppError::Validacion(
            "El motivo debe ser COMPRA, VENTA, MERMA, AJUSTE, DEVOLUCION, TRANSFERENCIA, INVENTARIO, IMPORTACION u OTRO.".into(),
        ))
    }
}

/// Observación opcional; con motivo `OTRO` es obligatoria (FR-024).
pub fn validar_observacion(
    valor: Option<&str>,
    motivo: &str,
) -> Result<Option<String>, AppError> {
    let observacion = valor
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned);

    if motivo == "OTRO" && observacion.is_none() {
        return Err(AppError::Validacion(
            "Con motivo OTRO tenés que indicar una observación.".into(),
        ));
    }

    match observacion {
        Some(texto) if texto.chars().count() > OBSERVACION_MAX => Err(AppError::Validacion(
            format!("La observación no puede superar los {OBSERVACION_MAX} caracteres."),
        )),
        otro => Ok(otro),
    }
}

/// Valida que el delta tenga el signo que corresponde al tipo.
fn validar_delta(tipo: &str, cantidad_delta: i32) -> Result<(), AppError> {
    if cantidad_delta == 0 {
        return Err(AppError::Validacion(
            "La cantidad de cada línea debe ser distinta de cero.".into(),
        ));
    }
    let correcto = match tipo {
        "ENTRADA" => cantidad_delta > 0,
        "SALIDA" | "MERMA" => cantidad_delta < 0,
        _ => true,
    };
    if correcto {
        Ok(())
    } else if tipo == "ENTRADA" {
        Err(AppError::Validacion(
            "En una ENTRADA la cantidad debe ser mayor que cero.".into(),
        ))
    } else {
        Err(AppError::Validacion(format!(
            "En una {tipo} la cantidad debe ser negativa."
        )))
    }
}

/// Valida la lista completa: no vacía, sin repetir producto+ubicación y con signos correctos.
pub fn validar_lineas(tipo: &str, lineas: &[LineaNueva]) -> Result<(), AppError> {
    if lineas.is_empty() {
        return Err(AppError::Validacion(
            "El movimiento debe tener al menos una línea.".into(),
        ));
    }
    if lineas.len() > LINEAS_MAX {
        return Err(AppError::Validacion(format!(
            "El movimiento no puede superar las {LINEAS_MAX} líneas."
        )));
    }

    let mut vistas = std::collections::HashSet::with_capacity(lineas.len());
    for linea in lineas {
        if linea.producto_id <= 0 {
            return Err(AppError::Validacion(
                "El producto de cada línea es obligatorio.".into(),
            ));
        }
        if let Some(ubicacion_id) = linea.ubicacion_id {
            if ubicacion_id <= 0 {
                return Err(AppError::Validacion(
                    "La ubicación indicada no es válida.".into(),
                ));
            }
        }
        validar_delta(tipo, linea.cantidad_delta)?;
        if !vistas.insert((linea.producto_id, linea.ubicacion_id)) {
            return Err(AppError::Validacion(
                "No podés repetir el mismo producto y ubicación en dos líneas.".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn acepta_tipos_y_motivos_con_espacios() {
        assert_eq!(validar_tipo(" entrada ").unwrap_or_default(), "ENTRADA");
        assert_eq!(validar_motivo("compra").unwrap_or_default(), "COMPRA");
    }

    #[test]
    fn otro_exige_observacion() {
        assert!(validar_observacion(None, "OTRO").is_err());
        assert!(validar_observacion(Some("   "), "OTRO").is_err());
        assert_eq!(
            validar_observacion(Some(" rotura "), "OTRO").unwrap_or_default(),
            Some("rotura".to_string())
        );
    }
}
