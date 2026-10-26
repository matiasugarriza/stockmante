//! 005 — reglas puras de depósitos y ubicaciones.
//!
//! La jerarquía (mismo depósito, ciclos, dependencias) se cubre por HTTP en
//! `tests/integration/test_depositos_http.rs`; acá se aísla la validación.

use axum::http::StatusCode;
use stockmante_backend::error::AppError;
use stockmante_backend::models::deposito::{
    normalizar_direccion, validar_direccion, validar_identificador, validar_nombre_deposito,
    validar_nombre_ubicacion, validar_tipo_ubicacion,
};

#[test]
fn nombre_de_deposito_recorta_y_valida_longitud() {
    assert_eq!(
        validar_nombre_deposito("  Depósito Central  ").expect("nombre válido"),
        "Depósito Central"
    );

    let vacio = validar_nombre_deposito("   ").expect_err("vacío");
    assert_eq!(vacio.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(vacio.mensaje_publico(), "El nombre del depósito es obligatorio.");

    let largo = "a".repeat(121);
    let error = validar_nombre_deposito(&largo).expect_err("121 caracteres");
    assert!(error.mensaje_publico().contains("120"));

    let justo = "a".repeat(120);
    assert_eq!(
        validar_nombre_deposito(&justo).expect("120 caracteres"),
        justo
    );
}

#[test]
fn identificador_de_deposito_es_obligatorio_y_acotado() {
    assert_eq!(
        validar_identificador("  DEP-CENTRAL ").expect("identificador válido"),
        "DEP-CENTRAL"
    );

    let vacio = validar_identificador("").expect_err("vacío");
    assert_eq!(vacio.codigo(), "validacion");
    assert!(vacio.mensaje_publico().contains("obligatorio"));

    let error = validar_identificador(&"X".repeat(41)).expect_err("41 caracteres");
    assert!(error.mensaje_publico().contains("40"));
}

#[test]
fn direccion_vacia_es_nula_y_tiene_tope() {
    assert_eq!(normalizar_direccion(None), None);
    assert_eq!(normalizar_direccion(Some("   ")), None);
    assert_eq!(
        normalizar_direccion(Some("  Av. Siempre Viva 742 ")),
        Some("Av. Siempre Viva 742".to_string())
    );

    assert_eq!(validar_direccion(Some("   ")).expect("vacía"), None);
    assert_eq!(
        validar_direccion(Some("  Calle 1 ")).expect("válida"),
        Some("Calle 1".to_string())
    );

    let largo = "a".repeat(501);
    let error = validar_direccion(Some(&largo)).expect_err("501 caracteres");
    assert!(error.mensaje_publico().contains("500"));

    let justo = "a".repeat(500);
    assert!(validar_direccion(Some(&justo)).is_ok());
}

#[test]
fn nombre_de_ubicacion_y_tipo_se_normalizan() {
    assert_eq!(
        validar_nombre_ubicacion("  Estantería A ").expect("nombre"),
        "Estantería A"
    );
    assert_eq!(
        validar_nombre_ubicacion("")
            .expect_err("vacío")
            .mensaje_publico(),
        "El nombre de la ubicación es obligatorio."
    );
    assert!(validar_nombre_ubicacion(&"x".repeat(81))
        .expect_err("81 caracteres")
        .mensaje_publico()
        .contains("80"));

    for tipo in ["PASILLO", "ESTANTERIA", "NIVEL", "POSICION"] {
        assert_eq!(validar_tipo_ubicacion(tipo).expect(tipo), tipo);
    }
    assert_eq!(
        validar_tipo_ubicacion(" estanteria ").expect("minúsculas"),
        "ESTANTERIA"
    );
    let error = validar_tipo_ubicacion("DEPOSITO").expect_err("tipo inválido");
    assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        error.mensaje_publico(),
        "El tipo de ubicación debe ser PASILLO, ESTANTERIA, NIVEL o POSICION."
    );
}

#[test]
fn errores_de_depositos_y_ubicaciones_mapean_status() {
    for (error, esperado) in [
        (AppError::DepositoNombreDuplicado, StatusCode::CONFLICT),
        (
            AppError::DepositoIdentificadorDuplicado,
            StatusCode::CONFLICT,
        ),
        (AppError::DepositoConDependencias, StatusCode::CONFLICT),
        (AppError::UbicacionDuplicada, StatusCode::CONFLICT),
        (AppError::UbicacionConDependencias, StatusCode::CONFLICT),
        (AppError::UbicacionHuerfana, StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        assert_eq!(error.status(), esperado, "{error:?}");
    }

    assert_eq!(
        AppError::DepositoNombreDuplicado.codigo(),
        "deposito_nombre_duplicado"
    );
    assert_eq!(
        AppError::DepositoIdentificadorDuplicado.codigo(),
        "deposito_identificador_duplicado"
    );
    assert_eq!(
        AppError::DepositoConDependencias.codigo(),
        "deposito_con_dependencias"
    );
    assert_eq!(AppError::UbicacionDuplicada.codigo(), "ubicacion_duplicada");
    assert_eq!(
        AppError::UbicacionConDependencias.codigo(),
        "ubicacion_con_dependencias"
    );
    assert_eq!(AppError::UbicacionHuerfana.codigo(), "ubicacion_huerfana");
}
