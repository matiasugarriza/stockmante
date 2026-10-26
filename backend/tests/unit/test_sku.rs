//! 003/T015 — normalización y formato del SKU (contracts/validation.md).

use stockmante_backend::error::AppError;
use stockmante_backend::models::producto::{normalizar_sku, validar_sku};

fn es_validacion<T>(resultado: Result<T, AppError>) -> bool {
    matches!(resultado, Err(AppError::Validacion(_)))
}

#[test]
fn normaliza_trim_y_mayusculas() {
    assert_eq!(normalizar_sku("  yerba-1k "), "YERBA-1K");
    assert_eq!(normalizar_sku("sku-001"), "SKU-001");
}

#[test]
fn acepta_minusculas_y_tres_caracteres() {
    assert_eq!(validar_sku("sku-001").expect("SKU válido"), "SKU-001");
    assert_eq!(validar_sku("abc").expect("3 caracteres"), "ABC");
}

#[test]
fn bad_es_valido_porque_normaliza_a_bad() {
    assert_eq!(validar_sku("bad").expect("BAD es válido"), "BAD");
}

#[test]
fn acepta_el_maximo_de_30_caracteres() {
    let treinta = "A".repeat(30);
    assert_eq!(validar_sku(&treinta).expect("30 caracteres"), treinta);
}

#[test]
fn rechaza_caracteres_invalidos_longitud_y_guion_inicial() {
    for invalido in ["bad!", "AB", "-ABC", "A B", "ÑANDU"] {
        assert!(es_validacion(validar_sku(invalido)), "{invalido}");
    }
    assert!(es_validacion(validar_sku(&"A".repeat(31))));
    assert!(es_validacion(validar_sku("")));
    assert!(es_validacion(validar_sku("   ")));
}
