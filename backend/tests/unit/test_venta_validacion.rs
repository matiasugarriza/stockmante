//! 013 (T007) — reglas puras de ventas: agregación de líneas, medio de pago y observación.

use stockmante_backend::models::venta::{validar_medio_pago, validar_observacion};
use stockmante_backend::services::ventas::{agregar_lineas, NuevaVentaLinea};

fn linea(producto_id: i32, cantidad: i32) -> NuevaVentaLinea {
    NuevaVentaLinea {
        producto_id,
        cantidad,
    }
}

#[test]
fn agrega_por_producto_en_orden_de_primera_aparicion() {
    let agregadas = agregar_lineas(&[linea(12, 2), linea(15, 1), linea(12, 1)]);
    assert_eq!(agregadas, vec![(12, 3), (15, 1)]);
}

#[test]
fn una_sola_entrada_por_producto() {
    let agregadas = agregar_lineas(&[linea(7, 1), linea(7, 1), linea(7, 1)]);
    assert_eq!(agregadas, vec![(7, 3)]);
    assert!(agregar_lineas(&[]).is_empty());
}

#[test]
fn medio_de_pago_del_catalogo() {
    assert_eq!(validar_medio_pago(" efectivo ").ok().as_deref(), Some("EFECTIVO"));
    assert_eq!(validar_medio_pago("TARJETA").ok().as_deref(), Some("TARJETA"));
    let error = validar_medio_pago("CHEQUE").err().map(|e| e.to_string());
    assert_eq!(
        error.as_deref(),
        Some("El medio de pago debe ser EFECTIVO, TARJETA o TRANSFERENCIA.")
    );
}

#[test]
fn observacion_opcional_con_tope() {
    assert_eq!(validar_observacion(Some("  mostrador ")).ok().flatten().as_deref(), Some("mostrador"));
    assert_eq!(validar_observacion(Some("  ")).ok(), Some(None));
    assert!(validar_observacion(Some(&"x".repeat(501))).is_err());
}
