//! Modelos y reglas puras del catálogo (003): categorías y productos.
//!
//! Toda la validación vive acá para que servicios y handlers compartan el mismo
//! mensaje en español (contracts/validation.md). No hay `unwrap`, `expect` ni
//! `panic!`: cada regla devuelve `Result<_, AppError>`.

use crate::error::AppError;
use rust_decimal::Decimal;
use serde::Serialize;

/// Unidades de medida admitidas (lista cerrada, contracts/validation.md).
pub const UNIDADES: [&str; 4] = ["UNIDAD", "KG", "METRO", "CAJA"];

/// Tope superior de cualquier monto (`< 10^10`, contracts/validation.md).
pub const MONTO_MAXIMO: i64 = 10_000_000_000;

/// Categoría tal como la devuelve la API.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Categoria {
    pub id: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub activo: bool,
}

/// Referencia mínima a la categoría dentro de un producto.
#[derive(Debug, Clone, Serialize)]
pub struct CategoriaBreve {
    pub id: i32,
    pub nombre: String,
}

/// Producto tal como lo devuelve la API.
#[derive(Debug, Clone, Serialize)]
pub struct Producto {
    pub id: i32,
    pub nombre: String,
    pub sku: String,
    pub codigo_barras: Option<String>,
    pub categoria: CategoriaBreve,
    pub unidad_medida: String,
    pub costo: Decimal,
    pub precio_venta: Decimal,
    pub stock_minimo: i32,
    pub activo: bool,
}

/// Fila plana del `SELECT` con `JOIN categorias`: SQLx no mapea structs anidados.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProductoFila {
    pub id: i32,
    pub nombre: String,
    pub sku: String,
    pub codigo_barras: Option<String>,
    pub categoria_id: i32,
    pub categoria_nombre: String,
    pub unidad_medida: String,
    pub costo: Decimal,
    pub precio_venta: Decimal,
    pub stock_minimo: i32,
    pub activo: bool,
}

impl From<ProductoFila> for Producto {
    fn from(fila: ProductoFila) -> Self {
        Producto {
            id: fila.id,
            nombre: fila.nombre,
            sku: fila.sku,
            codigo_barras: fila.codigo_barras,
            categoria: CategoriaBreve {
                id: fila.categoria_id,
                nombre: fila.categoria_nombre,
            },
            unidad_medida: fila.unidad_medida,
            costo: fila.costo,
            precio_venta: fila.precio_venta,
            stock_minimo: fila.stock_minimo,
            activo: fila.activo,
        }
    }
}

/// Página del listado de gestión de productos.
#[derive(Debug, Clone, Serialize)]
pub struct ListaProductos {
    pub items: Vec<Producto>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

/// `trim` + mayúsculas: la unicidad se evalúa ya normalizada.
pub fn normalizar_sku(sku: &str) -> String {
    sku.trim().to_uppercase()
}

/// 3–30 caracteres, `^[A-Z0-9][A-Z0-9-]{2,29}$` sobre el SKU normalizado.
pub fn validar_sku(sku: &str) -> Result<String, AppError> {
    let normalizado = normalizar_sku(sku);
    let mut caracteres = normalizado.chars();
    let primero_valido = caracteres
        .next()
        .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    let resto_valido = caracteres.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-');
    let longitud = normalizado.chars().count();
    if (3..=30).contains(&longitud) && primero_valido && resto_valido {
        Ok(normalizado)
    } else {
        Err(AppError::Validacion(
            "El SKU debe tener entre 3 y 30 caracteres en mayúsculas, dígitos y guiones, y empezar con letra o dígito.".into(),
        ))
    }
}

/// `trim`; vacío o solo espacios → `None` (se persiste `NULL`).
pub fn normalizar_codigo_barras(valor: Option<&str>) -> Option<String> {
    valor
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// 8–64 alfanuméricos; opcional (vacío → `None`).
pub fn validar_codigo_barras(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match normalizar_codigo_barras(valor) {
        None => Ok(None),
        Some(codigo) => {
            let longitud = codigo.len();
            let alfanumerico = codigo.bytes().all(|b| b.is_ascii_alphanumeric());
            if (8..=64).contains(&longitud) && alfanumerico {
                Ok(Some(codigo))
            } else {
                Err(AppError::Validacion(
                    "El código de barras debe tener entre 8 y 64 caracteres alfanuméricos.".into(),
                ))
            }
        }
    }
}

/// `trim`; vacío o solo espacios → `None`.
pub fn normalizar_descripcion(valor: Option<&str>) -> Option<String> {
    valor
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// Hasta 255 caracteres tras normalizar.
pub fn validar_descripcion(valor: Option<&str>) -> Result<Option<String>, AppError> {
    match normalizar_descripcion(valor) {
        None => Ok(None),
        Some(descripcion) if descripcion.chars().count() <= 255 => Ok(Some(descripcion)),
        Some(_) => Err(AppError::Validacion(
            "La descripción no puede superar los 255 caracteres.".into(),
        )),
    }
}

/// `trim`: la unicidad compara sin espacios de extremo.
pub fn normalizar_nombre_categoria(nombre: &str) -> String {
    nombre.trim().to_string()
}

/// 1–80 caracteres tras normalizar.
pub fn validar_nombre_categoria(nombre: &str) -> Result<String, AppError> {
    let normalizado = normalizar_nombre_categoria(nombre);
    let longitud = normalizado.chars().count();
    if (1..=80).contains(&longitud) {
        Ok(normalizado)
    } else if normalizado.is_empty() {
        Err(AppError::Validacion(
            "El nombre de la categoría es obligatorio.".into(),
        ))
    } else {
        Err(AppError::Validacion(
            "El nombre de la categoría no puede superar los 80 caracteres.".into(),
        ))
    }
}

/// 1–160 caracteres tras normalizar.
pub fn validar_nombre_producto(nombre: &str) -> Result<String, AppError> {
    let normalizado = nombre.trim().to_string();
    let longitud = normalizado.chars().count();
    if (1..=160).contains(&longitud) {
        Ok(normalizado)
    } else if normalizado.is_empty() {
        Err(AppError::Validacion("El nombre del producto es obligatorio.".into()))
    } else {
        Err(AppError::Validacion(
            "El nombre del producto no puede superar los 160 caracteres.".into(),
        ))
    }
}

/// Lista cerrada `UNIDAD`, `KG`, `METRO`, `CAJA` (comparación exacta tras `trim`).
pub fn validar_unidad(unidad: &str) -> Result<String, AppError> {
    let normalizada = unidad.trim();
    if UNIDADES.contains(&normalizada) {
        Ok(normalizada.to_string())
    } else {
        Err(AppError::Validacion(
            "La unidad de medida debe ser UNIDAD, KG, METRO o CAJA.".into(),
        ))
    }
}

/// Monto ≥ 0, hasta 2 decimales y menor a `10^10`.
pub fn validar_monto(valor: Decimal, campo: &str) -> Result<Decimal, AppError> {
    let fuera_de_rango = valor.is_sign_negative()
        || valor.scale() > 2
        || valor >= Decimal::from(MONTO_MAXIMO);
    if fuera_de_rango {
        Err(AppError::Validacion(format!(
            "El {campo} debe ser un monto mayor o igual a 0, con hasta 2 decimales y menor a 10.000.000.000."
        )))
    } else {
        Ok(valor)
    }
}

/// Entero ≥ 0 (el tipo ya garantiza la parte entera al deserializar).
pub fn validar_stock_minimo(valor: i32) -> Result<i32, AppError> {
    if valor < 0 {
        Err(AppError::Validacion(
            "El stock mínimo debe ser un entero mayor o igual a 0.".into(),
        ))
    } else {
        Ok(valor)
    }
}
