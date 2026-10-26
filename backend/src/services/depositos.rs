//! Servicio de depósitos, ubicaciones jerárquicas y asignaciones producto–ubicación
//! (005, FR-015…FR-018).
//!
//! Decisiones:
//! - `ubicaciones.deposito_id` es NOT NULL desde 0003: no puede existir una
//!   ubicación huérfana. El servicio además exige que el padre pertenezca al
//!   mismo depósito y esté activo, y evita ciclos al mover una ubicación.
//! - El borrado físico de un depósito o ubicación se bloquea con 409 si hay
//!   dependencias; la alternativa es la baja lógica (`activo = FALSE`).
//! - La asignación producto–ubicación reemplaza el conjunto dentro de una
//!   transacción y exige ubicaciones y depósitos activos.
//! - Toda escritura corre en `pool.begin()`; no hay `unwrap`/`expect`/`panic!`.

use crate::error::AppError;
use crate::models::deposito::{
    validar_direccion, validar_identificador, validar_nombre_deposito, validar_nombre_ubicacion,
    validar_tipo_ubicacion, Deposito, Ubicacion,
};
use sqlx::{PgConnection, PgPool};

/// Alta de depósito; el servicio valida cada campo.
#[derive(Debug)]
pub struct NuevoDeposito {
    pub nombre: String,
    pub identificador: String,
    pub direccion: Option<String>,
}

/// `PUT` de depósito: `None` conserva; `Some(None)` limpia la dirección.
#[derive(Debug, Default)]
pub struct CambiosDeposito {
    pub nombre: Option<String>,
    pub identificador: Option<String>,
    pub direccion: Option<Option<String>>,
}

/// Alta de ubicación; `deposito_id` es obligatorio.
#[derive(Debug)]
pub struct NuevaUbicacion {
    pub deposito_id: i32,
    pub padre_id: Option<i32>,
    pub nombre: String,
    pub tipo: String,
}

/// `PUT` de ubicación: `padre_id: Some(None)` mueve a la raíz.
#[derive(Debug, Default)]
pub struct CambiosUbicacion {
    pub nombre: Option<String>,
    pub tipo: Option<String>,
    pub padre_id: Option<Option<i32>>,
    pub deposito_id: Option<i32>,
}

fn deposito_no_encontrado() -> AppError {
    AppError::RecursoNoEncontrado("El depósito no existe".into())
}

fn ubicacion_no_encontrada() -> AppError {
    AppError::RecursoNoEncontrado("La ubicación no existe".into())
}

fn producto_no_encontrado() -> AppError {
    AppError::RecursoNoEncontrado("El producto no existe".into())
}

fn mapear_unicidad_deposito(error: sqlx::Error) -> AppError {
    if let Some(db) = error.as_database_error() {
        if db.code().as_deref() == Some("23505") {
            return match db.constraint() {
                Some("depositos_nombre_key") => AppError::DepositoNombreDuplicado,
                Some("depositos_identificador_key") => AppError::DepositoIdentificadorDuplicado,
                _ => error.into(),
            };
        }
    }
    error.into()
}

fn mapear_unicidad_ubicacion(error: sqlx::Error) -> AppError {
    if let Some(db) = error.as_database_error() {
        match db.code().as_deref() {
            Some("23505") => return AppError::UbicacionDuplicada,
            Some("23503") => return AppError::UbicacionHuerfana,
            _ => {}
        }
    }
    error.into()
}

async fn fila_deposito<'e, E>(ejecutor: E, id: i32) -> Result<Deposito, AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Deposito>(
        "SELECT id, nombre, identificador, direccion, activo FROM depositos WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(ejecutor)
    .await?
    .ok_or_else(deposito_no_encontrado)
}

async fn fila_ubicacion<'e, E>(ejecutor: E, id: i32) -> Result<Ubicacion, AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Ubicacion>(
        r#"SELECT id, deposito_id, padre_id, nombre, tipo, activo
             FROM ubicaciones
            WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(ejecutor)
    .await?
    .ok_or_else(ubicacion_no_encontrada)
}

async fn validar_deposito_activo(tx: &mut PgConnection, deposito_id: i32) -> Result<(), AppError> {
    let activo: Option<bool> = sqlx::query_scalar("SELECT activo FROM depositos WHERE id = $1")
        .bind(deposito_id)
        .fetch_optional(&mut *tx)
        .await?;
    match activo {
        Some(true) => Ok(()),
        _ => Err(AppError::UbicacionHuerfana),
    }
}

/// Un padre solo es válido si existe, está activo y vive en el mismo depósito.
async fn validar_padre(
    tx: &mut PgConnection,
    deposito_id: i32,
    padre_id: i32,
    ubicacion_actual: Option<i32>,
) -> Result<(), AppError> {
    if ubicacion_actual == Some(padre_id) {
        return Err(AppError::Validacion(
            "Una ubicación no puede ser padre de sí misma.".into(),
        ));
    }
    let fila: Option<(i32, bool)> =
        sqlx::query_as("SELECT deposito_id, activo FROM ubicaciones WHERE id = $1")
            .bind(padre_id)
            .fetch_optional(&mut *tx)
            .await?;
    match fila {
        Some((deposito_padre, true)) if deposito_padre == deposito_id => Ok(()),
        _ => Err(AppError::UbicacionHuerfana),
    }
}

/// `true` si `candidato` está en la descendencia de `ancestro` (o es él mismo).
async fn es_descendiente(
    tx: &mut PgConnection,
    ancestro: i32,
    candidato: i32,
) -> Result<bool, AppError> {
    let existe: bool = sqlx::query_scalar(
        r#"WITH RECURSIVE cadena AS (
               SELECT id, padre_id FROM ubicaciones WHERE id = $2
               UNION ALL
               SELECT u.id, u.padre_id
                 FROM ubicaciones u
                 JOIN cadena c ON u.id = c.padre_id
           )
           SELECT EXISTS (SELECT 1 FROM cadena WHERE id = $1)"#,
    )
    .bind(ancestro)
    .bind(candidato)
    .fetch_one(&mut *tx)
    .await?;
    Ok(existe)
}

pub async fn listar(pool: &PgPool, activo: Option<bool>) -> Result<Vec<Deposito>, AppError> {
    let depositos = sqlx::query_as::<_, Deposito>(
        r#"SELECT id, nombre, identificador, direccion, activo
             FROM depositos
            WHERE ($1::bool IS NULL OR activo = $1)
            ORDER BY nombre ASC, id ASC"#,
    )
    .bind(activo)
    .fetch_all(pool)
    .await?;
    Ok(depositos)
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<Deposito, AppError> {
    fila_deposito(pool, id).await
}

pub async fn crear(pool: &PgPool, datos: NuevoDeposito) -> Result<Deposito, AppError> {
    let nombre = validar_nombre_deposito(&datos.nombre)?;
    let identificador = validar_identificador(&datos.identificador)?;
    let direccion = validar_direccion(datos.direccion.as_deref())?;

    let mut tx = pool.begin().await?;
    let id: i32 = sqlx::query_scalar(
        r#"INSERT INTO depositos (nombre, identificador, direccion)
           VALUES ($1, $2, $3)
        RETURNING id"#,
    )
    .bind(&nombre)
    .bind(&identificador)
    .bind(&direccion)
    .fetch_one(&mut *tx)
    .await
    .map_err(mapear_unicidad_deposito)?;
    let deposito = fila_deposito(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(deposito)
}

pub async fn actualizar(
    pool: &PgPool,
    id: i32,
    cambios: CambiosDeposito,
) -> Result<Deposito, AppError> {
    let nombre = match cambios.nombre {
        Some(valor) => Some(validar_nombre_deposito(&valor)?),
        None => None,
    };
    let identificador = match cambios.identificador {
        Some(valor) => Some(validar_identificador(&valor)?),
        None => None,
    };
    let aplicar_direccion = cambios.direccion.is_some();
    let direccion = validar_direccion(cambios.direccion.as_ref().and_then(|v| v.as_deref()))?;
    if nombre.is_none() && identificador.is_none() && !aplicar_direccion {
        return Err(AppError::Validacion(
            "Indicá al menos un campo para actualizar.".into(),
        ));
    }

    let mut tx = pool.begin().await?;
    let filas = sqlx::query(
        r#"UPDATE depositos
              SET nombre = COALESCE($2::text, nombre),
                  identificador = COALESCE($3::text, identificador),
                  direccion = CASE WHEN $4 THEN $5::text ELSE direccion END
            WHERE id = $1"#,
    )
    .bind(id)
    .bind(nombre.as_deref())
    .bind(identificador.as_deref())
    .bind(aplicar_direccion)
    .bind(direccion.as_deref())
    .execute(&mut *tx)
    .await
    .map_err(mapear_unicidad_deposito)?
    .rows_affected();
    if filas == 0 {
        return Err(deposito_no_encontrado());
    }
    let deposito = fila_deposito(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(deposito)
}

pub async fn desactivar(pool: &PgPool, id: i32) -> Result<(), AppError> {
    cambiar_estado(pool, id, false).await
}

pub async fn reactivar(pool: &PgPool, id: i32) -> Result<(), AppError> {
    cambiar_estado(pool, id, true).await
}

async fn cambiar_estado(pool: &PgPool, id: i32, activo: bool) -> Result<(), AppError> {
    let filas = sqlx::query("UPDATE depositos SET activo = $2 WHERE id = $1")
        .bind(id)
        .bind(activo)
        .execute(pool)
        .await?
        .rows_affected();
    if filas == 0 {
        Err(deposito_no_encontrado())
    } else {
        Ok(())
    }
}

/// FR-018: borrado físico solo sin ubicaciones ni asignaciones.
pub async fn eliminar(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let existe: Option<i32> = sqlx::query_scalar("SELECT id FROM depositos WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if existe.is_none() {
        return Err(deposito_no_encontrado());
    }

    let ubicaciones: i64 =
        sqlx::query_scalar("SELECT count(*) FROM ubicaciones WHERE deposito_id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    let asignaciones: i64 = sqlx::query_scalar(
        r#"SELECT count(*)
             FROM producto_ubicacion_permitida p
             JOIN ubicaciones u ON u.id = p.ubicacion_id
            WHERE u.deposito_id = $1"#,
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if ubicaciones > 0 || asignaciones > 0 {
        return Err(AppError::DepositoConDependencias);
    }

    sqlx::query("DELETE FROM depositos WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| match e.as_database_error().and_then(|db| db.code()) {
            Some(codigo) if codigo.as_ref() == "23503" => AppError::DepositoConDependencias,
            _ => e.into(),
        })?;
    tx.commit().await?;
    Ok(())
}

pub async fn listar_ubicaciones(
    pool: &PgPool,
    deposito_id: i32,
) -> Result<Vec<Ubicacion>, AppError> {
    let existe: Option<i32> = sqlx::query_scalar("SELECT id FROM depositos WHERE id = $1")
        .bind(deposito_id)
        .fetch_optional(pool)
        .await?;
    if existe.is_none() {
        return Err(deposito_no_encontrado());
    }
    let ubicaciones = sqlx::query_as::<_, Ubicacion>(
        r#"SELECT id, deposito_id, padre_id, nombre, tipo, activo
             FROM ubicaciones
            WHERE deposito_id = $1
            ORDER BY nombre ASC, id ASC"#,
    )
    .bind(deposito_id)
    .fetch_all(pool)
    .await?;
    Ok(ubicaciones)
}

pub async fn obtener_ubicacion(pool: &PgPool, id: i32) -> Result<Ubicacion, AppError> {
    fila_ubicacion(pool, id).await
}

pub async fn crear_ubicacion(
    pool: &PgPool,
    datos: NuevaUbicacion,
) -> Result<Ubicacion, AppError> {
    let nombre = validar_nombre_ubicacion(&datos.nombre)?;
    let tipo = validar_tipo_ubicacion(&datos.tipo)?;

    let mut tx = pool.begin().await?;
    validar_deposito_activo(&mut tx, datos.deposito_id).await?;
    if let Some(padre_id) = datos.padre_id {
        validar_padre(&mut tx, datos.deposito_id, padre_id, None).await?;
    }

    let id: i32 = sqlx::query_scalar(
        r#"INSERT INTO ubicaciones (deposito_id, padre_id, nombre, tipo)
           VALUES ($1, $2, $3, $4)
        RETURNING id"#,
    )
    .bind(datos.deposito_id)
    .bind(datos.padre_id)
    .bind(&nombre)
    .bind(&tipo)
    .fetch_one(&mut *tx)
    .await
    .map_err(mapear_unicidad_ubicacion)?;
    let ubicacion = fila_ubicacion(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(ubicacion)
}

pub async fn actualizar_ubicacion(
    pool: &PgPool,
    id: i32,
    cambios: CambiosUbicacion,
) -> Result<Ubicacion, AppError> {
    let nombre = match cambios.nombre {
        Some(valor) => Some(validar_nombre_ubicacion(&valor)?),
        None => None,
    };
    let tipo = match cambios.tipo {
        Some(valor) => Some(validar_tipo_ubicacion(&valor)?),
        None => None,
    };
    let sin_cambios = nombre.is_none()
        && tipo.is_none()
        && cambios.padre_id.is_none()
        && cambios.deposito_id.is_none();
    if sin_cambios {
        return Err(AppError::Validacion(
            "Indicá al menos un campo para actualizar.".into(),
        ));
    }

    let mut tx = pool.begin().await?;
    let actual = fila_ubicacion(&mut *tx, id).await?;
    let deposito_destino = cambios.deposito_id.unwrap_or(actual.deposito_id);
    if deposito_destino <= 0 {
        return Err(AppError::Validacion(
            "El depósito debe ser un entero mayor a 0.".into(),
        ));
    }

    let cambia_deposito = deposito_destino != actual.deposito_id;
    if cambia_deposito {
        validar_deposito_activo(&mut tx, deposito_destino).await?;
        let hijos: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM ubicaciones WHERE padre_id = $1)")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        let asignaciones: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM producto_ubicacion_permitida WHERE ubicacion_id = $1)",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if hijos || asignaciones {
            return Err(AppError::UbicacionConDependencias);
        }
    }

    // Al cambiar de depósito sin indicar padre, la ubicación pasa a ser raíz:
    // conservar el padre anterior violaría la regla de mismo depósito.
    let padre_destino = match cambios.padre_id {
        Some(valor) => valor,
        None if cambia_deposito => None,
        None => actual.padre_id,
    };
    if let Some(padre_id) = padre_destino {
        validar_padre(&mut tx, deposito_destino, padre_id, Some(id)).await?;
        if es_descendiente(&mut tx, id, padre_id).await? {
            return Err(AppError::Validacion(
                "No se puede mover una ubicación dentro de sí misma o de una de sus hijas.".into(),
            ));
        }
    }

    let filas = sqlx::query(
        r#"UPDATE ubicaciones
              SET nombre = COALESCE($2::text, nombre),
                  tipo = COALESCE($3::text, tipo),
                  deposito_id = $4,
                  padre_id = $5
            WHERE id = $1"#,
    )
    .bind(id)
    .bind(nombre.as_deref())
    .bind(tipo.as_deref())
    .bind(deposito_destino)
    .bind(padre_destino)
    .execute(&mut *tx)
    .await
    .map_err(mapear_unicidad_ubicacion)?
    .rows_affected();
    if filas == 0 {
        return Err(ubicacion_no_encontrada());
    }
    let ubicacion = fila_ubicacion(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(ubicacion)
}

pub async fn desactivar_ubicacion(pool: &PgPool, id: i32) -> Result<(), AppError> {
    cambiar_estado_ubicacion(pool, id, false).await
}

pub async fn reactivar_ubicacion(pool: &PgPool, id: i32) -> Result<(), AppError> {
    cambiar_estado_ubicacion(pool, id, true).await
}

async fn cambiar_estado_ubicacion(pool: &PgPool, id: i32, activo: bool) -> Result<(), AppError> {
    let filas = sqlx::query("UPDATE ubicaciones SET activo = $2 WHERE id = $1")
        .bind(id)
        .bind(activo)
        .execute(pool)
        .await?
        .rows_affected();
    if filas == 0 {
        Err(ubicacion_no_encontrada())
    } else {
        Ok(())
    }
}

/// Borrado físico solo sin hijos ni productos asignados.
pub async fn eliminar_ubicacion(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let existe: Option<i32> =
        sqlx::query_scalar("SELECT id FROM ubicaciones WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    if existe.is_none() {
        return Err(ubicacion_no_encontrada());
    }

    let hijos: i64 = sqlx::query_scalar("SELECT count(*) FROM ubicaciones WHERE padre_id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    let asignaciones: i64 =
        sqlx::query_scalar("SELECT count(*) FROM producto_ubicacion_permitida WHERE ubicacion_id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if hijos > 0 || asignaciones > 0 {
        return Err(AppError::UbicacionConDependencias);
    }

    sqlx::query("DELETE FROM ubicaciones WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| match e.as_database_error().and_then(|db| db.code()) {
            Some(codigo) if codigo.as_ref() == "23503" => AppError::UbicacionConDependencias,
            _ => e.into(),
        })?;
    tx.commit().await?;
    Ok(())
}

pub async fn ubicaciones_de_producto(
    pool: &PgPool,
    producto_id: i32,
) -> Result<Vec<Ubicacion>, AppError> {
    let existe: Option<i32> = sqlx::query_scalar("SELECT id FROM productos WHERE id = $1")
        .bind(producto_id)
        .fetch_optional(pool)
        .await?;
    if existe.is_none() {
        return Err(producto_no_encontrado());
    }
    let ubicaciones = sqlx::query_as::<_, Ubicacion>(
        r#"SELECT u.id, u.deposito_id, u.padre_id, u.nombre, u.tipo, u.activo
             FROM producto_ubicacion_permitida p
             JOIN ubicaciones u ON u.id = p.ubicacion_id
             JOIN depositos d ON d.id = u.deposito_id
            WHERE p.producto_id = $1
            ORDER BY d.nombre ASC, u.nombre ASC, u.id ASC"#,
    )
    .bind(producto_id)
    .fetch_all(pool)
    .await?;
    Ok(ubicaciones)
}

/// FR-017: reemplaza el conjunto permitido; vacío lo limpia.
pub async fn reemplazar_ubicaciones_producto(
    pool: &PgPool,
    producto_id: i32,
    ubicacion_ids: Vec<i32>,
) -> Result<(), AppError> {
    let mut ids = Vec::with_capacity(ubicacion_ids.len());
    for id in ubicacion_ids {
        if id <= 0 {
            return Err(AppError::Validacion(
                "Cada ubicación debe ser un entero mayor a 0.".into(),
            ));
        }
        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    let mut tx = pool.begin().await?;
    let existe: Option<i32> = sqlx::query_scalar("SELECT id FROM productos WHERE id = $1")
        .bind(producto_id)
        .fetch_optional(&mut *tx)
        .await?;
    if existe.is_none() {
        return Err(producto_no_encontrado());
    }

    if !ids.is_empty() {
        let encontradas: i64 = sqlx::query_scalar(
            r#"SELECT count(*)
                 FROM ubicaciones u
                 JOIN depositos d ON d.id = u.deposito_id
                WHERE u.id = ANY($1)
                  AND u.activo
                  AND d.activo"#,
        )
        .bind(&ids)
        .fetch_one(&mut *tx)
        .await?;
        if encontradas != ids.len() as i64 {
            return Err(AppError::RecursoNoEncontrado(
                "Alguna de las ubicaciones indicadas no existe o está inactiva.".into(),
            ));
        }
    }

    sqlx::query("DELETE FROM producto_ubicacion_permitida WHERE producto_id = $1")
        .bind(producto_id)
        .execute(&mut *tx)
        .await?;
    for ubicacion_id in ids {
        sqlx::query(
            "INSERT INTO producto_ubicacion_permitida (producto_id, ubicacion_id) VALUES ($1, $2)",
        )
        .bind(producto_id)
        .bind(ubicacion_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
