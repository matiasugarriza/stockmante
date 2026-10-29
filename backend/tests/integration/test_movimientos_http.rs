//! 007 — integración HTTP de movimientos de stock y consulta de stock.
//!
//! Cubre los escenarios 7.1–7.8 de la spec y los SC-013/014/015: atomicidad,
//! no negatividad, autoría de sesión, RBAC de AJUSTE y coherencia del cache.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, cuerpo_json, nombre_categoria_unico, sesion_gestion, sesion_operacion, sku_unico,
    solicitud,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

async fn pedir(
    app: &Router,
    cookie: &str,
    metodo: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud(metodo, uri, cookie, body))
        .await
        .expect("request de prueba");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

async fn crear_categoria(app: &Router, cookie: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/categorias",
        Some(json!({ "nombre": nombre_categoria_unico("Mov") })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de categoría: {body}");
    body["id"].as_i64().expect("id de categoría") as i32
}

async fn crear_producto(app: &Router, cookie: &str, categoria_id: i32) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/productos",
        Some(json!({
            "nombre": "Producto de movimientos",
            "sku": sku_unico("mov"),
            "categoria_id": categoria_id,
            "unidad_medida": "UNIDAD",
            "costo": 100,
            "precio_venta": 150,
            "stock_minimo": 5,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de producto: {body}");
    body["id"].as_i64().expect("id de producto") as i32
}

fn sufijo() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase()
}

async fn crear_deposito(app: &Router, cookie: &str) -> i32 {
    let marca = sufijo();
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/depositos",
        Some(json!({
            "nombre": format!("Principal {marca}"),
            "identificador": format!("PRIN-{marca}"),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de depósito: {body}");
    body["id"].as_i64().expect("id de depósito") as i32
}

/// Crea un depósito inactivo directo en la BD (no hay endpoint de alta inactiva).
async fn crear_deposito_inactivo(pool: &PgPool) -> i32 {
    let marca = sufijo();
    sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO depositos (nombre, identificador, activo)
           VALUES ($1, $2, FALSE) RETURNING id"#,
    )
    .bind(format!("Inactivo {marca}"))
    .bind(format!("INAC-{marca}"))
    .fetch_one(pool)
    .await
    .expect("insert de depósito inactivo")
}

async fn entrada(
    app: &Router,
    cookie: &str,
    deposito_id: i32,
    producto_id: i32,
    cantidad: i32,
) -> (StatusCode, Value) {
    pedir(
        app,
        cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "ENTRADA",
            "deposito_id": deposito_id,
            "motivo": "COMPRA",
            "lineas": [{
                "producto_id": producto_id,
                "ubicacion_id": null,
                "cantidad_delta": cantidad
            }]
        })),
    )
    .await
}

async fn stock_en_deposito(pool: &PgPool, producto_id: i32, deposito_id: i32) -> Option<i32> {
    sqlx::query_scalar::<_, i32>(
        r#"SELECT cantidad FROM stock_por_deposito
            WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL"#,
    )
    .bind(producto_id)
    .bind(deposito_id)
    .fetch_optional(pool)
    .await
    .expect("lectura de stock_por_deposito")
}

async fn cache_del_producto(pool: &PgPool, producto_id: i32) -> i32 {
    sqlx::query_scalar::<_, i32>(r#"SELECT stock_actual FROM productos WHERE id = $1"#)
        .bind(producto_id)
        .fetch_one(pool)
        .await
        .expect("lectura de stock_actual")
}

#[tokio::test]
async fn entrada_actualiza_stock_por_deposito_y_cache() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    // Escenario 7.1: 201 con movimiento, autor y stock resultante por línea.
    let (status, body) = entrada(&app, &cookie, deposito, producto, 10).await;
    assert_eq!(status, StatusCode::CREATED, "entrada: {body}");
    assert_eq!(body["tipo"], "ENTRADA");
    assert_eq!(body["motivo"], "COMPRA");
    assert_eq!(body["deposito"]["id"], deposito);
    assert!(!body["usuario"]["nombre"].as_str().unwrap_or_default().is_empty());
    assert_eq!(body["lineas"][0]["cantidad_delta"], 10);
    assert_eq!(body["lineas"][0]["stock_resultante"], 10);
    let mensaje = format!(
        "Depósito Principal {}: 10 unidades",
        body["deposito"]["nombre"]
            .as_str()
            .unwrap_or_default()
            .trim_start_matches("Principal ")
    );
    // El stock resultante por línea informa el saldo del depósito.
    assert!(
        body["lineas"][0]["stock_resultante"].as_i64() == Some(10),
        "stock_resultante debe ser 10: {mensaje}"
    );

    // SC-014: cache = suma por depósito.
    assert_eq!(stock_en_deposito(&pool, producto, deposito).await, Some(10));
    assert_eq!(cache_del_producto(&pool, producto).await, 10);

    // Auditoría en la misma transacción.
    let auditadas: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM auditoria WHERE tabla_ref = 'movimientos'
            AND registro_id = $1 AND accion = 'CREAR'"#,
    )
    .bind(body["id"].as_i64().expect("id de movimiento") as i32)
    .fetch_one(&pool)
    .await
    .expect("lectura de auditoría");
    assert_eq!(auditadas, 1, "el movimiento debe quedar auditado una vez");
}

#[tokio::test]
async fn salida_sin_stock_responde_409_y_no_cambia_nada() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    let (status, _) = entrada(&app, &cookie, deposito, producto, 3).await;
    assert_eq!(status, StatusCode::CREATED);

    // Escenario 7.2: mensaje literal y stock intacto.
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "SALIDA",
            "deposito_id": deposito,
            "motivo": "VENTA",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": null,
                "cantidad_delta": -5
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "salida sin stock: {body}");
    assert_eq!(body["error"]["codigo"], "stock_insuficiente");
    let nombre_deposito: String = sqlx::query_scalar(
        r#"SELECT nombre FROM depositos WHERE id = $1"#,
    )
    .bind(deposito)
    .fetch_one(&pool)
    .await
    .expect("nombre del depósito");
    assert_eq!(
        body["error"]["mensaje"],
        format!("Stock insuficiente en {nombre_deposito}: 3 disponibles, 5 solicitadas.")
    );

    // SC-013: el stock no cambió.
    assert_eq!(stock_en_deposito(&pool, producto, deposito).await, Some(3));
    assert_eq!(cache_del_producto(&pool, producto).await, 3);
}

#[tokio::test]
async fn merma_parcial_baja_el_stock_y_queda_auditada() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    let (status, _) = entrada(&app, &cookie, deposito, producto, 20).await;
    assert_eq!(status, StatusCode::CREATED);

    // Escenario 7.3: merma de 2 con motivo VENCIMIENTO → stock 18.
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "MERMA",
            "deposito_id": deposito,
            "motivo": "OTRO",
            "observacion": "Vencimiento de lote",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": null,
                "cantidad_delta": -2
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "merma: {body}");
    assert_eq!(body["lineas"][0]["stock_resultante"], 18);
    assert_eq!(stock_en_deposito(&pool, producto, deposito).await, Some(18));
    assert_eq!(cache_del_producto(&pool, producto).await, 18);

    let auditadas: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM auditoria WHERE tabla_ref = 'movimientos'
            AND registro_id = $1 AND accion = 'CREAR'"#,
    )
    .bind(body["id"].as_i64().expect("id de movimiento") as i32)
    .fetch_one(&pool)
    .await
    .expect("lectura de auditoría");
    assert_eq!(auditadas, 1);
}

#[tokio::test]
async fn ajuste_reservado_a_gestion() {
    let (app, pool, _correo) = app_de_test().await;
    let gestion = sesion_gestion(&pool).await;
    let operacion = sesion_operacion(&pool).await;
    let categoria = crear_categoria(&app, &gestion).await;
    let producto = crear_producto(&app, &gestion, categoria).await;
    let deposito = crear_deposito(&app, &gestion).await;

    // Escenario 7.4: OPERACION no puede ajustar.
    let (status, body) = pedir(
        &app,
        &operacion,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "AJUSTE",
            "deposito_id": deposito,
            "motivo": "AJUSTE",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": null,
                "cantidad_delta": 7
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "ajuste de operación: {body}");
    assert_eq!(body["error"]["codigo"], "sin_permisos");

    // GESTION sí; el ajuste puede ser positivo.
    let (status, body) = pedir(
        &app,
        &gestion,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "AJUSTE",
            "deposito_id": deposito,
            "motivo": "INVENTARIO",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": null,
                "cantidad_delta": 7
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "ajuste de gestión: {body}");
    assert_eq!(cache_del_producto(&pool, producto).await, 7);

    // Y también negativo, hasta agotar el stock.
    let (status, body) = pedir(
        &app,
        &gestion,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "AJUSTE",
            "deposito_id": deposito,
            "motivo": "INVENTARIO",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": null,
                "cantidad_delta": -10
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "ajuste que agota: {body}");
    assert_eq!(body["error"]["codigo"], "stock_insuficiente");
    assert_eq!(cache_del_producto(&pool, producto).await, 7);
}

#[tokio::test]
async fn multilínea_con_falla_no_deja_nada_escrito() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto_a = crear_producto(&app, &cookie, categoria).await;
    let producto_b = crear_producto(&app, &cookie, categoria).await;
    let producto_c = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    let (status, _) = entrada(&app, &cookie, deposito, producto_a, 5).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = entrada(&app, &cookie, deposito, producto_b, 1).await;
    assert_eq!(status, StatusCode::CREATED);

    let movimientos_antes: i64 =
        sqlx::query_scalar(r#"SELECT count(*) FROM movimientos"#)
            .fetch_one(&pool)
            .await
            .expect("conteo previo");
    let lineas_antes: i64 =
        sqlx::query_scalar(r#"SELECT count(*) FROM movimiento_lineas"#)
            .fetch_one(&pool)
            .await
            .expect("conteo previo de líneas");

    // Escenario 7.5: la tercera línea no tiene stock → 409 y nada escrito.
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "SALIDA",
            "deposito_id": deposito,
            "motivo": "VENTA",
            "lineas": [
                { "producto_id": producto_a, "ubicacion_id": null, "cantidad_delta": -2 },
                { "producto_id": producto_b, "ubicacion_id": null, "cantidad_delta": -1 },
                { "producto_id": producto_c, "ubicacion_id": null, "cantidad_delta": -100 }
            ]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "multi-línea fallida: {body}");
    assert_eq!(body["error"]["codigo"], "stock_insuficiente");

    // SC-015: sin cabecera, sin líneas, sin stock tocado.
    let movimientos_despues: i64 =
        sqlx::query_scalar(r#"SELECT count(*) FROM movimientos"#)
            .fetch_one(&pool)
            .await
            .expect("conteo posterior");
    let lineas_despues: i64 =
        sqlx::query_scalar(r#"SELECT count(*) FROM movimiento_lineas"#)
            .fetch_one(&pool)
            .await
            .expect("conteo posterior de líneas");
    assert_eq!(movimientos_despues, movimientos_antes);
    assert_eq!(lineas_despues, lineas_antes);
    assert_eq!(stock_en_deposito(&pool, producto_a, deposito).await, Some(5));
    assert_eq!(cache_del_producto(&pool, producto_a).await, 5);

    // SC-014 tras el fallo: el cache sigue coherente.
    let suma: i32 = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(cantidad), 0)::int FROM stock_por_deposito WHERE producto_id = $1"#,
    )
    .bind(producto_a)
    .fetch_one(&pool)
    .await
    .expect("suma por depósito");
    assert_eq!(cache_del_producto(&pool, producto_a).await, suma);
}

#[tokio::test]
async fn ubicacion_invalida_responde_422() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;
    let otro_deposito = crear_deposito(&app, &cookie).await;

    // Ubicación de otro depósito → 422.
    let (status, body_ub) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/ubicaciones",
        Some(json!({
            "deposito_id": otro_deposito,
            "nombre": format!("Pasillo {}", sufijo()),
            "tipo": "PASILLO",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de ubicación: {body_ub}");
    let ubicacion_ajena = body_ub["id"].as_i64().expect("id de ubicación") as i32;

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "ENTRADA",
            "deposito_id": deposito,
            "motivo": "COMPRA",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": ubicacion_ajena,
                "cantidad_delta": 1
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "ubicación ajena: {body}");
    assert_eq!(body["error"]["codigo"], "validacion");

    // Ubicación del depósito correcto pero no asignada al producto → 422.
    let (status, body_propia) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/ubicaciones",
        Some(json!({
            "deposito_id": deposito,
            "nombre": format!("Estante {}", sufijo()),
            "tipo": "ESTANTERIA",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de ubicación propia: {body_propia}");
    let ubicacion_propia = body_propia["id"].as_i64().expect("id de ubicación") as i32;

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "ENTRADA",
            "deposito_id": deposito,
            "motivo": "COMPRA",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": ubicacion_propia,
                "cantidad_delta": 1
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "ubicación no permitida: {body}");
    assert_eq!(body["error"]["codigo"], "validacion");

    // Con la asignación hecha, la misma ubicación sí procede (005).
    let (status, body_asig) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/productos/{producto}/ubicaciones"),
        Some(json!({ "ubicacion_ids": [ubicacion_propia] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "asignación: {body_asig}");

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "ENTRADA",
            "deposito_id": deposito,
            "motivo": "COMPRA",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": ubicacion_propia,
                "cantidad_delta": 4
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "ubicación permitida: {body}");
    assert_eq!(body["lineas"][0]["stock_resultante"], 4);
    // El stock con ubicación no se imputa al depósito sin ubicación.
    assert_eq!(stock_en_deposito(&pool, producto, deposito).await, None);
}

#[tokio::test]
async fn entidades_inactivas_rechazan_el_movimiento() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    // Escenario 7.7a: producto inactivo → 409 producto_inactivo.
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/productos/{producto}/desactivar"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "desactivar producto: {body}");

    let (status, body) = entrada(&app, &cookie, deposito, producto, 1).await;
    assert_eq!(status, StatusCode::CONFLICT, "producto inactivo: {body}");
    assert_eq!(body["error"]["codigo"], "producto_inactivo");
    assert_eq!(
        body["error"]["mensaje"],
        "El producto está inactivo y no admite movimientos."
    );

    // Reactivar para el caso del depósito.
    let (status, _) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/productos/{producto}/reactivar"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Escenario 7.7b: depósito inactivo → 422.
    let deposito_inactivo = crear_deposito_inactivo(&pool).await;
    let (status, body) = entrada(&app, &cookie, deposito_inactivo, producto, 1).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "depósito inactivo: {body}");
    assert_eq!(body["error"]["codigo"], "validacion");
    assert_eq!(cache_del_producto(&pool, producto).await, 0);
}

#[tokio::test]
async fn consulta_de_historial_y_stock() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    let (status, entrada_body) = entrada(&app, &cookie, deposito, producto, 10).await;
    assert_eq!(status, StatusCode::CREATED);
    let id_entrada = entrada_body["id"].as_i64().expect("id de entrada") as i32;

    let (status, _) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "SALIDA",
            "deposito_id": deposito,
            "motivo": "VENTA",
            "lineas": [{
                "producto_id": producto,
                "ubicacion_id": null,
                "cantidad_delta": -4
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Escenario 7.8: historial con filtros.
    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/movimientos?tipo=SALIDA&deposito_id={deposito}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "historial filtrado: {body}");
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["tipo"], "SALIDA");
    assert!(body["items"][0]["lineas"].as_array().is_some_and(|l| !l.is_empty()));

    // Detalle por id con sus líneas.
    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/movimientos/{id_entrada}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "detalle: {body}");
    assert_eq!(body["id"], id_entrada);
    assert_eq!(body["lineas"][0]["cantidad_delta"], 10);
    assert_eq!(body["lineas"][0]["stock_resultante"], 10);
    assert!(!body["usuario"]["nombre"].as_str().unwrap_or_default().is_empty());

    // 404 para un id inexistente.
    let (status, body) = pedir(&app, &cookie, "GET", "/api/movimientos/99999999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "404: {body}");

    // Filtro por producto.
    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/movimientos?producto_id={producto}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2, "el filtro por producto debe traer ambos movimientos");

    // Listado de stock: cache y bajo_minimo coherentes.
    // `bajo_minimo=true` es global en la BD de test: alcanza con que el
    // producto de este test (6 unidades, mínimo 5) no aparezca.
    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/stock?deposito_id={deposito}&bajo_minimo=true"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "stock bajo mínimo: {body}");
    let aparece = body["items"]
        .as_array()
        .map(|items| items.iter().any(|i| i["producto_id"] == producto))
        .unwrap_or(false);
    assert!(!aparece, "con 6 unidades y mínimo 5 no debe estar bajo mínimo: {body}");

    // El listado es paginado y la BD de test es compartida: se filtra por el
    // SKU único del producto para no depender del orden de la página 1.
    let sku: String = sqlx::query_scalar(r#"SELECT sku FROM productos WHERE id = $1"#)
        .bind(producto)
        .fetch_one(&pool)
        .await
        .expect("sku del producto");
    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/stock?deposito_id={deposito}&q={sku}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item = &body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|i| i["producto_id"] == producto)
        .expect("el producto debe estar en el listado");
    assert_eq!(item["stock_actual"], 6);
    assert_eq!(item["stock_deposito"], 6);
    assert_eq!(item["bajo_minimo"], false);

    // Desglose por producto: cache = suma del desglose (SC-014).
    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/stock/{producto}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "desglose: {body}");
    assert_eq!(body["stock_actual"], 6);
    let suma: i64 = body["desglose"]
        .as_array()
        .expect("desglose")
        .iter()
        .map(|fila| fila["cantidad"].as_i64().expect("cantidad"))
        .sum();
    assert_eq!(i64::from(cache_del_producto(&pool, producto).await), suma);
}

#[tokio::test]
async fn sin_sesion_los_nuevos_endpoints_respoden_401() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;
    let deposito = crear_deposito(&app, &cookie).await;

    for (metodo, uri, body) in [
        (
            "POST",
            "/api/movimientos".to_string(),
            Some(json!({
                "tipo": "ENTRADA",
                "deposito_id": deposito,
                "motivo": "COMPRA",
                "lineas": [{ "producto_id": producto, "ubicacion_id": null, "cantidad_delta": 1 }]
            })),
        ),
        ("GET", "/api/movimientos".to_string(), None),
        ("GET", "/api/movimientos/1".to_string(), None),
        ("GET", "/api/stock".to_string(), None),
        ("GET", format!("/api/stock/{producto}"), None),
    ] {
        let (status, body_resp) = pedir(&app, "", metodo, &uri, body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{metodo} {uri}: {body_resp}");
        assert_eq!(body_resp["error"]["codigo"], "no_autenticado");
    }
}
