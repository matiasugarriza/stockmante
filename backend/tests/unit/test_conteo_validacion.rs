//! 009 — reglas puras de conteos (T007), sin base de datos.

use stockmante_backend::models::conteo::{
    validar_cantidad_contada, validar_conteo, ConteoNuevo, OBSERVACION_MAX,
};

fn conteo(deposito_id: i32, observacion: Option<&str>) -> ConteoNuevo {
    ConteoNuevo {
        deposito_id,
        observacion: observacion.map(str::to_owned),
    }
}

#[test]
fn deposito_no_positivo_es_validacion() {
    for id in [0, -1] {
        let error = validar_conteo(&conteo(id, None)).err();
        assert_eq!(error.as_ref().map(|e| e.codigo()), Some("validacion"));
        assert_eq!(
            error.map(|e| e.to_string()).as_deref(),
            Some("El depósito es obligatorio.")
        );
    }
}

#[test]
fn cantidad_contada_negativa_es_validacion() {
    let error = validar_cantidad_contada(-1).err();
    assert_eq!(error.as_ref().map(|e| e.codigo()), Some("validacion"));
    assert!(validar_cantidad_contada(0).is_ok());
    assert!(validar_cantidad_contada(15).is_ok());
}

#[test]
fn observacion_se_recorta_y_en_blanco_es_none() {
    assert_eq!(
        validar_conteo(&conteo(1, Some("  cierre de mes  "))).ok().flatten().as_deref(),
        Some("cierre de mes")
    );
    assert_eq!(validar_conteo(&conteo(1, Some("   "))).ok(), Some(None));
    assert_eq!(validar_conteo(&conteo(1, None)).ok(), Some(None));
}

#[test]
fn observacion_mayor_al_tope_es_validacion() {
    let larga = "x".repeat(OBSERVACION_MAX + 1);
    let error = validar_conteo(&conteo(1, Some(&larga))).err();
    assert_eq!(error.map(|e| e.codigo()), Some("validacion"));
    let justa = "x".repeat(OBSERVACION_MAX);
    assert!(validar_conteo(&conteo(1, Some(&justa))).is_ok());
}
