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

/// Motivos admitidos en el alta manual por tipo (ampliación 2026-11-07, 7.9).
///
/// Los flujos de 008, 009, 012, 013 y 016 insertan sus movimientos con su propio
/// SQL y no pasan por esta matriz.
pub fn motivos_para_tipo(tipo: &str) -> &'static [&'static str] {
    match tipo {
        "ENTRADA" => &["COMPRA", "DEVOLUCION", "OTRO"],
        "SALIDA" => &["VENTA", "DEVOLUCION", "OTRO"],
        "MERMA" => &["MERMA", "OTRO"],
        "AJUSTE" => &["AJUSTE", "OTRO"],
        _ => &[],
    }
}

/// Coherencia tipo ↔ motivo del alta manual; los motivos reservados nombran su flujo.
pub fn validar_motivo_para_tipo(tipo: &str, motivo: &str) -> Result<(), AppError> {
    let reservado = match motivo {
        "TRANSFERENCIA" => Some("el flujo de transferencias"),
        "INVENTARIO" => Some("el cierre de conteos"),
        "IMPORTACION" => Some("la importación masiva"),
        _ => None,
    };
    if let Some(flujo) = reservado {
        return Err(AppError::Validacion(format!(
            "El motivo {motivo} lo registra {flujo}."
        )));
    }
    let admitidos = motivos_para_tipo(tipo);
    if admitidos.contains(&motivo) {
        return Ok(());
    }
    let lista = match admitidos.split_last() {
        Some((ultimo, resto)) if !resto.is_empty() => format!("{} u {ultimo}", resto.join(", ")),
        Some((ultimo, _)) => (*ultimo).to_string(),
        None => String::new(),
    };
    Err(AppError::Validacion(format!(
        "El motivo {motivo} no corresponde a una {tipo}. Motivos válidos: {lista}."
    )))
}

/// Convierte la cantidad recibida como número JSON a `i32` (7.11).
pub fn cantidad_entera(valor: &serde_json::Number) -> Result<i32, AppError> {
    let fuera_de_rango = || {
        AppError::Validacion(
            "La cantidad de cada línea supera el máximo admitido. Revisá el valor.".into(),
        )
    };
    if let Some(v) = valor.as_i64() {
        return i32::try_from(v).map_err(|_| fuera_de_rango());
    }
    if valor.as_u64().is_some() {
        return Err(fuera_de_rango());
    }
    Err(AppError::Validacion(
        "La cantidad de cada línea debe ser un número entero distinto de cero.".into(),
    ))
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
                "El producto de cada línea es obligatorio. Elegí un producto o quitá la línea.".into(),
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
