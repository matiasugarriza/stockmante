//! Reglas puras de conteos de inventario (009).

use crate::error::AppError;

pub const OBSERVACION_MAX: usize = 500;

#[derive(Debug, Clone)]
pub struct ConteoNuevo {
    pub deposito_id: i32,
    pub observacion: Option<String>,
}

pub fn validar_conteo(input: &ConteoNuevo) -> Result<Option<String>, AppError> {
    if input.deposito_id <= 0 {
        return Err(AppError::Validacion("El depósito es obligatorio.".into()));
    }
    let observacion = input
        .observacion
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned);
    if let Some(texto) = &observacion {
        if texto.chars().count() > OBSERVACION_MAX {
            return Err(AppError::Validacion(format!(
                "La observación no puede superar los {OBSERVACION_MAX} caracteres."
            )));
        }
    }
    Ok(observacion)
}

#[derive(Debug, Clone)]
pub struct LineaContada {
    pub linea_id: i32,
    pub cantidad_contada: i32,
}

pub fn validar_cantidad_contada(valor: i32) -> Result<(), AppError> {
    if valor < 0 {
        return Err(AppError::Validacion(
            "La cantidad contada no puede ser negativa.".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn valido() {
        let i = ConteoNuevo {
            deposito_id: 1,
            observacion: Some("  hola  ".into()),
        };
        assert_eq!(validar_conteo(&i).unwrap().unwrap(), "hola");
    }

    #[test]
    fn deposito_invalido() {
        let i = ConteoNuevo {
            deposito_id: 0,
            observacion: None,
        };
        assert!(validar_conteo(&i).is_err());
    }

    #[test]
    fn cantidad_negativa() {
        assert!(validar_cantidad_contada(-1).is_err());
        assert!(validar_cantidad_contada(0).is_ok());
    }
}
