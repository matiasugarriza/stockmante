//! Servicio del catálogo: categorías (003, CU-CAT-01…06).
//!
//! Decisiones:
//! - No existe borrado físico: el retiro es `activo = FALSE`.
//! - La unicidad del nombre la impone la BD (`uq_categorias_nombre_normalizado`
//!   sobre `lower(btrim(nombre))` y, como respaldo, `categorias_nombre_key`);
//!   se mapea la violación `23505` sin pre-chequeo TOCTOU.
//! - `desactivar`/`reactivar` son idempotentes: un `UPDATE` que no encuentra la
//!   fila es 404, repetirlo sobre la misma fila es 204.

use crate::error::AppError;
use crate::models::producto::{validar_descripcion, validar_nombre_categoria, Categoria};
use sqlx::PgPool;

/// Cambios de un `PUT`: `None` conserva; `Some(None)` limpia la descripción.
#[derive(Debug, Default)]
pub struct CambiosCategoria {
    pub nombre: Option<String>,
    pub descripcion: Option<Option<String>>,
}

fn es_nombre_duplicado(error: &sqlx::Error) -> bool {
    error.as_database_error().is_some_and(|db| {
        db.code().as_deref() == Some("23505")
            && db.constraint().is_some_and(|c| {
                c == "uq_categorias_nombre_normalizado" || c == "categorias_nombre_key"
            })
    })
}

fn no_encontrada() -> AppError {
    AppError::RecursoNoEncontrado("La categoría no existe".into())
}

/// Lista plana ordenada por nombre; `None` = todas.
pub async fn listar(pool: &PgPool, activo: Option<bool>) -> Result<Vec<Categoria>, AppError> {
    let categorias = sqlx::query_as::<_, Categoria>(
        r#"SELECT id, nombre, descripcion, activo
             FROM categorias
            WHERE ($1::bool IS NULL OR activo = $1)
            ORDER BY nombre ASC, id ASC"#,
    )
    .bind(activo)
    .fetch_all(pool)
    .await?;
    Ok(categorias)
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<Categoria, AppError> {
    sqlx::query_as::<_, Categoria>(
        r#"SELECT id, nombre, descripcion, activo FROM categorias WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(no_encontrada)
}

pub async fn crear(
    pool: &PgPool,
    nombre: &str,
    descripcion: Option<&str>,
) -> Result<Categoria, AppError> {
    let nombre = validar_nombre_categoria(nombre)?;
    let descripcion = validar_descripcion(descripcion)?;
    let creada = sqlx::query_as::<_, Categoria>(
        r#"INSERT INTO categorias (nombre, descripcion)
                VALUES ($1, $2)
             RETURNING id, nombre, descripcion, activo"#,
    )
    .bind(&nombre)
    .bind(&descripcion)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if es_nombre_duplicado(&e) {
            AppError::CategoriaDuplicada
        } else {
            e.into()
        }
    })?;
    Ok(creada)
}

pub async fn actualizar(
    pool: &PgPool,
    id: i32,
    cambios: CambiosCategoria,
) -> Result<Categoria, AppError> {
    let nombre = match cambios.nombre {
        Some(valor) => Some(validar_nombre_categoria(&valor)?),
        None => None,
    };
    // `None` conserva; `Some(valor)` aplica (y normaliza `""` a NULL).
    let descripcion = match cambios.descripcion {
        Some(valor) => Some(validar_descripcion(valor.as_deref())?),
        None => None,
    };
    if nombre.is_none() && descripcion.is_none() {
        return Err(AppError::Validacion(
            "Indicá al menos un campo para actualizar.".into(),
        ));
    }
    let aplicar_descripcion = descripcion.is_some();
    let descripcion_valor = descripcion.flatten();

    let actualizada = sqlx::query_as::<_, Categoria>(
        r#"UPDATE categorias
              SET nombre = COALESCE($2::text, nombre),
                  descripcion = CASE WHEN $3 THEN $4::text ELSE descripcion END
            WHERE id = $1
        RETURNING id, nombre, descripcion, activo"#,
    )
    .bind(id)
    .bind(nombre.as_deref())
    .bind(aplicar_descripcion)
    .bind(descripcion_valor)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        if es_nombre_duplicado(&e) {
            AppError::CategoriaDuplicada
        } else {
            e.into()
        }
    })?
    .ok_or_else(no_encontrada)?;
    Ok(actualizada)
}

pub async fn desactivar(pool: &PgPool, id: i32) -> Result<(), AppError> {
    cambiar_estado(pool, id, false).await
}

pub async fn reactivar(pool: &PgPool, id: i32) -> Result<(), AppError> {
    cambiar_estado(pool, id, true).await
}

async fn cambiar_estado(pool: &PgPool, id: i32, activo: bool) -> Result<(), AppError> {
    let filas = sqlx::query("UPDATE categorias SET activo = $2 WHERE id = $1")
        .bind(id)
        .bind(activo)
        .execute(pool)
        .await?
        .rows_affected();
    if filas == 0 {
        Err(no_encontrada())
    } else {
        Ok(())
    }
}
