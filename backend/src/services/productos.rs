//! Servicio del catálogo: productos (003, CU-PROD-01…06).
//!
//! Decisiones:
//! - No existe borrado físico: el retiro es `activo = FALSE`.
//! - `crear`/`actualizar` corren en una transacción y validan la categoría
//!   dentro de ella: si la categoría es inexistente, está inactiva o el cambio
//!   se rechaza, no queda ninguna escritura parcial.
//! - La unicidad la impone la BD (`productos_sku_key`,
//!   `productos_codigo_barras_key`); se mapea `23505` sin pre-chequeo TOCTOU.
//! - `q` busca subcadena sobre `nombre` o `sku` escapando `%`, `_` y `\`.

use crate::error::AppError;
use crate::models::producto::{
    normalizar_sku, validar_codigo_barras, validar_monto, validar_nombre_producto, validar_sku,
    validar_stock_minimo, validar_unidad, ListaBusqueda, ListaProductos, Producto, ProductoFila,
};
use rust_decimal::Decimal;
use sqlx::{PgConnection, PgPool};
use super::auditoria;

const POR_PAGINA_DEFAULT: i64 = 20;
const POR_PAGINA_MAX: i64 = 100;
const Q_MAX: usize = 160;

/// Límite por defecto de la búsqueda de caja (004).
pub const LIMITE_BUSQUEDA_DEFAULT: i64 = 20;
/// Tope de resultados de la búsqueda de caja.
pub const LIMITE_BUSQUEDA_MAX: i64 = 50;
/// Nombre y categoría exigen al menos dos caracteres (spec 004, casos borde).
const Q_MIN_NOMBRE: usize = 2;

/// Filtros del listado de gestión (contracts/api.md §Productos).
#[derive(Debug, Default)]
pub struct FiltrosProductos {
    pub q: Option<String>,
    pub categoria_id: Option<i32>,
    pub activo: Option<bool>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

/// Campos de alta; cada uno se valida en el servicio.
#[derive(Debug)]
pub struct NuevoProducto {
    pub nombre: String,
    pub sku: String,
    pub codigo_barras: Option<String>,
    pub categoria_id: i32,
    pub unidad_medida: String,
    pub costo: Decimal,
    pub precio_venta: Decimal,
    pub stock_minimo: Option<i32>,
}

/// Cambios de un `PUT`: `None` conserva; `Some(None)` limpia el código de barras.
#[derive(Debug, Default)]
pub struct CambiosProducto {
    pub nombre: Option<String>,
    pub sku: Option<String>,
    pub codigo_barras: Option<Option<String>>,
    pub categoria_id: Option<i32>,
    pub unidad_medida: Option<String>,
    pub costo: Option<Decimal>,
    pub precio_venta: Option<Decimal>,
    pub stock_minimo: Option<i32>,
}

fn no_encontrado() -> AppError {
    AppError::RecursoNoEncontrado("El producto no existe".into())
}

fn categoria_invalida() -> AppError {
    AppError::Validacion("La categoría indicada no existe o está dada de baja.".into())
}

/// `23505` por SKU o código de barras; `23503` como red de seguridad.
fn mapear_unicidad(error: sqlx::Error, sku: Option<&str>) -> AppError {
    if let Some(db) = error.as_database_error() {
        match db.code().as_deref() {
            Some("23505") => {
                return match db.constraint() {
                    Some("productos_sku_key") => {
                        AppError::SkuDuplicado(sku.unwrap_or_default().to_string())
                    }
                    Some("productos_codigo_barras_key") => AppError::CodigoBarrasDuplicado,
                    _ => error.into(),
                };
            }
            Some("23503") => return categoria_invalida(),
            _ => {}
        }
    }
    error.into()
}

/// Escapa los metacaracteres de `LIKE`/`ILIKE` para que `q` sea literal.
fn escapar_like(termino: &str) -> String {
    let mut escapado = String::with_capacity(termino.len());
    for caracter in termino.chars() {
        if matches!(caracter, '\\' | '%' | '_') {
            escapado.push('\\');
        }
        escapado.push(caracter);
    }
    escapado
}

fn normalizar_paginacion(
    pagina: Option<i64>,
    por_pagina: Option<i64>,
) -> Result<(i64, i64, i64), AppError> {
    let pagina = pagina.unwrap_or(1);
    let por_pagina = por_pagina.unwrap_or(POR_PAGINA_DEFAULT);
    if pagina < 1 {
        return Err(AppError::Validacion(
            "La página debe ser mayor o igual a 1".into(),
        ));
    }
    if !(1..=POR_PAGINA_MAX).contains(&por_pagina) {
        return Err(AppError::Validacion(format!(
            "por_pagina debe estar entre 1 y {POR_PAGINA_MAX}"
        )));
    }
    let offset = pagina
        .checked_sub(1)
        .and_then(|previas| previas.checked_mul(por_pagina))
        .ok_or_else(|| AppError::Validacion("La página es demasiado grande".into()))?;
    Ok((pagina, por_pagina, offset))
}

/// Fila con la categoría ya unida (SQLx no mapea structs anidados).
async fn fila_por_id<'e, E>(ejecutor: E, id: i32) -> Result<ProductoFila, AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, ProductoFila>(
        r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                  p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                  p.costo, p.precio_venta, p.stock_minimo, p.activo
             FROM productos p
             JOIN categorias c ON c.id = p.categoria_id
            WHERE p.id = $1"#,
    )
    .bind(id)
    .fetch_optional(ejecutor)
    .await?
    .ok_or_else(no_encontrado)
}

async fn validar_categoria_activa(tx: &mut PgConnection, categoria_id: i32) -> Result<(), AppError> {
    let encontrada: Option<i32> =
        sqlx::query_scalar("SELECT id FROM categorias WHERE id = $1 AND activo = TRUE")
            .bind(categoria_id)
            .fetch_optional(&mut *tx)
            .await?;
    match encontrada {
        Some(_) => Ok(()),
        None => Err(categoria_invalida()),
    }
}

/// Listado paginado con filtros combinados por AND y orden estable.
pub async fn listar(
    pool: &PgPool,
    filtros: FiltrosProductos,
) -> Result<ListaProductos, AppError> {
    let (pagina, por_pagina, offset) =
        normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;

    let patron = match filtros.q {
        Some(valor) => {
            let recortado = valor.trim();
            if recortado.chars().count() > Q_MAX {
                return Err(AppError::Validacion(format!(
                    "El filtro q no puede superar los {Q_MAX} caracteres."
                )));
            }
            if recortado.is_empty() {
                None
            } else {
                Some(format!("%{}%", escapar_like(recortado)))
            }
        }
        None => None,
    };

    let categoria_id = match filtros.categoria_id {
        Some(id) if id <= 0 => {
            return Err(AppError::Validacion(
                "El filtro categoria_id debe ser un entero mayor a 0.".into(),
            ));
        }
        otro => otro,
    };

    let items = sqlx::query_as::<_, ProductoFila>(
        r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                  p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                  p.costo, p.precio_venta, p.stock_minimo, p.activo
             FROM productos p
             JOIN categorias c ON c.id = p.categoria_id
            WHERE ($1::bool IS NULL OR p.activo = $1)
              AND ($2::int IS NULL OR p.categoria_id = $2)
              AND ($3::text IS NULL
                   OR p.nombre ILIKE $3 ESCAPE '\'
                   OR p.sku ILIKE $3 ESCAPE '\')
            ORDER BY p.nombre ASC, p.id ASC
            LIMIT $4 OFFSET $5"#,
    )
    .bind(filtros.activo)
    .bind(categoria_id)
    .bind(patron.as_deref())
    .bind(por_pagina)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*)
             FROM productos p
            WHERE ($1::bool IS NULL OR p.activo = $1)
              AND ($2::int IS NULL OR p.categoria_id = $2)
              AND ($3::text IS NULL
                   OR p.nombre ILIKE $3 ESCAPE '\'
                   OR p.sku ILIKE $3 ESCAPE '\')"#,
    )
    .bind(filtros.activo)
    .bind(categoria_id)
    .bind(patron.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(ListaProductos {
        items: items.into_iter().map(Producto::from).collect(),
        total,
        pagina,
        por_pagina,
    })
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<Producto, AppError> {
    Ok(fila_por_id(pool, id).await?.into())
}

pub async fn crear(
    pool: &PgPool,
    datos: NuevoProducto,
    actor_id: i32,
) -> Result<Producto, AppError> {
    let nombre = validar_nombre_producto(&datos.nombre)?;
    let sku = validar_sku(&datos.sku)?;
    let codigo_barras = validar_codigo_barras(datos.codigo_barras.as_deref())?;
    let unidad_medida = validar_unidad(&datos.unidad_medida)?;
    let costo = validar_monto(datos.costo, "costo")?;
    let precio_venta = validar_monto(datos.precio_venta, "precio de venta")?;
    let stock_minimo = validar_stock_minimo(datos.stock_minimo.unwrap_or(0))?;
    let sku_error = sku.clone();

    let mut tx = pool.begin().await?;
    validar_categoria_activa(&mut tx, datos.categoria_id).await?;
    let id: i32 = sqlx::query_scalar(
        r#"INSERT INTO productos
                  (nombre, sku, codigo_barras, categoria_id, unidad_medida,
                   costo, precio_venta, stock_minimo)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id"#,
    )
    .bind(&nombre)
    .bind(&sku)
    .bind(&codigo_barras)
    .bind(datos.categoria_id)
    .bind(&unidad_medida)
    .bind(costo)
    .bind(precio_venta)
    .bind(stock_minimo)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| mapear_unicidad(e, Some(&sku_error)))?;
    let producto: Producto = fila_por_id(&mut *tx, id).await?.into();
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_PRODUCTOS,
        producto.id,
        auditoria::ACCION_CREAR,
        actor_id,
    )
    .await?;
    tx.commit().await?;
    Ok(producto)
}

pub async fn actualizar(
    pool: &PgPool,
    id: i32,
    cambios: CambiosProducto,
    actor_id: i32,
) -> Result<Producto, AppError> {
    let nombre = match cambios.nombre {
        Some(valor) => Some(validar_nombre_producto(&valor)?),
        None => None,
    };
    let sku = match cambios.sku {
        Some(valor) => Some(validar_sku(&valor)?),
        None => None,
    };
    let aplicar_codigo = cambios.codigo_barras.is_some();
    let codigo_barras = match cambios.codigo_barras.as_ref() {
        Some(valor) => validar_codigo_barras(valor.as_deref())?,
        None => None,
    };
    let unidad_medida = match cambios.unidad_medida {
        Some(valor) => Some(validar_unidad(&valor)?),
        None => None,
    };
    let costo = match cambios.costo {
        Some(valor) => Some(validar_monto(valor, "costo")?),
        None => None,
    };
    let precio_venta = match cambios.precio_venta {
        Some(valor) => Some(validar_monto(valor, "precio de venta")?),
        None => None,
    };
    let stock_minimo = match cambios.stock_minimo {
        Some(valor) => Some(validar_stock_minimo(valor)?),
        None => None,
    };
    let sin_cambios = nombre.is_none()
        && sku.is_none()
        && !aplicar_codigo
        && cambios.categoria_id.is_none()
        && unidad_medida.is_none()
        && costo.is_none()
        && precio_venta.is_none()
        && stock_minimo.is_none();
    if sin_cambios {
        return Err(AppError::Validacion(
            "Indicá al menos un campo para actualizar.".into(),
        ));
    }
    let sku_error = sku.clone();

    let mut tx = pool.begin().await?;
    let categoria_actual: i32 =
        sqlx::query_scalar("SELECT categoria_id FROM productos WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(no_encontrado)?;

    // Conservar la categoría actual es válido aunque esté inactiva; asignar
    // otra exige que exista y esté activa.
    if let Some(nueva) = cambios.categoria_id {
        if nueva != categoria_actual {
            validar_categoria_activa(&mut *tx, nueva).await?;
        }
    }

    let filas = sqlx::query(
        r#"UPDATE productos
              SET nombre = COALESCE($2::text, nombre),
                  sku = COALESCE($3::text, sku),
                  codigo_barras = CASE WHEN $4 THEN $5::text ELSE codigo_barras END,
                  categoria_id = COALESCE($6::int, categoria_id),
                  unidad_medida = COALESCE($7::text, unidad_medida),
                  costo = COALESCE($8::numeric, costo),
                  precio_venta = COALESCE($9::numeric, precio_venta),
                  stock_minimo = COALESCE($10::int, stock_minimo),
                  actualizado_en = now()
            WHERE id = $1"#,
    )
    .bind(id)
    .bind(nombre.as_deref())
    .bind(sku.as_deref())
    .bind(aplicar_codigo)
    .bind(codigo_barras.as_deref())
    .bind(cambios.categoria_id)
    .bind(unidad_medida.as_deref())
    .bind(costo)
    .bind(precio_venta)
    .bind(stock_minimo)
    .execute(&mut *tx)
    .await
    .map_err(|e| mapear_unicidad(e, sku_error.as_deref()))?
    .rows_affected();

    if filas == 0 {
        return Err(no_encontrado());
    }
    let producto: Producto = fila_por_id(&mut *tx, id).await?.into();
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_PRODUCTOS,
        producto.id,
        auditoria::ACCION_ACTUALIZAR,
        actor_id,
    )
    .await?;
    tx.commit().await?;
    Ok(producto)
}

pub async fn desactivar(pool: &PgPool, id: i32, actor_id: i32) -> Result<(), AppError> {
    cambiar_estado(pool, id, false, actor_id).await
}

pub async fn reactivar(pool: &PgPool, id: i32, actor_id: i32) -> Result<(), AppError> {
    cambiar_estado(pool, id, true, actor_id).await
}

async fn cambiar_estado(pool: &PgPool, id: i32, activo: bool, actor_id: i32) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let filas = sqlx::query("UPDATE productos SET activo = $2, actualizado_en = now() WHERE id = $1")
        .bind(id)
        .bind(activo)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if filas == 0 {
        return Err(no_encontrado());
    }
    let accion = if activo {
        auditoria::ACCION_REACTIVAR
    } else {
        auditoria::ACCION_DESACTIVAR
    };
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_PRODUCTOS,
        id,
        accion,
        actor_id,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Campo de la búsqueda de caja (004). Ausente equivale a `Todos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampoBusqueda {
    Sku,
    Codigo,
    Nombre,
    Categoria,
    Todos,
}

impl CampoBusqueda {
    /// Parsea el query param `campo`; cualquier otro valor es 422.
    pub fn parsear(valor: Option<&str>) -> Result<Self, AppError> {
        match valor.map(str::trim) {
            None | Some("") => Ok(Self::Todos),
            Some(valor) => match valor.to_ascii_lowercase().as_str() {
                "todos" => Ok(Self::Todos),
                "sku" => Ok(Self::Sku),
                "codigo" => Ok(Self::Codigo),
                "nombre" => Ok(Self::Nombre),
                "categoria" => Ok(Self::Categoria),
                _ => Err(AppError::Validacion(
                    "El campo de búsqueda debe ser sku, codigo, nombre, categoria o todos.".into(),
                )),
            },
        }
    }
}

/// Parámetros ya extraídos del query string de `GET /api/productos/buscar`.
#[derive(Debug)]
pub struct BusquedaProductos {
    pub q: String,
    pub campo: CampoBusqueda,
    pub incluir_inactivos: bool,
    pub categoria_id: Option<i32>,
    pub limite: Option<i64>,
}

/// Búsqueda de caja (FR-011/FR-013): exacta para SKU/código y por trigramas
/// para nombre/categoría. Los inactivos se excluyen salvo pedido explícito.
pub async fn buscar(
    pool: &PgPool,
    busqueda: BusquedaProductos,
) -> Result<ListaBusqueda, AppError> {
    let termino = busqueda.q.trim();
    if termino.chars().count() > Q_MAX {
        return Err(AppError::Validacion(format!(
            "El término de búsqueda no puede superar los {Q_MAX} caracteres."
        )));
    }
    let categoria_id = match busqueda.categoria_id {
        Some(id) if id <= 0 => {
            return Err(AppError::Validacion(
                "El filtro categoria_id debe ser un entero mayor a 0.".into(),
            ));
        }
        otro => otro,
    };
    let limite = busqueda.limite.unwrap_or(LIMITE_BUSQUEDA_DEFAULT);
    if !(1..=LIMITE_BUSQUEDA_MAX).contains(&limite) {
        return Err(AppError::Validacion(format!(
            "El límite debe estar entre 1 y {LIMITE_BUSQUEDA_MAX}."
        )));
    }

    // Filtrar solo por categoría es válido: es el escenario 4 sin texto.
    if termino.is_empty() {
        let Some(categoria_id) = categoria_id else {
            return Err(AppError::Validacion(
                "Ingresá un término de búsqueda o elegí una categoría.".into(),
            ));
        };
        let filas = sqlx::query_as::<_, ProductoFila>(
            r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                      p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                      p.costo, p.precio_venta, p.stock_minimo, p.activo
                 FROM productos p
                 JOIN categorias c ON c.id = p.categoria_id
                WHERE p.categoria_id = $1
                  AND ($2::bool OR p.activo)
                ORDER BY p.nombre ASC, p.id ASC
                LIMIT $3"#,
        )
        .bind(categoria_id)
        .bind(busqueda.incluir_inactivos)
        .bind(limite)
        .fetch_all(pool)
        .await?;
        return Ok(ListaBusqueda {
            items: filas.into_iter().map(Producto::from).collect(),
        });
    }

    let termino = termino.to_string();
    let largo = termino.chars().count();
    if matches!(
        busqueda.campo,
        CampoBusqueda::Nombre | CampoBusqueda::Categoria
    ) && largo < Q_MIN_NOMBRE
    {
        return Err(AppError::Validacion(
            "La búsqueda por nombre o categoría requiere al menos 2 caracteres.".into(),
        ));
    }

    let patron = format!("%{}%", escapar_like(&termino));
    let incluir = busqueda.incluir_inactivos;

    let filas = match busqueda.campo {
        CampoBusqueda::Sku => {
            let sku = normalizar_sku(&termino);
            sqlx::query_as::<_, ProductoFila>(
                r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                          p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                          p.costo, p.precio_venta, p.stock_minimo, p.activo
                     FROM productos p
                     JOIN categorias c ON c.id = p.categoria_id
                    WHERE p.sku = $1
                      AND ($2::int IS NULL OR p.categoria_id = $2)
                      AND ($3::bool OR p.activo)
                    ORDER BY p.nombre ASC, p.id ASC
                    LIMIT $4"#,
            )
            .bind(&sku)
            .bind(categoria_id)
            .bind(incluir)
            .bind(limite)
            .fetch_all(pool)
            .await?
        }
        CampoBusqueda::Codigo => {
            sqlx::query_as::<_, ProductoFila>(
                r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                          p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                          p.costo, p.precio_venta, p.stock_minimo, p.activo
                     FROM productos p
                     JOIN categorias c ON c.id = p.categoria_id
                    WHERE p.codigo_barras = $1
                      AND ($2::int IS NULL OR p.categoria_id = $2)
                      AND ($3::bool OR p.activo)
                    ORDER BY p.nombre ASC, p.id ASC
                    LIMIT $4"#,
            )
            .bind(&termino)
            .bind(categoria_id)
            .bind(incluir)
            .bind(limite)
            .fetch_all(pool)
            .await?
        }
        CampoBusqueda::Nombre => {
            sqlx::query_as::<_, ProductoFila>(
                r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                          p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                          p.costo, p.precio_venta, p.stock_minimo, p.activo
                     FROM productos p
                     JOIN categorias c ON c.id = p.categoria_id
                    WHERE (p.nombre % $1 OR p.nombre ILIKE $2 ESCAPE '\')
                      AND ($3::int IS NULL OR p.categoria_id = $3)
                      AND ($4::bool OR p.activo)
                    ORDER BY similarity(p.nombre, $1) DESC, p.nombre ASC, p.id ASC
                    LIMIT $5"#,
            )
            .bind(&termino)
            .bind(&patron)
            .bind(categoria_id)
            .bind(incluir)
            .bind(limite)
            .fetch_all(pool)
            .await?
        }
        CampoBusqueda::Categoria => {
            sqlx::query_as::<_, ProductoFila>(
                r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                          p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                          p.costo, p.precio_venta, p.stock_minimo, p.activo
                     FROM productos p
                     JOIN categorias c ON c.id = p.categoria_id
                    WHERE (c.nombre % $1 OR c.nombre ILIKE $2 ESCAPE '\')
                      AND ($3::int IS NULL OR p.categoria_id = $3)
                      AND ($4::bool OR p.activo)
                    ORDER BY similarity(c.nombre, $1) DESC, p.nombre ASC, p.id ASC
                    LIMIT $5"#,
            )
            .bind(&termino)
            .bind(&patron)
            .bind(categoria_id)
            .bind(incluir)
            .bind(limite)
            .fetch_all(pool)
            .await?
        }
        CampoBusqueda::Todos => {
            // Con un solo carácter solo tienen sentido las igualdades exactas.
            let por_nombre = largo >= Q_MIN_NOMBRE;
            let sku = normalizar_sku(&termino);
            sqlx::query_as::<_, ProductoFila>(
                r#"SELECT p.id, p.nombre, p.sku, p.codigo_barras,
                          p.categoria_id, c.nombre AS categoria_nombre, p.unidad_medida,
                          p.costo, p.precio_venta, p.stock_minimo, p.activo
                     FROM productos p
                     JOIN categorias c ON c.id = p.categoria_id
                    WHERE (
                            p.sku = $1
                         OR p.codigo_barras = $2
                         OR ($3::bool AND (p.nombre % $4 OR p.nombre ILIKE $5 ESCAPE '\'))
                          )
                      AND ($6::int IS NULL OR p.categoria_id = $6)
                      AND ($7::bool OR p.activo)
                    ORDER BY CASE
                               WHEN p.sku = $1 THEN 0
                               WHEN p.codigo_barras = $2 THEN 1
                               ELSE 2
                             END,
                             similarity(p.nombre, $4) DESC,
                             p.nombre ASC,
                             p.id ASC
                    LIMIT $8"#,
            )
            .bind(&sku)
            .bind(&termino)
            .bind(por_nombre)
            .bind(&termino)
            .bind(&patron)
            .bind(categoria_id)
            .bind(incluir)
            .bind(limite)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(ListaBusqueda {
        items: filas.into_iter().map(Producto::from).collect(),
    })
}
