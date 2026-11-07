//! Importación masiva de productos y stock inicial desde CSV (016, FR-063 … FR-065).
//!
//! - Parseo y validación **puros** (`detectar_dialecto`, `parsear_archivo`, `validar_fila`,
//!   `repetidos`): se prueban sin BD.
//! - `modo=validar` nunca escribe y devuelve el resumen con los errores por celda.
//! - `modo=aplicar` es todo o nada: con cualquier error responde 422 `importacion_invalida` sin
//!   escribir; sin errores, crea productos, stock inicial (`ENTRADA`/`IMPORTACION`) y auditoría
//!   en **una** transacción.
//! - Nunca crea categorías ni depósitos ni actualiza productos existentes.

pub use crate::error::DetalleImportacion;
use crate::error::AppError;
use crate::models::producto::{
    validar_codigo_barras, validar_monto, validar_nombre_producto, validar_sku,
    validar_stock_minimo, validar_unidad,
};
use crate::models::usuario::Usuario;
use crate::services::auditoria;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use utoipa::ToSchema;

pub const FILAS_MAX: usize = 5_000;
pub const BYTES_MAX: usize = 5 * 1024 * 1024;
const MENSAJE_LIMITE: &str = "El archivo supera el límite de 5.000 filas o 5 MB.";

/// Alias histórico del detalle (lo usa el OpenAPI).
pub type FilaErrorImportacion = DetalleImportacion;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResumenImportacion {
    pub filas_leidas: usize,
    pub validas: usize,
    pub con_error: usize,
    pub errores: Vec<DetalleImportacion>,
    pub creados: usize,
    pub aplicado: bool,
}

fn detalle(fila: usize, columna: &str, mensaje: impl Into<String>) -> DetalleImportacion {
    DetalleImportacion {
        fila,
        columna: columna.to_string(),
        mensaje: mensaje.into(),
    }
}

fn global(fila: usize, columna: &str, mensaje: impl Into<String>) -> AppError {
    AppError::ImportacionInvalida(vec![detalle(fila, columna, mensaje)])
}

/// Delimitador y separador decimal del archivo (D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dialecto {
    pub delimitador: u8,
    pub decimal: char,
}

/// `;` si el encabezado tiene más `;` que `,` fuera de comillas (decimal `,`); si no, `,`/`.`.
pub fn detectar_dialecto(encabezado: &str) -> Dialecto {
    let (mut comas, mut puntos_y_coma, mut en_comillas) = (0, 0, false);
    for c in encabezado.chars() {
        match c {
            '"' => en_comillas = !en_comillas,
            ',' if !en_comillas => comas += 1,
            ';' if !en_comillas => puntos_y_coma += 1,
            _ => {}
        }
    }
    if puntos_y_coma > comas {
        Dialecto { delimitador: b';', decimal: ',' }
    } else {
        Dialecto { delimitador: b',', decimal: '.' }
    }
}

/// Una fila de datos tal como vino (celdas recortadas).
#[derive(Debug, Clone, Default)]
pub struct FilaCsv {
    /// Número de fila del archivo (el encabezado es la 1).
    pub numero: usize,
    pub nombre: String,
    pub sku: String,
    pub categoria: String,
    pub unidad_medida: String,
    pub costo: String,
    pub precio_venta: String,
    pub stock_minimo: String,
    pub codigo_barras: String,
    pub stock_inicial: String,
    pub deposito: String,
    pub columnas_de_mas: bool,
}

#[derive(Debug, Clone)]
pub struct ArchivoCsv {
    pub dialecto: Dialecto,
    pub filas: Vec<FilaCsv>,
}

const OBLIGATORIAS: [&str; 7] = [
    "nombre",
    "sku",
    "categoria",
    "unidad_medida",
    "costo",
    "precio_venta",
    "stock_minimo",
];

/// Lee el archivo completo: BOM, dialecto, encabezado y límites. Los errores de archivo son
/// `ImportacionInvalida` global; los de fila quedan para `validar_fila`.
pub fn parsear_archivo(bytes: &[u8]) -> Result<ArchivoCsv, AppError> {
    if bytes.len() > BYTES_MAX {
        return Err(global(0, "archivo", MENSAJE_LIMITE));
    }
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let texto = std::str::from_utf8(bytes).map_err(|_| {
        global(0, "archivo", "No se pudo leer el CSV. Revisá el formato del archivo.")
    })?;
    let primera = texto.lines().next().unwrap_or_default();
    let dialecto = detectar_dialecto(primera);

    let mut lector = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(dialecto.delimitador)
        .from_reader(texto.as_bytes());
    let encabezado: Vec<String> = lector
        .headers()
        .map_err(|_| global(0, "archivo", "No se pudo leer el CSV. Revisá el formato del archivo."))?
        .iter()
        .map(|h| h.trim().to_lowercase())
        .collect();
    let posicion = |nombre: &str| encabezado.iter().position(|h| h == nombre);
    for obligatoria in OBLIGATORIAS {
        if posicion(obligatoria).is_none() {
            return Err(global(1, obligatoria, format!("Falta la columna '{obligatoria}'.")));
        }
    }

    let mut filas = Vec::new();
    for (indice, registro) in lector.records().enumerate() {
        let registro = registro.map_err(|_| {
            global(0, "archivo", "No se pudo leer el CSV. Revisá el formato del archivo.")
        })?;
        if registro.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        if filas.len() >= FILAS_MAX {
            return Err(global(0, "archivo", MENSAJE_LIMITE));
        }
        let celda = |nombre: &str| {
            posicion(nombre)
                .and_then(|i| registro.get(i))
                .map(|v| v.trim().to_string())
                .unwrap_or_default()
        };
        filas.push(FilaCsv {
            numero: indice + 2,
            nombre: celda("nombre"),
            sku: celda("sku"),
            categoria: celda("categoria"),
            unidad_medida: celda("unidad_medida"),
            costo: celda("costo"),
            precio_venta: celda("precio_venta"),
            stock_minimo: celda("stock_minimo"),
            codigo_barras: celda("codigo_barras"),
            stock_inicial: celda("stock_inicial"),
            deposito: celda("deposito"),
            columnas_de_mas: registro.len() > encabezado.len(),
        });
    }
    if filas.is_empty() {
        return Err(global(0, "archivo", "El archivo no contiene filas para importar."));
    }
    Ok(ArchivoCsv { dialecto, filas })
}

/// Fila validada, lista para persistir.
#[derive(Debug, Clone)]
pub struct FilaValida {
    pub numero: usize,
    pub nombre: String,
    pub sku: String,
    pub categoria: String,
    pub unidad: String,
    pub costo: Decimal,
    pub precio: Decimal,
    pub stock_minimo: i32,
    pub codigo_barras: Option<String>,
    pub stock_inicial: i32,
    /// `None` = depósito "Principal" (solo importa si hay stock inicial).
    pub deposito: Option<String>,
}

fn parsear_monto(valor: &str, decimal: char) -> Option<Decimal> {
    let otro = if decimal == ',' { '.' } else { ',' };
    if valor.is_empty() || valor.contains(otro) {
        return None;
    }
    Decimal::from_str(&valor.replace(decimal, ".")).ok()
}

/// Valida todas las celdas de una fila (sin BD): devuelve la fila válida o sus errores.
pub fn validar_fila(fila: &FilaCsv, decimal: char) -> (Option<FilaValida>, Vec<DetalleImportacion>) {
    let n = fila.numero;
    if fila.columnas_de_mas {
        return (
            None,
            vec![detalle(n, "archivo", "La fila tiene más columnas que el encabezado.")],
        );
    }
    let mut errores = Vec::new();
    let mut anotar = |columna: &str, error: AppError| errores.push(detalle(n, columna, error.to_string()));

    let mut obligatorio = |valor: &str, columna: &str, mensaje: &str| {
        let vacio = valor.is_empty();
        if vacio {
            anotar(columna, AppError::Validacion(mensaje.into()));
        }
        !vacio
    };
    let hay_nombre = obligatorio(&fila.nombre, "nombre", "El nombre es obligatorio.");
    let hay_sku = obligatorio(&fila.sku, "sku", "El SKU es obligatorio.");
    let hay_costo = obligatorio(&fila.costo, "costo", "El costo es obligatorio.");
    let hay_precio = obligatorio(&fila.precio_venta, "precio_venta", "El precio de venta es obligatorio.");

    let nombre = if hay_nombre {
        validar_nombre_producto(&fila.nombre).map_err(|e| anotar("nombre", e)).ok()
    } else {
        None
    };
    let sku = if hay_sku { validar_sku(&fila.sku).map_err(|e| anotar("sku", e)).ok() } else { None };
    let categoria = if fila.categoria.is_empty() {
        anotar("categoria", AppError::Validacion("La categoría es obligatoria.".into()));
        None
    } else {
        Some(fila.categoria.clone())
    };
    let unidad = validar_unidad(&fila.unidad_medida.to_uppercase())
        .map_err(|e| anotar("unidad_medida", e))
        .ok();
    let mut monto = |valor: &str, columna: &str, etiqueta: &str| match parsear_monto(valor, decimal) {
        Some(m) => validar_monto(m, etiqueta).map_err(|e| anotar(columna, e)).ok(),
        None => {
            anotar(columna, AppError::Validacion(format!("{etiqueta} inválido.")));
            None
        }
    };
    let costo = if hay_costo { monto(&fila.costo, "costo", "Costo") } else { None };
    let precio = if hay_precio { monto(&fila.precio_venta, "precio_venta", "Precio de venta") } else { None };
    let stock_minimo = match fila.stock_minimo.parse::<i32>() {
        Ok(v) => validar_stock_minimo(v).map_err(|e| anotar("stock_minimo", e)).ok(),
        Err(_) => {
            anotar("stock_minimo", AppError::Validacion("El stock mínimo debe ser un entero mayor o igual a 0.".into()));
            None
        }
    };
    let codigo_barras = validar_codigo_barras(Some(&fila.codigo_barras))
        .map_err(|e| anotar("codigo_barras", e))
        .ok();
    let stock_inicial = if fila.stock_inicial.is_empty() {
        Some(0)
    } else {
        match fila.stock_inicial.parse::<i32>() {
            Ok(v) if v >= 0 => Some(v),
            _ => {
                anotar("stock_inicial", AppError::Validacion("El stock inicial debe ser un entero mayor o igual a 0.".into()));
                None
            }
        }
    };

    match (nombre, sku, categoria, unidad, costo, precio, stock_minimo, codigo_barras, stock_inicial) {
        (Some(nombre), Some(sku), Some(categoria), Some(unidad), Some(costo), Some(precio), Some(stock_minimo), Some(codigo_barras), Some(stock_inicial))
            if errores.is_empty() =>
        {
            (
                Some(FilaValida {
                    numero: n,
                    nombre,
                    sku,
                    categoria,
                    unidad,
                    costo,
                    precio,
                    stock_minimo,
                    codigo_barras,
                    stock_inicial,
                    deposito: (!fila.deposito.is_empty()).then(|| fila.deposito.clone()),
                }),
                errores,
            )
        }
        _ => (None, errores),
    }
}

/// Valores repetidos dentro del archivo: el error va en la fila repetida y nombra la primera.
pub fn repetidos(valores: &[(usize, String)], columna: &str, etiqueta: &str) -> Vec<DetalleImportacion> {
    let mut primera: HashMap<&str, usize> = HashMap::new();
    let mut errores = Vec::new();
    for (fila, valor) in valores {
        match primera.get(valor.as_str()) {
            Some(anterior) => errores.push(detalle(
                *fila,
                columna,
                format!("{etiqueta} '{valor}' está repetido en el archivo (filas {anterior} y {fila})."),
            )),
            None => {
                primera.insert(valor.as_str(), *fila);
            }
        }
    }
    errores
}

/// Valida contra la BD (SKU y códigos existentes, categoría, depósito) y, en `aplicar` sin
/// errores, persiste todo en una transacción.
pub async fn importar(
    pool: &PgPool,
    bytes: &[u8],
    aplicar: bool,
    usuario: &Usuario,
) -> Result<ResumenImportacion, AppError> {
    let archivo = parsear_archivo(bytes)?;
    let filas_leidas = archivo.filas.len();

    let mut errores: Vec<DetalleImportacion> = Vec::new();
    let mut validas: Vec<FilaValida> = Vec::new();
    for fila in &archivo.filas {
        let (valida, mut errores_fila) = validar_fila(fila, archivo.dialecto.decimal);
        errores.append(&mut errores_fila);
        if let Some(v) = valida {
            validas.push(v);
        }
    }

    // Duplicados dentro del archivo (sobre las filas que pasaron la validación de celda).
    let skus: Vec<(usize, String)> = validas.iter().map(|v| (v.numero, v.sku.clone())).collect();
    errores.extend(repetidos(&skus, "sku", "El SKU"));
    let codigos: Vec<(usize, String)> = validas
        .iter()
        .filter_map(|v| v.codigo_barras.clone().map(|c| (v.numero, c)))
        .collect();
    errores.extend(repetidos(&codigos, "codigo_barras", "El código de barras"));

    // Contra la BD.
    let existentes_sku: Vec<String> = sqlx::query_scalar("SELECT sku FROM productos WHERE sku = ANY($1)")
        .bind(skus.iter().map(|s| s.1.clone()).collect::<Vec<_>>())
        .fetch_all(pool)
        .await?;
    let existentes_codigo: Vec<String> = sqlx::query_scalar(
        "SELECT codigo_barras FROM productos WHERE codigo_barras = ANY($1)",
    )
    .bind(codigos.iter().map(|c| c.1.clone()).collect::<Vec<_>>())
    .fetch_all(pool)
    .await?;
    let categorias: HashMap<String, i32> = sqlx::query_as::<_, (i32, String)>(
        "SELECT id, nombre FROM categorias WHERE activo",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, nombre)| (nombre.trim().to_lowercase(), id))
    .collect();
    let depositos: HashMap<String, i32> = sqlx::query_as::<_, (i32, String)>(
        "SELECT id, nombre FROM depositos WHERE activo",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, nombre)| (nombre.trim().to_lowercase(), id))
    .collect();

    let mut destinos: HashMap<usize, (i32, Option<i32>)> = HashMap::new();
    for v in &validas {
        if existentes_sku.contains(&v.sku) {
            errores.push(detalle(v.numero, "sku", format!("El SKU '{}' ya existe.", v.sku)));
        }
        if let Some(codigo) = &v.codigo_barras {
            if existentes_codigo.contains(codigo) {
                errores.push(detalle(
                    v.numero,
                    "codigo_barras",
                    format!("El código de barras '{codigo}' ya existe."),
                ));
            }
        }
        let categoria = categorias.get(&v.categoria.trim().to_lowercase()).copied();
        if categoria.is_none() {
            errores.push(detalle(
                v.numero,
                "categoria",
                format!("La categoría '{}' no existe.", v.categoria),
            ));
        }
        let deposito = if v.stock_inicial > 0 {
            match &v.deposito {
                Some(nombre) => {
                    let id = depositos.get(&nombre.trim().to_lowercase()).copied();
                    if id.is_none() {
                        errores.push(detalle(
                            v.numero,
                            "deposito",
                            format!("El depósito '{nombre}' no existe."),
                        ));
                    }
                    id
                }
                None => {
                    let id = depositos.get("principal").copied();
                    if id.is_none() {
                        errores.push(detalle(v.numero, "deposito", "No se encontró el depósito Principal."));
                    }
                    id
                }
            }
        } else {
            None
        };
        if let Some(categoria) = categoria {
            destinos.insert(v.numero, (categoria, deposito));
        }
    }

    errores.sort_by(|a, b| a.fila.cmp(&b.fila).then_with(|| a.columna.cmp(&b.columna)));
    let mut filas_con_error: Vec<usize> = errores.iter().map(|e| e.fila).collect();
    filas_con_error.sort_unstable();
    filas_con_error.dedup();
    let con_error = filas_con_error.len();
    let resumen = |creados: usize, aplicado: bool, errores: Vec<DetalleImportacion>| ResumenImportacion {
        filas_leidas,
        validas: filas_leidas - con_error,
        con_error,
        errores,
        creados,
        aplicado,
    };

    if !aplicar {
        return Ok(resumen(0, false, errores));
    }
    if !errores.is_empty() {
        return Err(AppError::ImportacionInvalida(errores));
    }

    let mut tx = pool.begin().await?;
    for v in &validas {
        let Some((categoria_id, deposito_id)) = destinos.get(&v.numero).copied() else {
            return Err(AppError::Interno("fila validada sin destino".into()));
        };
        let producto_id: i32 = sqlx::query_scalar(
            r#"INSERT INTO productos (nombre, sku, codigo_barras, categoria_id, unidad_medida, costo, precio_venta, stock_minimo)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"#,
        )
        .bind(&v.nombre)
        .bind(&v.sku)
        .bind(v.codigo_barras.as_deref())
        .bind(categoria_id)
        .bind(&v.unidad)
        .bind(v.costo)
        .bind(v.precio)
        .bind(v.stock_minimo)
        .fetch_one(&mut *tx)
        .await?;
        auditoria::registrar(&mut tx, auditoria::TABLA_PRODUCTOS, producto_id, auditoria::ACCION_CREAR, usuario.id).await?;

        if let (Some(deposito_id), true) = (deposito_id, v.stock_inicial > 0) {
            sqlx::query(
                r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
                   VALUES ($1, $2, NULL, 0)
                   ON CONFLICT ON CONSTRAINT uq_stock_producto_deposito_ubicacion DO NOTHING"#,
            )
            .bind(producto_id)
            .bind(deposito_id)
            .execute(&mut *tx)
            .await?;
            let resultante: i32 = sqlx::query_scalar(
                r#"UPDATE stock_por_deposito SET cantidad = cantidad + $3, actualizado_en = now()
                    WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL
                RETURNING cantidad"#,
            )
            .bind(producto_id)
            .bind(deposito_id)
            .bind(v.stock_inicial)
            .fetch_one(&mut *tx)
            .await?;
            let movimiento_id: i32 = sqlx::query_scalar(
                r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id)
                   VALUES ('ENTRADA', $1, 'IMPORTACION', $2, $3) RETURNING id"#,
            )
            .bind(deposito_id)
            .bind(format!("Importación CSV, fila {}", v.numero))
            .bind(usuario.id)
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query(
                r#"INSERT INTO movimiento_lineas (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante)
                   VALUES ($1, $2, NULL, $3, $4)"#,
            )
            .bind(movimiento_id)
            .bind(producto_id)
            .bind(v.stock_inicial)
            .bind(resultante)
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE productos SET stock_actual = $2 WHERE id = $1")
                .bind(producto_id)
                .bind(resultante)
                .execute(&mut *tx)
                .await?;
            auditoria::registrar(&mut tx, auditoria::TABLA_MOVIMIENTOS, movimiento_id, auditoria::ACCION_CREAR, usuario.id).await?;
        }
    }
    tx.commit().await?;
    Ok(resumen(validas.len(), true, Vec::new()))
}
