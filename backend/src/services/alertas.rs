//! Alertas de stock mínimo (011, FR-041 … FR-043).
//!
//! - Detección: productos activos con `stock_minimo > 0 AND stock_actual < stock_minimo`.
//! - Ciclo: registra una notificación por (producto, gestor, día local) con
//!   `ON CONFLICT DO NOTHING`, reclama las no enviadas con `FOR UPDATE SKIP LOCKED`
//!   (un `POST` y el scheduler simultáneos no envían dos veces) y manda **un correo
//!   por usuario** con todos sus productos pendientes. Un fallo de transporte deja
//!   las filas `fallida` y el ciclo siguiente las reintenta; nunca rompe la API.
//! - No modifica stock ni audita (no es una escritura de gestión).

use crate::auth::reloj::Reloj;
use crate::error::AppError;
use crate::mail::envio::EnvioCorreo;
use crate::models::alertas::{AlertaItem, ListaAlertas, RespuestaCiclo};
use chrono::{FixedOffset, NaiveDate};
use sqlx::PgPool;
use std::collections::BTreeMap;

/// `por_pagina` por defecto y máximo del listado (contrato de 011).
pub const POR_PAGINA_DEFAULT: i64 = 50;
pub const POR_PAGINA_MAX: i64 = 100;

/// Asunto del digest.
pub const ASUNTO: &str = "Stockmante — productos bajo el stock mínimo";

/// Día local de Argentina (UTC−03:00 fijo, sin horario de verano).
pub fn fecha_local(reloj: &dyn Reloj) -> NaiveDate {
    let ahora = reloj.ahora();
    // `west_opt(10800)` siempre es válido; el fallback a UTC solo evita un `unwrap`.
    FixedOffset::west_opt(3 * 3600)
        .map(|offset| ahora.with_timezone(&offset).date_naive())
        .unwrap_or_else(|| ahora.date_naive())
}

/// Normaliza la paginación del listado: `(pagina, por_pagina, offset)`.
pub fn paginacion_alertas(
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
    let offset = (pagina - 1)
        .checked_mul(por_pagina)
        .ok_or_else(|| AppError::Validacion("La página es demasiado grande".into()))?;
    Ok((pagina, por_pagina, offset))
}

/// Una línea del correo.
#[derive(Debug, Clone)]
pub struct LineaCorreo {
    pub nombre: String,
    pub sku: String,
    pub stock_actual: i64,
    pub stock_minimo: i32,
    pub faltante: i64,
}

/// Cuerpo en texto plano: saludo y una línea por producto con su faltante.
pub fn cuerpo_correo(nombre: &str, lineas: &[LineaCorreo]) -> String {
    let mut cuerpo = format!(
        "Hola {nombre}:\n\nEstos productos están por debajo del stock mínimo:\n"
    );
    for l in lineas {
        cuerpo.push_str(&format!(
            "- {} (SKU {}): stock {}, mínimo {}, faltan {}\n",
            l.nombre, l.sku, l.stock_actual, l.stock_minimo, l.faltante
        ));
    }
    cuerpo.push_str("\nRegistrá una compra o una entrada para reponerlos.\n");
    cuerpo
}

/// Listado paginado de productos bajo mínimo, ordenado por nombre.
pub async fn detectar(
    pool: &PgPool,
    pagina: Option<i64>,
    por_pagina: Option<i64>,
) -> Result<ListaAlertas, AppError> {
    let (pagina, por_pagina, offset) = paginacion_alertas(pagina, por_pagina)?;
    let total: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM productos
            WHERE activo AND stock_minimo > 0 AND stock_actual < stock_minimo"#,
    )
    .fetch_one(pool)
    .await?;
    let items = sqlx::query_as::<_, AlertaItem>(
        r#"SELECT id AS producto_id, nombre, sku,
                  stock_actual::bigint AS stock_actual, stock_minimo,
                  (stock_minimo - stock_actual)::bigint AS faltante
             FROM productos
            WHERE activo AND stock_minimo > 0 AND stock_actual < stock_minimo
            ORDER BY nombre, id
            LIMIT $1 OFFSET $2"#,
    )
    .bind(por_pagina)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(ListaAlertas {
        items,
        total,
        pagina,
        por_pagina,
    })
}

/// Acota el ciclo a ciertos productos o usuarios. Producción usa `AlcanceCiclo::default()`
/// (todo); los tests lo acotan porque la BD de test es compartida.
#[derive(Debug, Clone, Default)]
pub struct AlcanceCiclo {
    pub productos: Option<Vec<i32>>,
    pub usuarios: Option<Vec<i32>>,
}

#[derive(Debug, sqlx::FromRow)]
struct Reclamada {
    id: i32,
    usuario_id: i32,
    email: String,
    usuario_nombre: String,
    producto_nombre: String,
    sku: String,
    stock_actual: i64,
    stock_minimo: i32,
}

/// Un ciclo completo: detectar, registrar, reclamar, enviar y marcar.
pub async fn ciclo(
    pool: &PgPool,
    correo: &dyn EnvioCorreo,
    reloj: &dyn Reloj,
    alcance: &AlcanceCiclo,
) -> Result<RespuestaCiclo, AppError> {
    let fecha = fecha_local(reloj);
    let productos_ids = alcance.productos.as_deref();
    let usuarios_ids = alcance.usuarios.as_deref();

    let productos: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM productos p
            WHERE p.activo AND p.stock_minimo > 0 AND p.stock_actual < p.stock_minimo
              AND ($1::int[] IS NULL OR p.id = ANY($1))"#,
    )
    .bind(productos_ids)
    .fetch_one(pool)
    .await?;
    let destinatarios: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM usuarios u
            WHERE u.activo AND u.familia_rol = 'GESTION'
              AND ($1::int[] IS NULL OR u.id = ANY($1))"#,
    )
    .bind(usuarios_ids)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"INSERT INTO notificaciones_stock (producto_id, usuario_id, fecha)
           SELECT p.id, u.id, $1
             FROM productos p
            CROSS JOIN usuarios u
            WHERE p.activo AND p.stock_minimo > 0 AND p.stock_actual < p.stock_minimo
              AND u.activo AND u.familia_rol = 'GESTION'
              AND ($2::int[] IS NULL OR p.id = ANY($2))
              AND ($3::int[] IS NULL OR u.id = ANY($3))
           ON CONFLICT ON CONSTRAINT uq_notificacion_producto_usuario_fecha DO NOTHING"#,
    )
    .bind(fecha)
    .bind(productos_ids)
    .bind(usuarios_ids)
    .execute(pool)
    .await?;

    let mut tx = pool.begin().await?;
    let reclamadas = sqlx::query_as::<_, Reclamada>(
        r#"SELECT n.id, n.usuario_id, u.email, u.nombre AS usuario_nombre,
                  p.nombre AS producto_nombre, p.sku,
                  p.stock_actual::bigint AS stock_actual, p.stock_minimo
             FROM notificaciones_stock n
             JOIN usuarios u ON u.id = n.usuario_id
             JOIN productos p ON p.id = n.producto_id
            WHERE n.fecha = $1 AND n.estado <> 'enviada'
              AND u.activo AND u.familia_rol = 'GESTION'
              AND p.activo AND p.stock_minimo > 0 AND p.stock_actual < p.stock_minimo
              AND ($2::int[] IS NULL OR n.producto_id = ANY($2))
              AND ($3::int[] IS NULL OR n.usuario_id = ANY($3))
            ORDER BY n.usuario_id, p.nombre, p.id
              FOR UPDATE OF n SKIP LOCKED"#,
    )
    .bind(fecha)
    .bind(productos_ids)
    .bind(usuarios_ids)
    .fetch_all(&mut *tx)
    .await?;

    let mut por_usuario: BTreeMap<i32, Vec<Reclamada>> = BTreeMap::new();
    for fila in reclamadas {
        por_usuario.entry(fila.usuario_id).or_default().push(fila);
    }

    let (mut enviados, mut fallidos) = (0_i64, 0_i64);
    for filas in por_usuario.values() {
        let Some(primera) = filas.first() else {
            continue;
        };
        let lineas: Vec<LineaCorreo> = filas
            .iter()
            .map(|f| LineaCorreo {
                nombre: f.producto_nombre.clone(),
                sku: f.sku.clone(),
                stock_actual: f.stock_actual,
                stock_minimo: f.stock_minimo,
                faltante: i64::from(f.stock_minimo) - f.stock_actual,
            })
            .collect();
        let cuerpo = cuerpo_correo(&primera.usuario_nombre, &lineas);
        let ids: Vec<i32> = filas.iter().map(|f| f.id).collect();
        let cantidad = ids.len() as i64;
        match correo
            .enviar_alerta_stock(&primera.email, &primera.usuario_nombre, &cuerpo)
            .await
        {
            Ok(()) => {
                sqlx::query(
                    r#"UPDATE notificaciones_stock
                          SET estado = 'enviada', enviada_en = now(),
                              intentos = intentos + 1, error = NULL
                        WHERE id = ANY($1)"#,
                )
                .bind(&ids)
                .execute(&mut *tx)
                .await?;
                enviados += cantidad;
            }
            Err(motivo) => {
                tracing::warn!(usuario_id = primera.usuario_id, error = %motivo, "alerta de stock no enviada");
                sqlx::query(
                    r#"UPDATE notificaciones_stock
                          SET estado = 'fallida', intentos = intentos + 1, error = $2
                        WHERE id = ANY($1)"#,
                )
                .bind(&ids)
                .bind(&motivo)
                .execute(&mut *tx)
                .await?;
                fallidos += cantidad;
            }
        }
    }
    tx.commit().await?;

    Ok(RespuestaCiclo {
        fecha,
        productos,
        destinatarios,
        enviados,
        fallidos,
    })
}
