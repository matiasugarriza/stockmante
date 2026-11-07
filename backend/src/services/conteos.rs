//! Conteos de inventario (009): apertura, carga, cierre transaccional, cancelación.

use crate::error::AppError;
use crate::models::conteo::{self, ConteoNuevo};
use crate::models::stock::{
    ConteoDetalle, ConteoLinea, ConteoLineaFila, ConteoFila, ListaConteos, UsuarioBreve,
};
use crate::models::usuario::Usuario;
use crate::services::{auditoria, normalizar_paginacion};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct NuevoConteo {
    pub deposito_id: i32,
    pub observacion: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FiltrosConteos {
    pub deposito_id: Option<i32>,
    pub estado: Option<String>,
    pub desde: Option<DateTime<Utc>>,
    pub hasta: Option<DateTime<Utc>>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct LineaContadaInput {
    pub linea_id: i32,
    pub cantidad_contada: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct DepositoFila {
    activo: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct StockFila {
    producto_id: i32,
    ubicacion_id: Option<i32>,
    cantidad: i32,
}

pub async fn crear(
    pool: &PgPool,
    nuevo: NuevoConteo,
    usuario: &Usuario,
) -> Result<ConteoDetalle, AppError> {
    let observacion = conteo::validar_conteo(&ConteoNuevo {
        deposito_id: nuevo.deposito_id,
        observacion: nuevo.observacion.clone(),
    })?;

    let mut tx = pool.begin().await?;

    let deposito = sqlx::query_as::<_, DepositoFila>(
        "SELECT activo FROM depositos WHERE id = $1 FOR UPDATE",
    )
    .bind(nuevo.deposito_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Validacion("El depósito indicado no existe.".into()))?;

    if !deposito.activo {
        return Err(AppError::DepositoInactivo);
    }

    // Capturar líneas del stock actual del depósito (snapshot de apertura).
    let filas = sqlx::query_as::<_, StockFila>(
        r#"SELECT s.producto_id, s.ubicacion_id, s.cantidad
             FROM stock_por_deposito s
            WHERE s.deposito_id = $1
            ORDER BY s.producto_id, s.ubicacion_id"#,
    )
    .bind(nuevo.deposito_id)
    .fetch_all(&mut *tx)
    .await?;
    if filas.is_empty() {
        return Err(AppError::Validacion(
            "El depósito no tiene stock registrado para contar.".into(),
        ));
    }

    let conteo_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO conteos (deposito_id, usuario_id, observacion)
           VALUES ($1, $2, $3) RETURNING id"#,
    )
    .bind(nuevo.deposito_id)
    .bind(usuario.id)
    .bind(observacion.as_deref())
    .fetch_one(&mut *tx)
    .await?;

    for f in &filas {
        sqlx::query(
            r#"INSERT INTO conteo_lineas
                   (conteo_id, producto_id, ubicacion_id, cantidad_sistema)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(conteo_id)
        .bind(f.producto_id)
        .bind(f.ubicacion_id)
        .bind(f.cantidad)
        .execute(&mut *tx)
        .await?;
    }

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_CONTEOS,
        conteo_id,
        auditoria::ACCION_CREAR,
        usuario.id,
    )
    .await?;

    tx.commit().await?;

    obtener(pool, conteo_id).await
}

pub async fn cargar_lineas(
    pool: &PgPool,
    conteo_id: i32,
    lineas: &[LineaContadaInput],
    usuario: &Usuario,
) -> Result<ConteoDetalle, AppError> {
    // Verificar existencia y estado.
    let estado: String = sqlx::query_scalar("SELECT estado FROM conteos WHERE id = $1")
        .bind(conteo_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ConteoNoEncontrado)?;

    if estado == "CERRADO" {
        return Err(AppError::ConteoCerrado);
    }
    if estado == "CANCELADO" {
        return Err(AppError::ConteoCancelado);
    }

    let mut tx = pool.begin().await?;

    // Bloquear conteo.
    let _locked: i32 = sqlx::query_scalar("SELECT id FROM conteos WHERE id = $1 FOR UPDATE")
        .bind(conteo_id)
        .fetch_one(&mut *tx)
        .await?;

    for lc in lineas {
        conteo::validar_cantidad_contada(lc.cantidad_contada)?;
        // Verificar que la línea pertenece al conteo.
        let existe: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM conteo_lineas WHERE id = $1 AND conteo_id = $2)",
        )
        .bind(lc.linea_id)
        .bind(conteo_id)
        .fetch_one(&mut *tx)
        .await?;
        if !existe {
            return Err(AppError::Validacion("La línea no pertenece al conteo.".into()));
        }
        sqlx::query(
            "UPDATE conteo_lineas SET cantidad_contada = $1, diferencia = $1 - cantidad_sistema WHERE id = $2",
        )
        .bind(lc.cantidad_contada)
        .bind(lc.linea_id)
        .execute(&mut *tx)
        .await?;
    }

    // Auditar carga como actualización.
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_CONTEOS,
        conteo_id,
        auditoria::ACCION_ACTUALIZAR,
        usuario.id,
    )
    .await?;

    tx.commit().await?;

    obtener(pool, conteo_id).await
}

pub async fn cerrar(
    pool: &PgPool,
    conteo_id: i32,
    usuario: &Usuario,
) -> Result<ConteoDetalle, AppError> {
    let mut tx = pool.begin().await?;

    let estado: String = sqlx::query_scalar("SELECT estado FROM conteos WHERE id = $1 FOR UPDATE")
        .bind(conteo_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::ConteoNoEncontrado)?;

    if estado == "CERRADO" {
        return Err(AppError::ConteoCerrado);
    }
    if estado == "CANCELADO" {
        return Err(AppError::ConteoCancelado);
    }

    // Cargar líneas.
    let lineas = sqlx::query_as::<_, ConteoLineaFila>(
        r#"SELECT cl.id, cl.conteo_id, cl.producto_id, p.nombre AS producto_nombre, p.sku,
                  cl.ubicacion_id, u.nombre AS ubicacion_nombre,
                  cl.cantidad_sistema, cl.cantidad_contada, cl.diferencia
             FROM conteo_lineas cl
             JOIN productos p ON p.id = cl.producto_id
             LEFT JOIN ubicaciones u ON u.id = cl.ubicacion_id
            WHERE cl.conteo_id = $1"#,
    )
    .bind(conteo_id)
    .fetch_all(&mut *tx)
    .await?;

    let deposito_id: i32 = sqlx::query_scalar("SELECT deposito_id FROM conteos WHERE id = $1")
        .bind(conteo_id)
        .fetch_one(&mut *tx)
        .await?;

    // Única compuerta: todas las líneas deben tener cantidad contada.
    let mut contadas: Vec<(&ConteoLineaFila, i32)> = Vec::with_capacity(lineas.len());
    for l in &lineas {
        let contada = l.cantidad_contada.ok_or(AppError::LineasSinContar)?;
        contadas.push((l, contada));
    }

    // El cierre ajusta contra el stock vigente (aclaración 3): la diferencia de
    // cada línea es `contada − vigente`, y el stock queda igual a lo contado.
    let mut productos_afectados: Vec<i32> = Vec::new();
    let mut ajustes: Vec<(i32, Option<i32>, i32, i32)> = Vec::new();
    for (l, contada) in &contadas {
        sqlx::query(
            r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
               VALUES ($1, $2, $3, 0)
               ON CONFLICT ON CONSTRAINT uq_stock_producto_deposito_ubicacion DO NOTHING"#,
        )
        .bind(l.producto_id)
        .bind(deposito_id)
        .bind(l.ubicacion_id)
        .execute(&mut *tx)
        .await?;

        let vigente: i32 = sqlx::query_scalar(
            r#"SELECT cantidad FROM stock_por_deposito
                WHERE producto_id = $1 AND deposito_id = $2
                  AND ubicacion_id IS NOT DISTINCT FROM $3
                FOR UPDATE"#,
        )
        .bind(l.producto_id)
        .bind(deposito_id)
        .bind(l.ubicacion_id)
        .fetch_one(&mut *tx)
        .await?;

        let diff = contada.checked_sub(vigente).ok_or_else(|| {
            AppError::Validacion("La diferencia del conteo supera el máximo admitido.".into())
        })?;

        sqlx::query(
            "UPDATE conteo_lineas SET cantidad_sistema = $1, diferencia = $2 WHERE id = $3",
        )
        .bind(vigente)
        .bind(diff)
        .bind(l.id)
        .execute(&mut *tx)
        .await?;

        if diff != 0 {
            sqlx::query(
                r#"UPDATE stock_por_deposito SET cantidad = $1, actualizado_en = now()
                   WHERE producto_id = $2 AND deposito_id = $3
                     AND ubicacion_id IS NOT DISTINCT FROM $4"#,
            )
            .bind(*contada)
            .bind(l.producto_id)
            .bind(deposito_id)
            .bind(l.ubicacion_id)
            .execute(&mut *tx)
            .await?;
            ajustes.push((l.producto_id, l.ubicacion_id, diff, *contada));
            if !productos_afectados.contains(&l.producto_id) {
                productos_afectados.push(l.producto_id);
            }
        }
    }

    // Un único AJUSTE (motivo INVENTARIO) con las líneas de diferencia ≠ 0.
    let mut movimiento_ajuste_id: Option<i32> = None;
    if !ajustes.is_empty() {
        let mov_id: i32 = sqlx::query_scalar(
            r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id)
               VALUES ('AJUSTE', $1, 'INVENTARIO', $2, $3)
               RETURNING id"#,
        )
        .bind(deposito_id)
        .bind(format!("Cierre del conteo {conteo_id}"))
        .bind(usuario.id)
        .fetch_one(&mut *tx)
        .await?;

        for (producto_id, ubicacion_id, diff, contada) in &ajustes {
            sqlx::query(
                r#"INSERT INTO movimiento_lineas
                       (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(mov_id)
            .bind(producto_id)
            .bind(ubicacion_id)
            .bind(diff)
            .bind(contada)
            .execute(&mut *tx)
            .await?;
        }

        auditoria::registrar(
            &mut tx,
            auditoria::TABLA_MOVIMIENTOS,
            mov_id,
            auditoria::ACCION_CREAR,
            usuario.id,
        )
        .await?;

        movimiento_ajuste_id = Some(mov_id);
    }

    // Recalcular cache de productos afectados.
    if !productos_afectados.is_empty() {
        sqlx::query(
            r#"UPDATE productos p
                  SET stock_actual = COALESCE(
                          (SELECT SUM(s.cantidad) FROM stock_por_deposito s WHERE s.producto_id = p.id),
                          0
                      )::int,
                      actualizado_en = now()
                WHERE p.id = ANY($1)"#,
        )
        .bind(&productos_afectados)
        .execute(&mut *tx)
        .await?;
    }

    // Cerrar conteo.
    sqlx::query(
        "UPDATE conteos SET estado = 'CERRADO', cerrado_en = now(), movimiento_ajuste_id = $1 WHERE id = $2",
    )
    .bind(movimiento_ajuste_id)
    .bind(conteo_id)
    .execute(&mut *tx)
    .await?;

    // Auditar cierre.
    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_CONTEOS,
        conteo_id,
        auditoria::ACCION_ACTUALIZAR,
        usuario.id,
    )
    .await?;

    tx.commit().await?;

    obtener(pool, conteo_id).await
}

pub async fn cancelar(
    pool: &PgPool,
    conteo_id: i32,
    usuario: &Usuario,
) -> Result<ConteoDetalle, AppError> {
    let mut tx = pool.begin().await?;

    let estado: String = sqlx::query_scalar("SELECT estado FROM conteos WHERE id = $1 FOR UPDATE")
        .bind(conteo_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::ConteoNoEncontrado)?;

    if estado == "CERRADO" {
        return Err(AppError::ConteoCerrado);
    }
    if estado == "CANCELADO" {
        return Err(AppError::ConteoCancelado);
    }

    sqlx::query(
        "UPDATE conteos SET estado = 'CANCELADO', cancelado_en = now() WHERE id = $1",
    )
    .bind(conteo_id)
    .execute(&mut *tx)
    .await?;

    auditoria::registrar(
        &mut tx,
        auditoria::TABLA_CONTEOS,
        conteo_id,
        auditoria::ACCION_DESACTIVAR,
        usuario.id,
    )
    .await?;

    tx.commit().await?;

    obtener(pool, conteo_id).await
}

pub async fn listar(
    pool: &PgPool,
    filtros: FiltrosConteos,
) -> Result<ListaConteos, AppError> {
    let (pagina, por_pagina, offset) =
        normalizar_paginacion(filtros.pagina, filtros.por_pagina)?;

    let estado = filtros.estado.clone();

    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*)
             FROM conteos c
            WHERE ($1::int IS NULL OR c.deposito_id = $1)
              AND ($2::text IS NULL OR c.estado = $2)
              AND ($3::timestamptz IS NULL OR c.creado_en >= $3)
              AND ($4::timestamptz IS NULL OR c.creado_en <= $4)"#,
    )
    .bind(filtros.deposito_id)
    .bind(estado.as_deref())
    .bind(filtros.desde)
    .bind(filtros.hasta)
    .fetch_one(pool)
    .await?;

    let filas = sqlx::query_as::<_, ConteoFila>(
        r#"SELECT c.id, c.deposito_id, d.nombre AS deposito_nombre, c.estado,
                  c.usuario_id, u.nombre AS usuario_nombre, c.creado_en,
                  c.cerrado_en, c.cancelado_en, c.movimiento_ajuste_id, c.observacion
             FROM conteos c
             JOIN depositos d ON d.id = c.deposito_id
             JOIN usuarios u ON u.id = c.usuario_id
            WHERE ($1::int IS NULL OR c.deposito_id = $1)
              AND ($2::text IS NULL OR c.estado = $2)
              AND ($3::timestamptz IS NULL OR c.creado_en >= $3)
              AND ($4::timestamptz IS NULL OR c.creado_en <= $4)
            ORDER BY c.creado_en DESC, c.id DESC
            LIMIT $5 OFFSET $6"#,
    )
    .bind(filtros.deposito_id)
    .bind(estado.as_deref())
    .bind(filtros.desde)
    .bind(filtros.hasta)
    .bind(por_pagina)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(filas.len());
    for f in filas {
        let lineas = cargar_lineas_de(pool, f.id).await?;
        items.push(ConteoDetalle {
            id: f.id,
            deposito_id: f.deposito_id,
            deposito_nombre: f.deposito_nombre,
            estado: f.estado,
            usuario: UsuarioBreve {
                id: f.usuario_id,
                nombre: f.usuario_nombre,
            },
            creado_en: f.creado_en,
            cerrado_en: f.cerrado_en,
            cancelado_en: f.cancelado_en,
            movimiento_ajuste_id: f.movimiento_ajuste_id,
            observacion: f.observacion,
            lineas,
        });
    }

    Ok(ListaConteos {
        items,
        total,
        pagina,
        por_pagina,
    })
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<ConteoDetalle, AppError> {
    let f = sqlx::query_as::<_, ConteoFila>(
        r#"SELECT c.id, c.deposito_id, d.nombre AS deposito_nombre, c.estado,
                  c.usuario_id, u.nombre AS usuario_nombre, c.creado_en,
                  c.cerrado_en, c.cancelado_en, c.movimiento_ajuste_id, c.observacion
             FROM conteos c
             JOIN depositos d ON d.id = c.deposito_id
             JOIN usuarios u ON u.id = c.usuario_id
            WHERE c.id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::ConteoNoEncontrado)?;

    let lineas = cargar_lineas_de(pool, f.id).await?;

    Ok(ConteoDetalle {
        id: f.id,
        deposito_id: f.deposito_id,
        deposito_nombre: f.deposito_nombre,
        estado: f.estado,
        usuario: UsuarioBreve {
            id: f.usuario_id,
            nombre: f.usuario_nombre,
        },
        creado_en: f.creado_en,
        cerrado_en: f.cerrado_en,
        cancelado_en: f.cancelado_en,
        movimiento_ajuste_id: f.movimiento_ajuste_id,
        observacion: f.observacion,
        lineas,
    })
}

async fn cargar_lineas_de(
    pool: &PgPool,
    conteo_id: i32,
) -> Result<Vec<ConteoLinea>, AppError> {
    let filas = sqlx::query_as::<_, ConteoLineaFila>(
        r#"SELECT cl.id, cl.conteo_id, cl.producto_id, p.nombre AS producto_nombre, p.sku,
                  cl.ubicacion_id, u.nombre AS ubicacion_nombre,
                  cl.cantidad_sistema, cl.cantidad_contada, cl.diferencia
             FROM conteo_lineas cl
             JOIN productos p ON p.id = cl.producto_id
             LEFT JOIN ubicaciones u ON u.id = cl.ubicacion_id
            WHERE cl.conteo_id = $1
            ORDER BY p.nombre, cl.ubicacion_id"#,
    )
    .bind(conteo_id)
    .fetch_all(pool)
    .await?;

    Ok(filas
        .into_iter()
        .map(|f| ConteoLinea {
            id: f.id,
            producto_id: f.producto_id,
            producto_nombre: f.producto_nombre,
            sku: f.sku,
            ubicacion_id: f.ubicacion_id,
            ubicacion_nombre: f.ubicacion_nombre,
            cantidad_sistema: f.cantidad_sistema,
            cantidad_contada: f.cantidad_contada,
            diferencia: f.diferencia,
        })
        .collect())
}