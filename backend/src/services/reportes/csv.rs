//! CSV de los reportes (014, R7): RFC 4180, `\r\n`, BOM UTF-8 para Excel, sin dependencias.

fn campo(valor: &str) -> String {
    if valor.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", valor.replace('"', "\"\""))
    } else {
        valor.to_string()
    }
}

pub fn a_csv(encabezados: &[&str], filas: impl Iterator<Item = Vec<String>>) -> String {
    let mut salida = String::from("\u{FEFF}");
    let cabecera: Vec<String> = encabezados.iter().map(|e| campo(e)).collect();
    salida.push_str(&cabecera.join(","));
    salida.push_str("\r\n");
    for fila in filas {
        let celdas: Vec<String> = fila.iter().map(|c| campo(c)).collect();
        salida.push_str(&celdas.join(","));
        salida.push_str("\r\n");
    }
    salida
}
