//! 003/T016 — validación de los campos de producto (contracts/validation.md).

use rust_decimal::Decimal;
use std::str::FromStr;
use stockmante_backend::error::AppError;
use stockmante_backend::models::producto::{
    normalizar_codigo_barras, validar_codigo_barras, validar_monto, validar_nombre_producto,
    validar_stock_minimo, validar_unidad,
};

fn es_validacion<T>(resultado: Result<T, AppError>) -> bool {
    matches!(resultado, Err(AppError::Validacion(_)))
}

fn decimal(valor: &str) -> Decimal {
    Decimal::from_str(valor).expect("decimal de test")
}

#[test]
fn codigo_barras_vacio_o_blanco_es_none() {
    assert_eq!(normalizar_codigo_barras(Some("")), None);
    assert_eq!(normalizar_codigo_barras(Some("   ")), None);
    assert_eq!(normalizar_codigo_barras(None), None);
    assert_eq!(
        normalizar_codigo_barras(Some("  7791234567890 ")),
        Some("7791234567890".to_string())
    );
}

#[test]
fn codigo_barras_acepta_8_a_64_alfanumericos() {
    assert_eq!(
        validar_codigo_barras(Some("7791234567890")).expect("13 dígitos"),
        Some("7791234567890".to_string())
    );
    assert!(validar_codigo_barras(Some("")).expect("vacío es null").is_none());
    let sesenta_y_cuatro = "A1".repeat(32);
    assert!(validar_codigo_barras(Some(&sesenta_y_cuatro)).is_ok());
    assert!(es_validacion(validar_codigo_barras(Some("1234567"))));
    assert!(es_validacion(validar_codigo_barras(Some(&"9".repeat(65)))));
    assert!(es_validacion(validar_codigo_barras(Some("7791-2345"))));
}

#[test]
fn montos_rechazan_negativos_mas_de_dos_decimales_y_el_tope() {
    assert_eq!(
        validar_monto(decimal("1200.50"), "costo").expect("monto válido"),
        decimal("1200.5")
    );
    assert!(validar_monto(Decimal::ZERO, "costo").is_ok());
    assert!(es_validacion(validar_monto(decimal("-1"), "costo")));
    assert!(es_validacion(validar_monto(decimal("10.999"), "precio_venta")));
    assert!(es_validacion(validar_monto(
        Decimal::from(10_000_000_000i64),
        "precio_venta"
    )));
    assert!(validar_monto(decimal("9999999999.99"), "precio_venta").is_ok());
}

#[test]
fn los_montos_del_json_conservan_dos_decimales() {
    // El contrato exige números JSON; `serde-float` no debe inflar la escala.
    let costo: Decimal = serde_json::from_str("1200.50").expect("número JSON → Decimal");
    assert_eq!(costo, decimal("1200.5"));
    assert!(costo.scale() <= 2);
    let precio: Decimal = serde_json::from_str("10.99").expect("número JSON → Decimal");
    assert_eq!(precio, decimal("10.99"));
    assert!(precio.scale() <= 2);
}

#[test]
fn stock_minimo_entero_no_negativo() {
    assert_eq!(validar_stock_minimo(0).expect("cero"), 0);
    assert_eq!(validar_stock_minimo(6).expect("seis"), 6);
    assert!(es_validacion(validar_stock_minimo(-3)));
}

#[test]
fn unidad_solo_admite_la_lista_cerrada() {
    for unidad in ["UNIDAD", "KG", "METRO", "CAJA"] {
        assert_eq!(validar_unidad(unidad).expect(unidad), unidad);
    }
    assert_eq!(validar_unidad(" KG ").expect("trim"), "KG");
    assert!(es_validacion(validar_unidad("LITRO")));
    assert!(es_validacion(validar_unidad("kg")));
    assert!(es_validacion(validar_unidad("")));
}

#[test]
fn nombre_de_producto_de_1_a_160() {
    assert_eq!(
        validar_nombre_producto("  Yerba 1 kg  ").expect("nombre válido"),
        "Yerba 1 kg"
    );
    assert!(validar_nombre_producto(&"a".repeat(160)).is_ok());
    assert!(es_validacion(validar_nombre_producto("")));
    assert!(es_validacion(validar_nombre_producto("   ")));
    assert!(es_validacion(validar_nombre_producto(&"a".repeat(161))));
}
