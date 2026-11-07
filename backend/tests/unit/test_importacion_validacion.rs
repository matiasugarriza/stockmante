//! 016 (T005) — parser y validación pura de la importación CSV.

use rust_decimal::Decimal;
use std::str::FromStr;
use stockmante_backend::error::AppError;
use stockmante_backend::services::importaciones::{
    detectar_dialecto, parsear_archivo, repetidos, validar_fila, DetalleImportacion,
};

const ENCABEZADO: &str = "nombre,sku,categoria,unidad_medida,costo,precio_venta,stock_minimo,codigo_barras,stock_inicial,deposito";

fn detalles(r: Result<impl std::fmt::Debug, AppError>) -> Vec<DetalleImportacion> {
    match r {
        Err(AppError::ImportacionInvalida(d)) => d,
        otro => panic!("se esperaba ImportacionInvalida, llegó {otro:?}"),
    }
}

fn dec(v: &str) -> Decimal {
    Decimal::from_str(v).unwrap_or_default()
}

#[test]
fn detecta_delimitador_y_decimal() {
    let coma = detectar_dialecto("nombre,sku,costo");
    assert_eq!((coma.delimitador, coma.decimal), (b',', '.'));
    let punto_y_coma = detectar_dialecto("nombre;sku;costo");
    assert_eq!((punto_y_coma.delimitador, punto_y_coma.decimal), (b';', ','));
    // Las comas dentro de comillas no cuentan.
    let comillas = detectar_dialecto("\"a,b,c\";sku;costo");
    assert_eq!(comillas.delimitador, b';');
}

#[test]
fn tolera_bom_y_comillas_y_numera_desde_la_fila_2() {
    let csv = format!(
        "\u{FEFF}{ENCABEZADO}\n\"Yerba, 1 kg\",yer-1k,Almacén,UNIDAD,100.5,150,5,,10,Principal\n"
    );
    let archivo = parsear_archivo(csv.as_bytes()).expect("parsea");
    assert_eq!(archivo.filas.len(), 1);
    assert_eq!(archivo.filas[0].numero, 2);
    assert_eq!(archivo.filas[0].nombre, "Yerba, 1 kg");
}

#[test]
fn errores_globales_del_archivo() {
    let sin_sku = "nombre,categoria,unidad_medida,costo,precio_venta,stock_minimo\nA,B,UNIDAD,1,1,0\n";
    let d = detalles(parsear_archivo(sin_sku.as_bytes()));
    assert_eq!((d[0].fila, d[0].columna.as_str(), d[0].mensaje.as_str()), (1, "sku", "Falta la columna 'sku'."));

    let vacio = format!("{ENCABEZADO}\n");
    let d = detalles(parsear_archivo(vacio.as_bytes()));
    assert_eq!(d[0].mensaje, "El archivo no contiene filas para importar.");

    let mut grande = format!("{ENCABEZADO}\n");
    for i in 0..5001 {
        grande.push_str(&format!("P{i},SKU-{i:05},Cat,UNIDAD,1,1,0,,,\n"));
    }
    let d = detalles(parsear_archivo(grande.as_bytes()));
    assert_eq!(d[0].mensaje, "El archivo supera el límite de 5.000 filas o 5 MB.");
}

#[test]
fn fila_con_columnas_de_mas_es_error_de_fila() {
    let csv = format!("{ENCABEZADO}\nA,SKU-A1,Cat,UNIDAD,1,1,0,,,,extra\n");
    let archivo = parsear_archivo(csv.as_bytes()).expect("parsea");
    let (valida, errores) = validar_fila(&archivo.filas[0], archivo.dialecto.decimal);
    assert!(valida.is_none());
    assert_eq!(errores[0].columna, "archivo");
    assert_eq!(errores[0].mensaje, "La fila tiene más columnas que el encabezado.");
}

#[test]
fn opcionales_vacios_y_decimal_con_coma() {
    let csv = "nombre;sku;categoria;unidad_medida;costo;precio_venta;stock_minimo\nYerba;yer-1k;Almacén;UNIDAD;1500,50;2000;3\n";
    let archivo = parsear_archivo(csv.as_bytes()).expect("parsea");
    let (valida, errores) = validar_fila(&archivo.filas[0], archivo.dialecto.decimal);
    assert!(errores.is_empty(), "{errores:?}");
    let v = valida.expect("válida");
    assert_eq!(v.sku, "YER-1K");
    assert_eq!(v.costo, dec("1500.50"));
    assert_eq!(v.stock_inicial, 0, "stock_inicial vacío = 0");
    assert_eq!(v.deposito, None, "depósito vacío = Principal");
    assert_eq!(v.codigo_barras, None);
}

#[test]
fn errores_por_celda() {
    let csv = format!("{ENCABEZADO}\n,X,Cat,LITRO,abc,-1,-2,12,-5,\n");
    let archivo = parsear_archivo(csv.as_bytes()).expect("parsea");
    let (valida, errores) = validar_fila(&archivo.filas[0], archivo.dialecto.decimal);
    assert!(valida.is_none());
    let columnas: Vec<&str> = errores.iter().map(|e| e.columna.as_str()).collect();
    for esperada in ["nombre", "sku", "unidad_medida", "costo", "precio_venta", "stock_minimo", "codigo_barras", "stock_inicial"] {
        assert!(columnas.contains(&esperada), "falta error en {esperada}: {errores:?}");
    }
    assert!(errores.iter().all(|e| e.fila == 2));
}

#[test]
fn repetidos_nombran_la_primera_fila() {
    let filas = vec![(3, "YER-1K".to_string()), (5, "ARE-1".to_string()), (8, "YER-1K".to_string())];
    let r = repetidos(&filas, "sku", "El SKU");
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].fila, 8);
    assert_eq!(r[0].mensaje, "El SKU 'YER-1K' está repetido en el archivo (filas 3 y 8).");
}
