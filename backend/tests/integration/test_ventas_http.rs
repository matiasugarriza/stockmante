//! 013 (T009) — ventas de caja: escenarios 13.1–13.9 y SC-021.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, crear_transferencia_de_prueba, cuerpo_json, sesion_gestion, sesion_operacion,
    solicitud,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

async fn pedir(app: &Router, cookie: &str, metodo: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud(metodo, uri, cookie, body))
        .await
        .expect("request de prueba");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

/// Producto con `stock` en un depósito propio y precio `precio`.
async fn escenario(pool: &PgPool, stock: i32, precio: i32) -> (i32, i32) {
    let e = crear_transferencia_de_prueba(pool, stock).await;
    sqlx::query("UPDATE productos SET precio_venta = $2 WHERE id = $1")
        .bind(e.producto_id)
        .bind(precio)
        .execute(pool)
        .await
        .expect("precio");
    (e.producto_id, e.origen_id)
}

/// Segundo producto en el mismo depósito.
async fn otro_producto(pool: &PgPool, deposito: i32, stock: i32, precio: i32) -> i32 {
    let e = crear_transferencia_de_prueba(pool, 0).await;
    sqlx::query("UPDATE productos SET precio_venta = $2, stock_actual = $3 WHERE id = $1")
        .bind(e.producto_id)
        .bind(precio)
        .bind(stock)
        .execute(pool)
        .await
        .expect("producto");
    sqlx::query("INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad) VALUES ($1, $2, NULL, $3)")
        .bind(e.producto_id)
        .bind(deposito)
        .bind(stock)
        .execute(pool)
        .await
        .expect("stock");
    e.producto_id
}

async fn stock(pool: &PgPool, producto: i32, deposito: i32) -> i32 {
    sqlx::query_scalar("SELECT COALESCE((SELECT cantidad FROM stock_por_deposito WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL), 0)")
        .bind(producto)
        .bind(deposito)
        .fetch_one(pool)
        .await
        .expect("stock")
}

async fn ventas_del_deposito(pool: &PgPool, deposito: i32) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM ventas WHERE deposito_id = $1")
        .bind(deposito)
        .fetch_one(pool)
        .await
        .expect("ventas")
}

/// 13.1 + 13.9 + 13.7: venta con líneas repetidas agregadas, descuento y stock resultante.
#[tokio::test]
async fn venta_descuenta_stock_y_devuelve_stock_resultante() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let (cemento, deposito) = escenario(&pool, 10, 3000).await;
    let arena = otro_producto(&pool, deposito, 5, 6000).await;

    let (status, v) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito,
        "medio_pago": "EFECTIVO",
        "descuento": 1000,
        "lineas": [
            { "producto_id": cemento, "cantidad": 2 },
            { "producto_id": cemento, "cantidad": 1 },
            { "producto_id": arena, "cantidad": 1 }
        ]
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    assert!(v["venta"]["numero"].as_str().is_some_and(|n| n.starts_with("V-")));
    assert_eq!(v["venta"]["subtotal"].as_f64(), Some(15000.0));
    assert_eq!(v["venta"]["total"].as_f64(), Some(14000.0));
    let lineas = v["lineas"].as_array().expect("lineas");
    assert_eq!(lineas.len(), 2, "una línea por producto: {v}");
    assert_eq!(lineas[0]["producto_id"], cemento);
    assert_eq!(lineas[0]["cantidad"], 3);
    assert_eq!(lineas[0]["stock_resultante"], 7);
    assert_eq!(lineas[1]["stock_resultante"], 4);
    assert_eq!(stock(&pool, cemento, deposito).await, 7);

    // El detalle trae el mismo stock resultante.
    let id = v["venta"]["id"].as_i64().expect("id");
    let (status, d) = pedir(&app, &cookie, "GET", &format!("/api/ventas/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(d["lineas"][0]["stock_resultante"], 7);
}

/// 13.2 + 13.3 + SC-021: stock insuficiente (cantidad agregada) no registra nada.
#[tokio::test]
async fn stock_insuficiente_sobre_la_cantidad_agregada_no_registra_nada() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let (producto, deposito) = escenario(&pool, 2, 100).await;
    let otro = otro_producto(&pool, deposito, 9, 50).await;
    let nombre: String = sqlx::query_scalar("SELECT nombre FROM depositos WHERE id = $1")
        .bind(deposito)
        .fetch_one(&pool)
        .await
        .expect("nombre");

    let (status, e) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito,
        "medio_pago": "TARJETA",
        "lineas": [
            { "producto_id": otro, "cantidad": 1 },
            { "producto_id": producto, "cantidad": 2 },
            { "producto_id": producto, "cantidad": 1 }
        ]
    }))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{e}");
    assert_eq!(e["error"]["codigo"], "stock_insuficiente");
    assert_eq!(
        e["error"]["mensaje"],
        format!("Stock insuficiente en {nombre}: 2 disponibles, 3 solicitadas. Reducí la cantidad o registrá antes una entrada en ese depósito.")
    );
    assert_eq!(ventas_del_deposito(&pool, deposito).await, 0);
    assert_eq!(stock(&pool, otro, deposito).await, 9, "atomicidad multi-línea");
}

/// 13.4 + 13.5: producto inactivo y precio enviado → 422 sin registrar.
#[tokio::test]
async fn producto_inactivo_y_precio_enviado_son_422() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let (producto, deposito) = escenario(&pool, 5, 100).await;

    let (status, e) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito,
        "medio_pago": "EFECTIVO",
        "lineas": [{ "producto_id": producto, "cantidad": 1, "precio_unitario": 1 }]
    }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(
        e["error"]["mensaje"],
        "El precio unitario se toma del catálogo; no se puede enviar en la venta."
    );

    sqlx::query("UPDATE productos SET activo = FALSE WHERE id = $1")
        .bind(producto)
        .execute(&pool)
        .await
        .expect("desactivar");
    let (status, e) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito,
        "medio_pago": "EFECTIVO",
        "lineas": [{ "producto_id": producto, "cantidad": 1 }]
    }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["error"]["mensaje"], "El producto no está disponible para la venta.");
    assert_eq!(ventas_del_deposito(&pool, deposito).await, 0);
}

/// 13.5: el precio queda congelado aunque el catálogo cambie después.
#[tokio::test]
async fn precio_congelado_al_confirmar() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let (producto, deposito) = escenario(&pool, 5, 100).await;
    let (status, v) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito, "medio_pago": "EFECTIVO",
        "lineas": [{ "producto_id": producto, "cantidad": 2 }]
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    sqlx::query("UPDATE productos SET precio_venta = 999 WHERE id = $1")
        .bind(producto)
        .execute(&pool)
        .await
        .expect("precio nuevo");
    let id = v["venta"]["id"].as_i64().expect("id");
    let (_, d) = pedir(&app, &cookie, "GET", &format!("/api/ventas/{id}"), None).await;
    assert_eq!(d["lineas"][0]["precio_unitario"].as_f64(), Some(100.0));
    assert_eq!(d["venta"]["total"].as_f64(), Some(200.0));
}

/// 13.6 + FR-055: anular repone stock (GESTION); doble anulación 409; OPERACION 403.
#[tokio::test]
async fn anulacion_repone_stock_y_solo_gestion() {
    let (app, pool, _correo) = app_de_test().await;
    let operacion = sesion_operacion(&pool).await;
    let gestion = sesion_gestion(&pool).await;
    let (producto, deposito) = escenario(&pool, 5, 100).await;
    let (status, v) = pedir(&app, &operacion, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito, "medio_pago": "EFECTIVO",
        "lineas": [{ "producto_id": producto, "cantidad": 2 }]
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    let id = v["venta"]["id"].as_i64().expect("id");
    let anular = format!("/api/ventas/{id}/anular");

    let (status, e) = pedir(&app, &operacion, "POST", &anular, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{e}");
    assert_eq!(e["error"]["codigo"], "sin_permisos");

    let (status, a) = pedir(&app, &gestion, "POST", &anular, None).await;
    assert_eq!(status, StatusCode::OK, "{a}");
    assert_eq!(a["venta"]["estado"], "ANULADA");
    assert!(a["venta"]["anulado_en"].is_string());
    assert_eq!(a["lineas"][0]["stock_resultante"], 5);
    assert_eq!(stock(&pool, producto, deposito).await, 5);
    let devoluciones: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM movimientos WHERE venta_id = $1 AND tipo = 'ENTRADA' AND motivo = 'DEVOLUCION'",
    )
    .bind(id as i32)
    .fetch_one(&pool)
    .await
    .expect("devoluciones");
    assert_eq!(devoluciones, 1);

    let (status, e) = pedir(&app, &gestion, "POST", &anular, None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{e}");
    assert_eq!(e["error"]["codigo"], "venta_ya_anulada");
    assert_eq!(e["error"]["mensaje"], "La venta ya está anulada; no se puede anular dos veces.");
}

/// 13.7 + 13.8: descuento mayor al subtotal, medio de pago inválido y cliente opcional.
#[tokio::test]
async fn descuento_medio_de_pago_y_cliente() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let gestion = sesion_gestion(&pool).await;
    let (producto, deposito) = escenario(&pool, 5, 100).await;
    for (cuerpo, mensaje) in [
        (
            json!({ "deposito_id": deposito, "medio_pago": "EFECTIVO", "descuento": 101,
                    "lineas": [{ "producto_id": producto, "cantidad": 1 }] }),
            "El descuento no puede superar el subtotal.",
        ),
        (
            json!({ "deposito_id": deposito, "medio_pago": "CHEQUE",
                    "lineas": [{ "producto_id": producto, "cantidad": 1 }] }),
            "El medio de pago debe ser EFECTIVO, TARJETA o TRANSFERENCIA.",
        ),
    ] {
        let (status, e) = pedir(&app, &cookie, "POST", "/api/ventas", Some(cuerpo)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
        assert_eq!(e["error"]["mensaje"], mensaje);
    }

    let (status, c) = pedir(&app, &gestion, "POST", "/api/clientes", Some(json!({ "nombre": "Ana Pérez" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{c}");
    let cliente = c["id"].as_i64().expect("id");
    let (status, v) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
        "deposito_id": deposito, "cliente_id": cliente, "medio_pago": "TRANSFERENCIA",
        "lineas": [{ "producto_id": producto, "cantidad": 1 }]
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    assert_eq!(v["venta"]["cliente_nombre"], "Ana Pérez");
    assert_eq!(v["venta"]["medio_pago"], "TRANSFERENCIA");
}
