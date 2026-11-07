//! 012 — integración HTTP del flujo de compras.
//!
//! Cubre alta de proveedor, compra en borrador, confirmación (entrada de stock),
//! confirmación duplicada (409) y anulación, además del RBAC de proveedores y
//! compras para la familia OPERACION.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, cuerpo_json, nombre_categoria_unico, sesion_gestion, sesion_operacion,
    sku_unico, solicitud,
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
        Some(json!({ "nombre": nombre_categoria_unico("Compras") })),
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
            "nombre": "Producto de compras",
            "sku": sku_unico("compra"),
            "categoria_id": categoria_id,
            "unidad_medida": "UNIDAD",
            "costo": 10,
            "precio_venta": 20,
            "stock_minimo": 0,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de producto: {body}");
    body["id"].as_i64().expect("id de producto") as i32
}

async fn crear_deposito(app: &Router, cookie: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/depositos",
        Some(json!({
            "nombre": format!("Depósito {}", sku_unico("dep")),
            "identificador": sku_unico("dep"),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de depósito: {body}");
    body["id"].as_i64().expect("id de depósito") as i32
}

async fn crear_proveedor(app: &Router, cookie: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/proveedores",
        Some(json!({ "nombre": format!("Proveedor {}", sku_unico("prov")) })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de proveedor: {body}");
    body["id"].as_i64().expect("id de proveedor") as i32
}

async fn stock_en_deposito(pool: &PgPool, producto_id: i32, deposito_id: i32) -> i32 {
    sqlx::query_scalar::<_, Option<i32>>(
        r#"SELECT cantidad FROM stock_por_deposito
            WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL"#,
    )
    .bind(producto_id)
    .bind(deposito_id)
    .fetch_optional(pool)
    .await
    .expect("lectura de stock_por_deposito")
    .flatten()
    .unwrap_or(0)
}

#[tokio::test]
async fn flujo_de_compra_borrador_confirmacion_y_anulacion() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;

    let categoria_id = crear_categoria(&app, &cookie).await;
    let producto_id = crear_producto(&app, &cookie, categoria_id).await;
    let deposito_id = crear_deposito(&app, &cookie).await;
    let proveedor_id = crear_proveedor(&app, &cookie).await;

    assert_eq!(
        stock_en_deposito(&pool, producto_id, deposito_id).await,
        0
    );

    let (status, detalle) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/compras",
        Some(json!({
            "proveedor_id": proveedor_id,
            "deposito_id": deposito_id,
            "numero": "OC-TEST-001",
            "observacion": null,
            "lineas": [{
                "producto_id": producto_id,
                "cantidad": 3,
                "costo_unitario": 25.5
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de compra: {detalle}");
    assert_eq!(detalle["compra"]["estado"], "BORRADOR");
    assert_eq!(
        detalle["compra"]["total_costo"].as_f64(),
        Some(76.5),
        "total de la compra: {detalle}"
    );
    assert_eq!(
        stock_en_deposito(&pool, producto_id, deposito_id).await,
        0,
        "un borrador no debe mover stock"
    );
    let compra_id = detalle["compra"]["id"].as_i64().expect("id de compra") as i32;

    let (status, confirmada) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/compras/{compra_id}/confirmar"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "confirmar: {confirmada}");
    assert_eq!(confirmada["compra"]["estado"], "CONFIRMADA");
    assert_eq!(
        stock_en_deposito(&pool, producto_id, deposito_id).await,
        3,
        "confirmar debe ingresar la cantidad comprada"
    );

    let (status, duplicada) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/compras/{compra_id}/confirmar"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "confirmación duplicada: {duplicada}");
    assert_eq!(duplicada["error"]["codigo"], "compra_ya_confirmada");
    assert_eq!(
        stock_en_deposito(&pool, producto_id, deposito_id).await,
        3,
        "la confirmación duplicada no debe duplicar stock"
    );

    let (status, anulada) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/compras/{compra_id}/anular"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "anular: {anulada}");
    assert_eq!(anulada["compra"]["estado"], "ANULADA");
    assert_eq!(
        stock_en_deposito(&pool, producto_id, deposito_id).await,
        0,
        "anular debe revertir el stock"
    );
}

#[tokio::test]
async fn operacion_no_accede_a_proveedores_ni_compras() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;

    for (metodo, uri, body) in [
        ("GET", "/api/proveedores", None),
        (
            "POST",
            "/api/proveedores",
            Some(json!({ "nombre": "No permitido" })),
        ),
        ("GET", "/api/compras", None),
        ("POST", "/api/compras", Some(json!({}))),
    ] {
        let (status, respuesta) = pedir(&app, &cookie, metodo, uri, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{metodo} {uri} debía ser 403: {respuesta}"
        );
        assert_eq!(respuesta["error"]["codigo"], "sin_permisos");
    }
}

fn cuit_unico() -> String {
    let n = uuid::Uuid::new_v4().as_u128() % 100_000_000_000u128;
    format!("{n:011}")
}

#[tokio::test]
async fn escenario_12_6_borrador_eliminable_y_estados_no_editables() {
    let (app, pool, _) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria_id = crear_categoria(&app, &cookie).await;
    let producto_id = crear_producto(&app, &cookie, categoria_id).await;
    let deposito_id = crear_deposito(&app, &cookie).await;
    let proveedor_id = crear_proveedor(&app, &cookie).await;
    let cuerpo = json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [{ "producto_id": producto_id, "cantidad": 2, "costo_unitario": 3.0 }]
    });
    let (_, borrador) = pedir(&app, &cookie, "POST", "/api/compras", Some(cuerpo.clone())).await;
    let borrador_id = borrador["compra"]["id"].as_i64().expect("id borrador") as i32;
    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/compras/{borrador_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(stock_en_deposito(&pool, producto_id, deposito_id).await, 0);

    let (_, detalle) = pedir(&app, &cookie, "POST", "/api/compras", Some(cuerpo.clone())).await;
    let compra_id = detalle["compra"]["id"].as_i64().expect("id compra") as i32;
    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/compras/{compra_id}/confirmar"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, error) = pedir(&app, &cookie, "PUT", &format!("/api/compras/{compra_id}"), Some(cuerpo.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["codigo"], "compra_no_editable");
    let (status, error) = pedir(&app, &cookie, "DELETE", &format!("/api/compras/{compra_id}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["codigo"], "compra_no_editable");
    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/compras/{compra_id}/anular"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, error) = pedir(&app, &cookie, "PUT", &format!("/api/compras/{compra_id}"), Some(cuerpo)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["codigo"], "compra_no_editable");
    assert_eq!(stock_en_deposito(&pool, producto_id, deposito_id).await, 0);
}

#[tokio::test]
async fn escenario_12_5_anular_sin_stock_es_409_y_no_cambia_la_compra() {
    let (app, pool, _) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria_id = crear_categoria(&app, &cookie).await;
    let producto_id = crear_producto(&app, &cookie, categoria_id).await;
    let deposito_id = crear_deposito(&app, &cookie).await;
    let proveedor_id = crear_proveedor(&app, &cookie).await;
    let (_, detalle) = pedir(&app, &cookie, "POST", "/api/compras", Some(json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [{ "producto_id": producto_id, "cantidad": 4, "costo_unitario": 2.0 }]
    }))).await;
    let compra_id = detalle["compra"]["id"].as_i64().expect("id compra") as i32;
    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/compras/{compra_id}/confirmar"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = pedir(&app, &cookie, "POST", "/api/movimientos", Some(json!({
        "tipo": "SALIDA", "deposito_id": deposito_id, "motivo": "DEVOLUCION",
        "observacion": null,
        "lineas": [{ "producto_id": producto_id, "ubicacion_id": null, "cantidad_delta": -4 }]
    }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let movimientos_antes: i64 = sqlx::query_scalar("SELECT count(*) FROM movimientos WHERE compra_id = $1")
        .bind(compra_id).fetch_one(&pool).await.expect("movimientos");
    let (status, error) = pedir(&app, &cookie, "POST", &format!("/api/compras/{compra_id}/anular"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["codigo"], "stock_insuficiente");
    let (_, actual) = pedir(&app, &cookie, "GET", &format!("/api/compras/{compra_id}"), None).await;
    assert_eq!(actual["compra"]["estado"], "CONFIRMADA");
    assert_eq!(stock_en_deposito(&pool, producto_id, deposito_id).await, 0);
    let movimientos_despues: i64 = sqlx::query_scalar("SELECT count(*) FROM movimientos WHERE compra_id = $1")
        .bind(compra_id).fetch_one(&pool).await.expect("movimientos");
    assert_eq!(movimientos_antes, movimientos_despues);
}

#[tokio::test]
async fn escenario_12_1_duplicado_exacto_y_baja_logica_impide_compra() {
    let (app, pool, _) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let nombre = format!("Proveedor {}", sku_unico("dup12"));
    let (status, _) = pedir(&app, &cookie, "POST", "/api/proveedores", Some(json!({ "nombre": nombre }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, error) = pedir(&app, &cookie, "POST", "/api/proveedores", Some(json!({ "nombre": nombre }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["codigo"], "proveedor_duplicado");
    assert_eq!(error["error"]["mensaje"], "Ya existe un proveedor con ese nombre.");
    let categoria_id = crear_categoria(&app, &cookie).await;
    let producto_id = crear_producto(&app, &cookie, categoria_id).await;
    let deposito_id = crear_deposito(&app, &cookie).await;
    let proveedor_id = crear_proveedor(&app, &cookie).await;
    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/proveedores/{proveedor_id}/desactivar"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, error) = pedir(&app, &cookie, "POST", "/api/compras", Some(json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [{ "producto_id": producto_id, "cantidad": 1, "costo_unitario": 1.0 }]
    }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["error"]["codigo"], "validacion");
    assert_eq!(error["error"]["mensaje"], "El proveedor está inactivo.");
}

#[tokio::test]
async fn fr_046_confirmar_valida_producto_activo_sin_mover_stock() {
    let (app, pool, _) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria_id = crear_categoria(&app, &cookie).await;
    let producto_id = crear_producto(&app, &cookie, categoria_id).await;
    let deposito_id = crear_deposito(&app, &cookie).await;
    let proveedor_id = crear_proveedor(&app, &cookie).await;
    let (_, detalle) = pedir(&app, &cookie, "POST", "/api/compras", Some(json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [{ "producto_id": producto_id, "cantidad": 2, "costo_unitario": 3.0 }]
    }))).await;
    let compra_id = detalle["compra"]["id"].as_i64().expect("id compra") as i32;
    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/productos/{producto_id}/desactivar"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, error) = pedir(&app, &cookie, "POST", &format!("/api/compras/{compra_id}/confirmar"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["codigo"], "producto_inactivo");
    let (_, actual) = pedir(&app, &cookie, "GET", &format!("/api/compras/{compra_id}"), None).await;
    assert_eq!(actual["compra"]["estado"], "BORRADOR");
    assert_eq!(stock_en_deposito(&pool, producto_id, deposito_id).await, 0);
    let movimientos: i64 = sqlx::query_scalar("SELECT count(*) FROM movimientos WHERE compra_id = $1")
        .bind(compra_id).fetch_one(&pool).await.expect("movimientos");
    assert_eq!(movimientos, 0);
}

#[tokio::test]
async fn validaciones_de_edicion_no_devuelven_500() {
    let (app, pool, _) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria_id = crear_categoria(&app, &cookie).await;
    let producto_id = crear_producto(&app, &cookie, categoria_id).await;
    let deposito_id = crear_deposito(&app, &cookie).await;
    let proveedor_id = crear_proveedor(&app, &cookie).await;

    let (_, detalle) = pedir(&app, &cookie, "POST", "/api/compras", Some(json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [{ "producto_id": producto_id, "cantidad": 1, "costo_unitario": 2.0 }]
    }))).await;
    let compra_id = detalle["compra"]["id"].as_i64().expect("id compra") as i32;

    // A1: PUT sin líneas → 422 validación, no 500.
    let (status, error) = pedir(&app, &cookie, "PUT", &format!("/api/compras/{compra_id}"), Some(json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id, "lineas": []
    }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["codigo"], "validacion");

    // A2: cantidad negativa → 422 validación, no 500.
    let (status, error) = pedir(&app, &cookie, "PUT", &format!("/api/compras/{compra_id}"), Some(json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [{ "producto_id": producto_id, "cantidad": -3, "costo_unitario": 2.0 }]
    }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["codigo"], "validacion");

    // A3: producto repetido → 422 en alta y en edición (evita el 500 de
    // movimiento_lineas al confirmar).
    let duplicada = json!({
        "proveedor_id": proveedor_id, "deposito_id": deposito_id,
        "lineas": [
            { "producto_id": producto_id, "cantidad": 1, "costo_unitario": 2.0 },
            { "producto_id": producto_id, "cantidad": 2, "costo_unitario": 2.0 }
        ]
    });
    let (status, error) = pedir(&app, &cookie, "POST", "/api/compras", Some(duplicada.clone())).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["codigo"], "validacion");
    let (status, error) = pedir(&app, &cookie, "PUT", &format!("/api/compras/{compra_id}"), Some(duplicada)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["codigo"], "validacion");

    // A4: CUIT duplicado en edición → 409 cuit_duplicado, no 500.
    let cuit_a = cuit_unico();
    let cuit_b = cuit_unico();
    let (_, prov_a) = pedir(&app, &cookie, "POST", "/api/proveedores", Some(json!({
        "nombre": format!("Proveedor A {}", sku_unico("cuit")), "cuit": cuit_a
    }))).await;
    let (status, _) = pedir(&app, &cookie, "POST", "/api/proveedores", Some(json!({
        "nombre": format!("Proveedor B {}", sku_unico("cuit")), "cuit": cuit_b
    }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let id_a = prov_a["id"].as_i64().expect("id proveedor A") as i32;
    let (status, error) = pedir(&app, &cookie, "PUT", &format!("/api/proveedores/{id_a}"), Some(json!({
        "nombre": prov_a["nombre"], "cuit": cuit_b
    }))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["codigo"], "cuit_duplicado");
}