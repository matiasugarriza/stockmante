//! Importación masiva de productos (016).

use crate::error::AppError;
use crate::models::usuario::Usuario;
use crate::services::auditoria;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashSet;
use std::io::Cursor;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FilaErrorImportacion {
    pub fila: usize,
    pub columna: String,
    pub mensaje: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResumenImportacion {
    pub filas_leidas: usize,
    pub validas: usize,
    pub con_error: usize,
    pub errores: Vec<FilaErrorImportacion>,
    pub creados: usize,
    pub aplicado: bool,
}

fn err(fila: usize, columna: &str, mensaje: String) -> FilaErrorImportacion {
    FilaErrorImportacion { fila, columna: columna.to_string(), mensaje }
}

pub async fn importar(
    pool: &PgPool,
    bytes: &[u8],
    modo: &str,
    usuario: &Usuario,
) -> Result<ResumenImportacion, AppError> {
    if bytes.len() > 5 * 1024 * 1024 {
        return Err(AppError::Validacion("El archivo supera el límite de 5.000 filas o 5 MB.".into()));
    }
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(Cursor::new(bytes));

    // headers
    let headers = reader.headers().map_err(|_| AppError::Validacion("CSV inválido.".into()))?.clone();
    let idx = |name: &str| headers.iter().position(|h| h.trim().eq_ignore_ascii_case(name));

    let i_nombre = idx("nombre").ok_or_else(|| AppError::Validacion("Falta columna nombre.".into()))?;
    let i_sku = idx("sku").ok_or_else(|| AppError::Validacion("Falta columna sku.".into()))?;
    let i_categoria = idx("categoria").ok_or_else(|| AppError::Validacion("Falta columna categoria.".into()))?;
    let i_unidad = idx("unidad_medida").ok_or_else(|| AppError::Validacion("Falta columna unidad_medida.".into()))?;
    let i_costo = idx("costo").ok_or_else(|| AppError::Validacion("Falta columna costo.".into()))?;
    let i_precio = idx("precio_venta").ok_or_else(|| AppError::Validacion("Falta columna precio_venta.".into()))?;
    let i_stock_min = idx("stock_minimo").ok_or_else(|| AppError::Validacion("Falta columna stock_minimo.".into()))?;
    let i_cb = idx("codigo_barras");
    let i_stock_ini = idx("stock_inicial").ok_or_else(|| AppError::Validacion("Falta columna stock_inicial.".into()))?;
    let i_deposito = idx("deposito");

    // prefetch categorias and depositos
    let categorias: Vec<(i32, String)> = sqlx::query_as("SELECT id, nombre FROM categorias").fetch_all(pool).await?;
    let cat_map: std::collections::HashMap<String, i32> = categorias.into_iter().map(|(id, n)| (n.to_lowercase(), id)).collect();
    let depositos: Vec<(i32, String)> = sqlx::query_as("SELECT id, nombre FROM depositos").fetch_all(pool).await?;
    let dep_map: std::collections::HashMap<String, i32> = depositos.into_iter().map(|(id, n)| (n.to_lowercase(), id)).collect();
    let principal_id: Option<i32> = dep_map.get("principal").copied();

    // existing skus
    let existing_skus: HashSet<String> = sqlx::query_scalar::<_, String>("SELECT sku FROM productos")
        .fetch_all(pool).await?.into_iter().map(|s| s.to_lowercase()).collect();

    // parse rows
    let mut rows: Vec<Vec<String>> = Vec::new();
    for rec in reader.records() {
        let r = rec.map_err(|_| AppError::Validacion("CSV inválido.".into()))?;
        rows.push(r.iter().map(|s| s.to_string()).collect());
        if rows.len() > 5000 {
            return Err(AppError::Validacion("El archivo supera el límite de 5.000 filas o 5 MB.".into()));
        }
    }

    let mut errores: Vec<FilaErrorImportacion> = Vec::new();
    let mut parsed: Vec<(usize, String, String, i32, String, Decimal, Decimal, i32, Option<String>, i32, i32)> = Vec::new();
    let mut seen_skus: HashSet<String> = HashSet::new();

    for (ri, row) in rows.iter().enumerate() {
        let fila = ri + 2; // 1 header + 1-based
        let get = |i: usize| row.get(i).map(|s| s.trim().to_string()).unwrap_or_default();

        let nombre = get(i_nombre);
        let sku = get(i_sku);
        let categoria = get(i_categoria);
        let unidad = get(i_unidad);
        let costo_s = get(i_costo);
        let precio_s = get(i_precio);
        let stock_min_s = get(i_stock_min);
        let cb = i_cb.map(|i| get(i)).filter(|s| !s.is_empty());
        let stock_ini_s = get(i_stock_ini);
        let deposito_s = i_deposito.map(|i| get(i)).filter(|s| !s.is_empty());

        // validate nombre
        if nombre.is_empty() { errores.push(err(fila, "nombre", "El nombre es obligatorio.".into())); }
        // sku
        if sku.is_empty() { errores.push(err(fila, "sku", "El SKU es obligatorio.".into())); }
        // duplicate in file
        if !sku.is_empty() {
            let key = sku.to_lowercase();
            if seen_skus.contains(&key) {
                // find first row
                errores.push(err(fila, "sku", format!("El SKU '{}' está repetido en el archivo.", sku)));
            } else {
                seen_skus.insert(key.clone());
            }
            if existing_skus.contains(&key) {
                errores.push(err(fila, "sku", format!("El SKU '{}' ya existe.", sku)));
            }
        }
        // categoria
        let cat_id = if categoria.is_empty() {
            errores.push(err(fila, "categoria", "La categoría es obligatoria.".into()));
            None
        } else {
            match cat_map.get(&categoria.to_lowercase()) {
                Some(id) => Some(*id),
                None => { errores.push(err(fila, "categoria", format!("La categoría '{}' no existe.", categoria))); None }
            }
        };
        // unidad
        if unidad.is_empty() { errores.push(err(fila, "unidad_medida", "La unidad de medida es obligatoria.".into())); }
        // costo
        let costo: Option<Decimal> = costo_s.parse::<Decimal>().ok();
        if costo_s.is_empty() { errores.push(err(fila, "costo", "El costo es obligatorio.".into())); }
        else if costo.is_none() { errores.push(err(fila, "costo", "Costo inválido.".into())); }
        else if costo.unwrap() < Decimal::ZERO { errores.push(err(fila, "costo", "El costo no puede ser negativo.".into())); }
        // precio
        let precio: Option<Decimal> = precio_s.parse::<Decimal>().ok();
        if precio_s.is_empty() { errores.push(err(fila, "precio_venta", "El precio de venta es obligatorio.".into())); }
        else if precio.is_none() { errores.push(err(fila, "precio_venta", "Precio inválido.".into())); }
        // stock_min
        let stock_min: Option<i32> = stock_min_s.parse::<i32>().ok();
        if stock_min.is_none() && !stock_min_s.is_empty() { errores.push(err(fila, "stock_minimo", "stock_minimo inválido.".into())); }
        let stock_min = stock_min.unwrap_or(0);
        if stock_min < 0 { errores.push(err(fila, "stock_minimo", "stock_minimo no puede ser negativo.".into())); }
        // stock_inicial
        let stock_ini: Option<i32> = stock_ini_s.parse::<i32>().ok();
        if stock_ini.is_none() && !stock_ini_s.is_empty() { errores.push(err(fila, "stock_inicial", "stock_inicial inválido.".into())); }
        let stock_ini = stock_ini.unwrap_or(0);
        if stock_ini < 0 { errores.push(err(fila, "stock_inicial", "stock_inicial no puede ser negativo.".into())); }
        // deposito
        let dep_id = if let Some(dn) = &deposito_s {
            match dep_map.get(&dn.to_lowercase()) {
                Some(id) => Some(*id),
                None => { errores.push(err(fila, "deposito", format!("El depósito '{}' no existe.", dn))); None }
            }
        } else {
            principal_id
        };
        if deposito_s.is_none() && principal_id.is_none() {
            errores.push(err(fila, "deposito", "No se encontró el depósito Principal.".into()));
        }

        // if no errors for this row, push parsed
        // We can't easily know per-row errors here; collect all errors first, then decide.
        // We'll push parsed if required fields valid
        if !nombre.is_empty() && !sku.is_empty() && cat_id.is_some() && !unidad.is_empty()
            && costo.is_some() && precio.is_some() && dep_id.is_some()
            && stock_min >= 0 && stock_ini >= 0 {
            parsed.push((fila, nombre, sku, cat_id.unwrap(), unidad, costo.unwrap(), precio.unwrap(), stock_min, cb, stock_ini, dep_id.unwrap()));
        }
    }

    let filas_leidas = rows.len();
    let con_error = if errores.is_empty() { 0 } else {
        // count distinct rows with errors
        let mut set = HashSet::new();
        for e in &errores { set.insert(e.fila); }
        set.len()
    };
    let validas = filas_leidas.saturating_sub(con_error);

    if modo == "validar" {
        return Ok(ResumenImportacion {
            filas_leidas, validas, con_error, errores, creados: 0, aplicado: false,
        });
    }

    // aplicar: if errors, return 422 via AppError? We'll return resumen with aplicado false and let handler return 422.
    if !errores.is_empty() {
        return Ok(ResumenImportacion {
            filas_leidas, validas, con_error, errores, creados: 0, aplicado: false,
        });
    }

    // apply atomically
    let mut tx = pool.begin().await?;
    let mut creados = 0usize;
    for (_fila, nombre, sku, cat_id, unidad, costo, precio, stock_min, cb, stock_ini, dep_id) in &parsed {
        let pid: i32 = sqlx::query_scalar(
            r#"INSERT INTO productos (nombre, sku, categoria_id, unidad_medida, costo, precio_venta, stock_minimo, codigo_barras, stock_actual)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,0) RETURNING id"#,
        )
        .bind(nombre).bind(sku).bind(cat_id).bind(unidad).bind(costo).bind(precio).bind(stock_min).bind(cb.as_deref())
        .fetch_one(&mut *tx).await?;

        if *stock_ini > 0 {
            // ensure stock row
            sqlx::query(
                r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
                   VALUES ($1,$2,NULL,0)
                   ON CONFLICT ON CONSTRAINT uq_stock_producto_deposito_ubicacion DO NOTHING"#,
            )
            .bind(pid).bind(dep_id).execute(&mut *tx).await?;

            let actual: i32 = sqlx::query_scalar(
                "SELECT cantidad FROM stock_por_deposito WHERE producto_id=$1 AND deposito_id=$2 AND ubicacion_id IS NULL FOR UPDATE",
            )
            .bind(pid).bind(dep_id).fetch_one(&mut *tx).await?;
            let resultante = actual + *stock_ini;
            sqlx::query("UPDATE stock_por_deposito SET cantidad=$1, actualizado_en=now() WHERE producto_id=$2 AND deposito_id=$3 AND ubicacion_id IS NULL")
                .bind(resultante).bind(pid).bind(dep_id).execute(&mut *tx).await?;

            let mov_id: i32 = sqlx::query_scalar(
                r#"INSERT INTO movimientos (tipo, deposito_id, motivo, observacion, usuario_id)
                   VALUES ('ENTRADA', $1, 'IMPORTACION', $2, $3) RETURNING id"#,
            )
            .bind(dep_id).bind(format!("Importación producto {}", sku)).bind(usuario.id)
            .fetch_one(&mut *tx).await?;

            sqlx::query(
                "INSERT INTO movimiento_lineas (movimiento_id, producto_id, ubicacion_id, cantidad_delta, stock_resultante) VALUES ($1,$2,NULL,$3,$4)",
            )
            .bind(mov_id).bind(pid).bind(*stock_ini).bind(resultante)
            .execute(&mut *tx).await?;

            sqlx::query("UPDATE productos SET stock_actual=$1, actualizado_en=now() WHERE id=$2")
                .bind(resultante).bind(pid).execute(&mut *tx).await?;

            auditoria::registrar(&mut tx, "movimientos", mov_id, auditoria::ACCION_CREAR, usuario.id).await?;
        }

        auditoria::registrar(&mut tx, "productos", pid, auditoria::ACCION_CREAR, usuario.id).await?;
        creados += 1;
    }

    tx.commit().await?;

    Ok(ResumenImportacion {
        filas_leidas, validas, con_error: 0, errores: vec![], creados, aplicado: true,
    })
}
