//! 008 — reglas puras de transferencias (T006), sin base de datos.

use stockmante_backend::error::AppError;
use stockmante_backend::models::transferencia::{
    validar_transferencia, TransferenciaNueva, OBSERVACION_MAX,
};

fn valida() -> TransferenciaNueva {
    TransferenciaNueva {
        producto_id: 1,
        deposito_origen_id: 1,
        deposito_destino_id: 2,
        cantidad: 5,
        observacion: None,
    }
}

fn mensaje(resultado: Result<Option<String>, AppError>) -> Option<String> {
    resultado.err().map(|e| e.to_string())
}

#[test]
fn una_transferencia_valida_pasa_y_normaliza_la_observacion() {
    let mut input = valida();
    input.observacion = Some("  reposición  ".into());
    assert_eq!(
        validar_transferencia(&input).ok().flatten().as_deref(),
        Some("reposición")
    );
    input.observacion = Some("   ".into());
    assert_eq!(validar_transferencia(&input).ok(), Some(None));
}

#[test]
fn origen_igual_a_destino_es_transferencia_mismo_deposito() {
    let mut input = valida();
    input.deposito_destino_id = input.deposito_origen_id;
    let error = validar_transferencia(&input).err();
    assert!(matches!(error, Some(AppError::TransferenciaMismoDeposito)));
    assert_eq!(
        error.map(|e| e.codigo()),
        Some("transferencia_mismo_deposito")
    );
}

#[test]
fn cantidad_no_positiva_es_422() {
    for cantidad in [0, -3] {
        let mut input = valida();
        input.cantidad = cantidad;
        assert_eq!(
            mensaje(validar_transferencia(&input)).as_deref(),
            Some("La cantidad transferida debe ser mayor que cero.")
        );
    }
}

#[test]
fn campos_obligatorios_faltantes_son_422() {
    let mut sin_producto = valida();
    sin_producto.producto_id = 0;
    assert_eq!(
        mensaje(validar_transferencia(&sin_producto)).as_deref(),
        Some("El producto es obligatorio.")
    );
    let mut sin_origen = valida();
    sin_origen.deposito_origen_id = 0;
    assert_eq!(
        mensaje(validar_transferencia(&sin_origen)).as_deref(),
        Some("El depósito de origen es obligatorio.")
    );
    let mut sin_destino = valida();
    sin_destino.deposito_destino_id = -1;
    assert_eq!(
        mensaje(validar_transferencia(&sin_destino)).as_deref(),
        Some("El depósito de destino es obligatorio.")
    );
}

#[test]
fn observacion_mayor_al_tope_es_422() {
    let mut input = valida();
    input.observacion = Some("x".repeat(OBSERVACION_MAX + 1));
    assert_eq!(
        mensaje(validar_transferencia(&input)).as_deref(),
        Some(format!("La observación no puede superar los {OBSERVACION_MAX} caracteres.").as_str())
    );
    input.observacion = Some("x".repeat(OBSERVACION_MAX));
    assert!(validar_transferencia(&input).is_ok());
}
