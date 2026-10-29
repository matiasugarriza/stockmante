//! 003/T010 — normalización y límites de categorías (contracts/validation.md).

use stockmante_backend::error::AppError;
use stockmante_backend::models::producto::{
    normalizar_descripcion, normalizar_nombre_categoria, validar_descripcion,
    validar_nombre_categoria,
};

fn es_validacion<T>(resultado: Result<T, AppError>) -> bool {
    matches!(resultado, Err(AppError::Validacion(_)))
}

#[test]
fn nombre_se_recorta_y_conserva_acentos() {
    assert_eq!(normalizar_nombre_categoria("  Almacén  "), "Almacén");
    assert_eq!(normalizar_nombre_categoria("Bebidas"), "Bebidas");
}

#[test]
fn nombre_vacio_o_solo_espacios_es_422() {
    assert!(es_validacion(validar_nombre_categoria("")));
    assert!(es_validacion(validar_nombre_categoria("   ")));
}

#[test]
fn nombre_acepta_1_y_80_y_rechaza_81() {
    assert_eq!(validar_nombre_categoria("A").expect("1 carácter"), "A");
    let ochenta = "a".repeat(80);
    assert_eq!(
        validar_nombre_categoria(&ochenta).expect("80 caracteres"),
        ochenta
    );
    assert!(es_validacion(validar_nombre_categoria(&"a".repeat(81))));
}

#[test]
fn descripcion_vacia_o_blanca_es_none() {
    assert_eq!(normalizar_descripcion(None), None);
    assert_eq!(normalizar_descripcion(Some("")), None);
    assert_eq!(normalizar_descripcion(Some("   ")), None);
    assert_eq!(
        normalizar_descripcion(Some("  Productos secos  ")),
        Some("Productos secos".to_string())
    );
}

#[test]
fn descripcion_acepta_255_y_rechaza_256() {
    let doscientos = "d".repeat(255);
    assert_eq!(
        validar_descripcion(Some(&doscientos)).expect("255 caracteres"),
        Some(doscientos)
    );
    assert!(es_validacion(validar_descripcion(Some(&"d".repeat(256)))));
}
