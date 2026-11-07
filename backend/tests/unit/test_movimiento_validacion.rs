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
    assert!(validar_observacion(None, "SALIDA", "OTRO").is_err());
    assert!(validar_observacion(Some("   "), "SALIDA", "OTRO").is_err());

    let observacion = validar_observacion(Some("  rotura de bulto  "), "SALIDA", "OTRO")
        .expect("observación válida")
        .unwrap_or_default();
    assert_eq!(observacion, "rotura de bulto");

    // Sin OTRO la observación es opcional; vacía equivale a ausente.
    assert_eq!(
        validar_observacion(Some("  "), "ENTRADA", "COMPRA").unwrap_or_default(),
        None
    );
    assert_eq!(
        validar_observacion(None, "ENTRADA", "COMPRA").unwrap_or_default(),
        None
    );
}

#[test]
fn ajuste_exige_justificacion() {
    assert!(validar_observacion(None, "AJUSTE", "AJUSTE").is_err());
    assert!(validar_observacion(Some("   "), "AJUSTE", "AJUSTE").is_err());
    let observacion = validar_observacion(Some("  conteo físico  "), "AJUSTE", "AJUSTE")
        .expect("justificación válida")
        .unwrap_or_default();
    assert_eq!(observacion, "conteo físico");
}

#[test]
fn observacion_tiene_tope_de_caracteres() {
    let larga = "x".repeat(501);
    assert!(validar_observacion(Some(&larga), "ENTRADA", "COMPRA").is_err());
    let justa = "x".repeat(500);
    assert!(validar_observacion(Some(&justa), "ENTRADA", "COMPRA").is_ok());
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

// --- Ampliación 2026-11-07 (T021): matriz tipo ↔ motivo y cantidades enteras. ---

use stockmante_backend::models::movimiento::{cantidad_entera, validar_motivo_para_tipo};

#[test]
fn matriz_tipo_motivo_admite_las_combinaciones_del_plan() {
    let admitidas = [
        ("ENTRADA", ["COMPRA", "DEVOLUCION", "OTRO"].as_slice()),
        ("SALIDA", ["VENTA", "DEVOLUCION", "OTRO"].as_slice()),
        ("MERMA", ["MERMA", "OTRO"].as_slice()),
        ("AJUSTE", ["AJUSTE", "OTRO"].as_slice()),
    ];
    for (tipo, motivos) in admitidas {
        for motivo in motivos {
            assert!(
                validar_motivo_para_tipo(tipo, motivo).is_ok(),
                "{tipo} + {motivo} debe admitirse"
            );
        }
    }
}

#[test]
fn matriz_tipo_motivo_rechaza_una_combinacion_por_tipo() {
    let casos = [
        ("ENTRADA", "VENTA", "COMPRA, DEVOLUCION u OTRO"),
        ("SALIDA", "COMPRA", "VENTA, DEVOLUCION u OTRO"),
        ("MERMA", "VENTA", "MERMA u OTRO"),
        ("AJUSTE", "COMPRA", "AJUSTE u OTRO"),
    ];
    for (tipo, motivo, lista) in casos {
        let error = validar_motivo_para_tipo(tipo, motivo).err().map(mensaje);
        assert_eq!(
            error.as_deref(),
            Some(format!("El motivo {motivo} no corresponde a una {tipo}. Motivos válidos: {lista}.").as_str())
        );
    }
}

#[test]
fn motivos_reservados_nombran_su_flujo() {
    let casos = [
        ("TRANSFERENCIA", "El motivo TRANSFERENCIA lo registra el flujo de transferencias."),
        ("INVENTARIO", "El motivo INVENTARIO lo registra el cierre de conteos."),
        ("IMPORTACION", "El motivo IMPORTACION lo registra la importación masiva."),
    ];
    for tipo in TIPOS_MOVIMIENTO {
        for (motivo, esperado) in casos {
            let error = validar_motivo_para_tipo(tipo, motivo).err().map(mensaje);
            assert_eq!(error.as_deref(), Some(esperado), "{tipo} + {motivo}");
        }
    }
}

#[test]
fn cantidad_entera_acepta_enteros_y_rechaza_decimales_y_desbordes() {
    let entero: serde_json::Number = serde_json::from_str("-7").unwrap_or(0.into());
    assert_eq!(cantidad_entera(&entero).ok(), Some(-7));

    let decimal: serde_json::Number = serde_json::from_str("2.5").unwrap_or(0.into());
    assert_eq!(
        cantidad_entera(&decimal).err().map(mensaje).as_deref(),
        Some("La cantidad de cada línea debe ser un número entero distinto de cero.")
    );

    let grande: serde_json::Number = serde_json::from_str("3000000000").unwrap_or(0.into());
    assert_eq!(
        cantidad_entera(&grande).err().map(mensaje).as_deref(),
        Some("La cantidad de cada línea supera el máximo admitido. Revisá el valor.")
    );
}
