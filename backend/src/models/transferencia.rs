//! Reglas puras de transferencias entre depósitos (008).
//!
//! El signo y las cantidades se validan acá, sin I/O ni BD.

use crate::error::AppError;

/// Tope de la observación libre (consistente con movimientos).
pub const OBSERVACION_MAX: usize = 500;

/// Datos de entrada ya deserializados.
#[derive(Debug, Clone)]
pub struct TransferenciaNueva {
    pub producto_id: i32,
    pub deposito_origen_id: i32,
    pub deposito_destino_id: i32,
    pub cantidad: i32,
    pub observacion: Option<String>,
}

/// Valida las reglas puras. No toca la BD.
pub fn validar_transferencia(input: &TransferenciaNueva) -> Result<Option<String>, AppError> {
    if input.producto_id <= 0 {
        return Err(AppError::Validacion(
            "El producto es obligatorio.".into(),
        ));
    }
    if input.deposito_origen_id <= 0 {
        return Err(AppError::Validacion(
            "El depósito de origen es obligatorio.".into(),
        ));
    }
    if input.deposito_destino_id <= 0 {
        return Err(AppError::Validacion(
            "El depósito de destino es obligatorio.".into(),
        ));
    }
    if input.deposito_origen_id == input.deposito_destino_id {
        return Err(AppError::TransferenciaMismoDeposito);
    }
    if input.cantidad <= 0 {
        return Err(AppError::Validacion(
            "La cantidad transferida debe ser mayor que cero.".into(),
        ));
    }

    // Observación opcional, con tope.
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

#[cfg(test)]
mod pruebas {
    use super::*;

    fn input() -> TransferenciaNueva {
        TransferenciaNueva {
            producto_id: 1,
            deposito_origen_id: 1,
            deposito_destino_id: 2,
            cantidad: 5,
            observacion: None,
        }
    }

    #[test]
    fn valido() {
        let v = validar_transferencia(&input()).expect("válido");
        assert!(v.is_none());
    }

    #[test]
    fn mismo_deposito_es_error() {
        let mut i = input();
        i.deposito_destino_id = 1;
        let err = validar_transferencia(&i).expect_err("mismo depósito");
        assert!(matches!(err, AppError::TransferenciaMismoDeposito));
        assert_eq!(err.status(), 422);
        assert_eq!(err.codigo(), "transferencia_mismo_deposito");
    }

    #[test]
    fn cantidad_no_positiva() {
        for c in [0, -1] {
            let mut i = input();
            i.cantidad = c;
            let err = validar_transferencia(&i).expect_err("cantidad inválida");
            assert!(matches!(err, AppError::Validacion(_)));
        }
    }

    #[test]
    fn ids_invalidos() {
        let mut i = input();
        i.producto_id = 0;
        assert!(validar_transferencia(&i).is_err());
        i.producto_id = 1;
        i.deposito_origen_id = 0;
        assert!(validar_transferencia(&i).is_err());
        i.deposito_origen_id = 1;
        i.deposito_destino_id = 0;
        assert!(validar_transferencia(&i).is_err());
    }

    #[test]
    fn observacion_tope() {
        let mut i = input();
        i.observacion = Some("x".repeat(501));
        assert!(validar_transferencia(&i).is_err());
        i.observacion = Some("  hola  ".into());
        let v = validar_transferencia(&i).expect("válido").unwrap();
        assert_eq!(v, "hola");
    }
}
