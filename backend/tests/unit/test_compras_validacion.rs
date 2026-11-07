//! 012 (T028) — validaciones puras de compras: número, costo, estado y fechas.

use rust_decimal::Decimal;
use std::str::FromStr;
use stockmante_backend::models::proveedor::{
    parsear_fecha, validar_costo, validar_estado, validar_numero,
};

fn mensaje<T: std::fmt::Debug>(r: Result<T, stockmante_backend::error::AppError>) -> Option<String> {
    r.err().map(|e| e.to_string())
}

#[test]
fn numero_hasta_40_caracteres() {
    assert_eq!(validar_numero(Some(" A-0001 ")).ok().flatten().as_deref(), Some("A-0001"));
    assert_eq!(validar_numero(Some("   ")).ok(), Some(None));
    assert!(validar_numero(Some(&"x".repeat(40))).is_ok());
    assert_eq!(
        mensaje(validar_numero(Some(&"x".repeat(41)))).as_deref(),
        Some("El número de factura o remito no puede superar los 40 caracteres.")
    );
}

#[test]
fn costo_entre_cero_y_el_maximo_de_numeric_12_2() {
    let maximo = Decimal::from_str("9999999999.99").unwrap_or_default();
    assert!(validar_costo(maximo).is_ok());
    assert!(validar_costo(Decimal::ZERO).is_ok());
    assert_eq!(
        mensaje(validar_costo(Decimal::from_str("10000000000").unwrap_or_default())).as_deref(),
        Some("El costo unitario no puede superar 9.999.999.999,99.")
    );
    assert_eq!(
        mensaje(validar_costo(Decimal::from_str("-1").unwrap_or_default())).as_deref(),
        Some("El costo unitario no puede ser negativo.")
    );
}

#[test]
fn estado_del_filtro() {
    assert_eq!(validar_estado(Some(" confirmada ")).ok().flatten().as_deref(), Some("CONFIRMADA"));
    assert_eq!(validar_estado(None).ok(), Some(None));
    assert_eq!(
        mensaje(validar_estado(Some("PAGADA"))).as_deref(),
        Some("El estado debe ser BORRADOR, CONFIRMADA o ANULADA.")
    );
}

#[test]
fn fechas_del_filtro_nombran_el_parametro() {
    assert!(parsear_fecha("desde", Some("2026-11-07")).ok().flatten().is_some());
    assert!(parsear_fecha("hasta", Some("2026-11-07T10:00:00Z")).ok().flatten().is_some());
    assert_eq!(parsear_fecha("desde", Some(" ")).ok(), Some(None));
    assert_eq!(
        mensaje(parsear_fecha("desde", Some("07-11-2026"))).as_deref(),
        Some("El parámetro 'desde' debe tener formato AAAA-MM-DD.")
    );
    assert_eq!(
        mensaje(parsear_fecha("hasta", Some("ayer"))).as_deref(),
        Some("El parámetro 'hasta' debe tener formato AAAA-MM-DD.")
    );
}
