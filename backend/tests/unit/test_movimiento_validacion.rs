//! 007 — reglas puras de movimientos de stock (sin base de datos).
//!
//! Cubre tipos, motivos, observación obligatoria con `OTRO`, signo del delta por tipo,
//! lista vacía, líneas repetidas y tope de líneas.

use stockmante_backend::error::AppError;
use stockmante_backend::models::movimiento::{
    validar_lineas, validar_motivo, validar_observacion, validar_tipo, LineaNueva, LINEAS_MAX,
    MOTIVOS, TIPOS_MOVIMIENTO,
};

fn linea(producto_id: i32, ubicacion_id: Option<i32>, cantidad_delta: i32) -> LineaNueva {
    LineaNueva {
        producto_id,
        ubicacion_id,
        cantidad_delta,
    }
}

fn mensaje(error: AppError) -> String {
    error.to_string()
}

#[test]
fn tipos_y_motivos_normalizan_espacios_y_mayusculas() {
    assert_eq!(validar_tipo(" entrada ").unwrap_or_default(), "ENTRADA");
    assert_eq!(validar_tipo("ajuste").unwrap_or_default(), "AJUSTE");
    assert_eq!(validar_motivo(" Compra ").unwrap_or_default(), "COMPRA");
    assert_eq!(validar_motivo("otro").unwrap_or_default(), "OTRO");

    for tipo in TIPOS_MOVIMIENTO {
        assert!(validar_tipo(tipo).is_ok(), "tipo válido rechazado: {tipo}");
    }
    for motivo in MOTIVOS {
        assert!(
            validar_motivo(motivo).is_ok(),
            "motivo válido rechazado: {motivo}"
        );
    }
}

#[test]
fn tipo_o_motivo_desconocido_es_422() {
    let error = validar_tipo("INGRESO").expect_err("INGRESO no es un tipo");
    assert!(matches!(error, AppError::Validacion(_)));
    assert_eq!(error.status(), 422);
    assert!(
        mensaje(error).contains("ENTRADA, SALIDA, MERMA o AJUSTE"),
        "el mensaje debe listar los tipos válidos"
    );

    let error = validar_motivo("ROBO").expect_err("ROBO no es un motivo");
    assert!(matches!(error, AppError::Validacion(_)));
    assert!(mensaje(error).contains("OTRO"));
}

#[test]
fn otro_exige_observacion_no_vacia() {
    assert!(validar_observacion(None, "OTRO").is_err());
    assert!(validar_observacion(Some("   "), "OTRO").is_err());

    let observacion = validar_observacion(Some("  rotura de bulto  "), "OTRO")
        .expect("observación válida")
        .unwrap_or_default();
    assert_eq!(observacion, "rotura de bulto");

    // Sin OTRO la observación es opcional; vacía equivale a ausente.
    assert_eq!(
        validar_observacion(Some("  "), "COMPRA").unwrap_or_default(),
        None
    );
    assert_eq!(
        validar_observacion(None, "COMPRA").unwrap_or_default(),
        None
    );
}

#[test]
fn observacion_tiene_tope_de_caracteres() {
    let larga = "x".repeat(501);
    assert!(validar_observacion(Some(&larga), "COMPRA").is_err());
    let justa = "x".repeat(500);
    assert!(validar_observacion(Some(&justa), "COMPRA").is_ok());
}

#[test]
fn delta_cero_es_invalido_en_cualquier_tipo() {
    for tipo in TIPOS_MOVIMIENTO {
        let error = validar_lineas(tipo, &[linea(1, None, 0)])
            .expect_err("delta cero debe fallar");
        assert!(matches!(error, AppError::Validacion(_)), "{tipo}");
        assert!(mensaje(error).contains("distinta de cero"), "{tipo}");
    }
}

#[test]
fn el_signo_del_delta_depende_del_tipo() {
    // ENTRADA exige positivo.
    assert!(validar_lineas("ENTRADA", &[linea(1, None, 5)]).is_ok());
    let error = validar_lineas("ENTRADA", &[linea(1, None, -5)]).expect_err("negativo");
    assert!(mensaje(error).contains("mayor que cero"));

    // SALIDA y MERMA exigen negativo.
    for tipo in ["SALIDA", "MERMA"] {
        assert!(validar_lineas(tipo, &[linea(1, None, -3)]).is_ok(), "{tipo}");
        let error = validar_lineas(tipo, &[linea(1, None, 3)]).expect_err("positivo");
        assert!(mensaje(error).contains("negativa"), "{tipo}");
    }

    // AJUSTE admite ambos signos.
    assert!(validar_lineas("AJUSTE", &[linea(1, None, 4)]).is_ok());
    assert!(validar_lineas("AJUSTE", &[linea(1, None, -4)]).is_ok());
}

#[test]
fn movimiento_sin_lineas_es_422() {
    let error = validar_lineas("ENTRADA", &[]).expect_err("sin líneas");
    assert!(matches!(error, AppError::Validacion(_)));
    assert!(mensaje(error).contains("al menos una línea"));
}

#[test]
fn no_se_repite_producto_y_ubicacion() {
    // Misma ubicación (incluida "sin ubicación") dos veces: error.
    let error = validar_lineas("ENTRADA", &[linea(1, None, 2), linea(1, None, 3)])
        .expect_err("duplicado sin ubicación");
    assert!(mensaje(error).contains("No podés repetir"));

    let error = validar_lineas("ENTRADA", &[linea(1, Some(7), 2), linea(1, Some(7), 3)])
        .expect_err("duplicado con ubicación");
    assert!(mensaje(error).contains("No podés repetir"));

    // Mismo producto en ubicaciones distintas: válido.
    assert!(validar_lineas("ENTRADA", &[linea(1, Some(7), 2), linea(1, Some(8), 3)]).is_ok());
}

#[test]
fn producto_y_ubicacion_deben_ser_positivos() {
    assert!(validar_lineas("ENTRADA", &[linea(0, None, 1)]).is_err());
    assert!(validar_lineas("ENTRADA", &[linea(1, Some(0), 1)]).is_err());
}

#[test]
fn hay_un_tope_de_lineas() {
    let lineas: Vec<LineaNueva> = (1..=LINEAS_MAX as i32)
        .map(|id| linea(id, None, 1))
        .collect();
    assert!(validar_lineas("ENTRADA", &lineas).is_ok());

    let mut excedidas = lineas;
    excedidas.push(linea(9999, None, 1));
    let error = validar_lineas("ENTRADA", &excedidas).expect_err("demasiadas líneas");
    assert!(mensaje(error).contains("no puede superar"));
}
