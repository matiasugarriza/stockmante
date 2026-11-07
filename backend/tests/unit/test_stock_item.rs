//! 010 (T002) — construcción pura de `StockItem`: faltante y valorizado por perfil.

use rust_decimal::Decimal;
use std::str::FromStr;
use stockmante_backend::models::stock::{StockFila, StockItem};

fn fila(stock_actual: i64, stock_minimo: i32, stock_deposito: Option<i64>, costo: &str) -> StockFila {
    StockFila {
        producto_id: 1,
        nombre: "Yerba".into(),
        sku: "YER-1".into(),
        categoria_id: 1,
        categoria_nombre: "Almacén".into(),
        unidad_medida: "UNIDAD".into(),
        activo: true,
        stock_actual,
        stock_minimo,
        stock_deposito,
        costo: Decimal::from_str(costo).unwrap_or_default(),
    }
}

fn dec(valor: &str) -> Decimal {
    Decimal::from_str(valor).unwrap_or_default()
}

#[test]
fn faltante_es_el_minimo_menos_el_stock_o_cero() {
    assert_eq!(StockItem::desde_fila(fila(3, 10, None, "1"), true).faltante, 7);
    assert_eq!(StockItem::desde_fila(fila(12, 10, None, "1"), true).faltante, 0);
    assert_eq!(StockItem::desde_fila(fila(10, 10, None, "1"), true).faltante, 0);
    assert_eq!(StockItem::desde_fila(fila(5, 0, None, "1"), true).faltante, 0);
}

#[test]
fn sin_permiso_de_costo_no_expone_costo_ni_valorizado() {
    let item = StockItem::desde_fila(fila(3, 10, Some(2), "150.50"), false);
    assert_eq!(item.costo, None);
    assert_eq!(item.valorizado, None);
}

#[test]
fn valorizado_usa_el_stock_del_deposito_si_existe() {
    let con_deposito = StockItem::desde_fila(fila(10, 0, Some(4), "150.50"), true);
    assert_eq!(con_deposito.costo, Some(dec("150.50")));
    assert_eq!(con_deposito.valorizado, Some(dec("602.00")));

    let global = StockItem::desde_fila(fila(10, 0, None, "150.50"), true);
    assert_eq!(global.valorizado, Some(dec("1505.00")));
}

#[test]
fn valorizado_redondea_a_dos_decimales() {
    let item = StockItem::desde_fila(fila(3, 0, None, "0.335"), true);
    assert_eq!(item.valorizado, Some(dec("1.01")));
}
