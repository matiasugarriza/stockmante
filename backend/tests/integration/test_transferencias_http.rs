//! 008 — integración HTTP de transferencias entre depósitos.
//!
//! Escenarios 8.1–8.5, SC-016 (éxito y fallas), auditoría (FR-033), stock por
//! fila de ubicación (decisión 9), desborde en destino (decisión 10), movimientos
//! por id (decisión 11) y 404 con siguiente paso (T033).

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, crear_transferencia_de_prueba, cuerpo_json, sesion_con_familia, sesion_gestion,
    solicitud, EscenarioTransferencia,
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

async fn transferir(app: &Router, cookie: &str, cuerpo: Value) -> (StatusCode, Value) {
    pedir(app, cookie, "POST", "/api/transferencias", Some(cuerpo)).await
}

fn cuerpo(e: &EscenarioTransferencia, cantidad: i32) -> Value {
    json!({
        "producto_id": e.producto_id,
        "deposito_origen_id": e.origen_id,
        "deposito_destino_id": e.destino_id,
        "cantidad": cantidad,
    })
}

async fn stock_fila(pool: &PgPool, producto: i32, deposito: i32, ubicacion: Option<i32>) -> i32 {
    sqlx::query_scalar::<_, i32>(
        r#"SELECT COALESCE((SELECT cantidad FROM stock_por_deposito
             WHERE producto_id = $1 AND deposito_id = $2
               AND ubicacion_id IS NOT DISTINCT FROM $3), 0)"#,
    )
    .bind(producto)
    .bind(deposito)
    .bind(ubicacion)
    .fetch_one(pool)
    .await
    .expect("stock de la fila")
}

/// Fotografía de SC-016: suma global, cache y filas generadas por el producto.
async fn foto(pool: &PgPool, producto: i32) -> (i64, i32, i64, i64, i64) {
    let suma: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(cantidad), 0)::bigint FROM stock_por_deposito WHERE producto_id = $1",
    )
    .bind(producto)
    .fetch_one(pool)
    .await
    .expect("suma");
    let cache: i32 = sqlx::query_scalar("SELECT stock_actual FROM productos WHERE id = $1")
        .bind(producto)
        .fetch_one(pool)
        .await
        .expect("cache");
    let transferencias: i64 =
        sqlx::query_scalar("SELECT count(*) FROM transferencias WHERE producto_id = $1")
            .bind(producto)
            .fetch_one(pool)
            .await
            .expect("transferencias");
    let movimientos: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM movimiento_lineas WHERE producto_id = $1",
    )
    .bind(producto)
    .fetch_one(pool)
    .await
    .expect("movimientos");
    let auditoria: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM auditoria a JOIN transferencias t ON t.id = a.registro_id
            WHERE a.tabla_ref = 'transferencias' AND t.producto_id = $1"#,
    )
    .bind(producto)
    .fetch_one(pool)
    .await
    .expect("auditoría");
    (suma, cache, transferencias, movimientos, auditoria)
}

async fn nombre_deposito(pool: &PgPool, id: i32) -> String {
    sqlx::query_scalar("SELECT nombre FROM depositos WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("nombre del depósito")
}

/// T007 (a, f) + T027: alta correcta, saldos, SC-016 y una fila de auditoría.
#[tokio::test]
async fn transferencia_exitosa_mueve_stock_y_audita() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "CAJERO", "OPERACION").await;
    let actor: i32 = sqlx::query_scalar("SELECT id FROM usuarios WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .expect("actor");
    let e = crear_transferencia_de_prueba(&pool, 20).await;
    let (suma_antes, ..) = foto(&pool, e.producto_id).await;

    let (status, body) = transferir(&app, &cookie, cuerpo(&e, 8)).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["stock_origen_resultante"], 12);
    assert_eq!(body["stock_destino_resultante"], 8);
    assert_eq!(body["usuario"]["id"], actor);
    assert_eq!(stock_fila(&pool, e.producto_id, e.origen_id, None).await, 12);
    assert_eq!(stock_fila(&pool, e.producto_id, e.destino_id, None).await, 8);

    let (suma, cache, transferencias, _, _) = foto(&pool, e.producto_id).await;
    assert_eq!(suma, suma_antes, "SC-016: la suma global no cambia");
    assert_eq!(i64::from(cache), suma);
    assert_eq!(transferencias, 1);

    let id = body["id"].as_i64().expect("id") as i32;
    let filas: Vec<(String, Option<i32>)> = sqlx::query_as(
        "SELECT accion, usuario_id FROM auditoria WHERE tabla_ref = 'transferencias' AND registro_id = $1",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .expect("auditoría");
    assert_eq!(filas, vec![("CREAR".to_string(), Some(actor))]);
}

/// T007 (b–e) + T031: fallas con mensaje literal y SC-016 intacto.
#[tokio::test]
async fn fallas_no_escriben_nada() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 3).await;
    let antes = foto(&pool, e.producto_id).await;

    // (b) stock insuficiente.
    let (status, body) = transferir(&app, &cookie, cuerpo(&e, 5)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "stock_insuficiente");
    let origen = nombre_deposito(&pool, e.origen_id).await;
    assert_eq!(
        body["error"]["mensaje"],
        format!(
            "Stock insuficiente en {origen}: 3 disponibles, 5 solicitadas. Reducí la cantidad o registrá antes una entrada en ese depósito."
        )
    );
    assert_eq!(foto(&pool, e.producto_id).await, antes);

    // (c) mismo depósito.
    let mut mismo = cuerpo(&e, 1);
    mismo["deposito_destino_id"] = json!(e.origen_id);
    let (status, body) = transferir(&app, &cookie, mismo).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "transferencia_mismo_deposito");
    assert_eq!(
        body["error"]["mensaje"],
        "El depósito de origen y el de destino deben ser distintos."
    );
    assert_eq!(foto(&pool, e.producto_id).await, antes);

    // (d) destino inactivo.
    sqlx::query("UPDATE depositos SET activo = FALSE WHERE id = $1")
        .bind(e.destino_id)
        .execute(&pool)
        .await
        .expect("desactivar destino");
    let (status, body) = transferir(&app, &cookie, cuerpo(&e, 1)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "validacion");
    assert_eq!(foto(&pool, e.producto_id).await, antes);
    sqlx::query("UPDATE depositos SET activo = TRUE WHERE id = $1")
        .bind(e.destino_id)
        .execute(&pool)
        .await
        .expect("reactivar destino");

    // (e) producto inactivo.
    sqlx::query("UPDATE productos SET activo = FALSE WHERE id = $1")
        .bind(e.producto_id)
        .execute(&pool)
        .await
        .expect("desactivar producto");
    let (status, body) = transferir(&app, &cookie, cuerpo(&e, 1)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "producto_inactivo");
    assert_eq!(foto(&pool, e.producto_id).await, antes);
}

async fn ubicacion(app: &Router, cookie: &str, deposito: i32, nombre: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/ubicaciones",
        Some(json!({ "deposito_id": deposito, "nombre": nombre, "tipo": "PASILLO" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_i64().expect("id") as i32
}

/// T009: ubicaciones de origen y destino.
#[tokio::test]
async fn ubicaciones_de_origen_y_destino() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 10).await;
    let a01 = ubicacion(&app, &cookie, e.destino_id, "A-01").await;
    let b01 = ubicacion(&app, &cookie, e.destino_id, "B-01").await;
    let ajena = ubicacion(&app, &cookie, e.origen_id, "Z-01").await;
    let (status, _) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/productos/{}/ubicaciones", e.producto_id),
        Some(json!({ "ubicacion_ids": [a01, ajena] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // (a) destino con ubicación permitida → el stock queda en esa fila.
    let mut ok = cuerpo(&e, 4);
    ok["ubicacion_destino_id"] = json!(a01);
    let (status, body) = transferir(&app, &cookie, ok).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(stock_fila(&pool, e.producto_id, e.destino_id, Some(a01)).await, 4);
    assert_eq!(stock_fila(&pool, e.producto_id, e.destino_id, None).await, 0);

    // (b) ubicación de otro depósito → 422.
    let mut ajena_destino = cuerpo(&e, 1);
    ajena_destino["ubicacion_destino_id"] = json!(ajena);
    let (status, body) = transferir(&app, &cookie, ajena_destino).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    // (c) ubicación no permitida para el producto → 422.
    let mut no_permitida = cuerpo(&e, 1);
    no_permitida["ubicacion_destino_id"] = json!(b01);
    let (status, body) = transferir(&app, &cookie, no_permitida).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(stock_fila(&pool, e.producto_id, e.destino_id, Some(b01)).await, 0);
}

/// T028: stock por fila de ubicación, saldos iguales a `movimiento_lineas` y desborde.
#[tokio::test]
async fn stock_por_fila_saldos_y_desborde_en_destino() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 0).await;
    let a01 = ubicacion(&app, &cookie, e.origen_id, "A-01").await;
    sqlx::query(
        "INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad) VALUES ($1, $2, $3, 20)",
    )
    .bind(e.producto_id)
    .bind(e.origen_id)
    .bind(a01)
    .execute(&pool)
    .await
    .expect("stock en A-01");
    sqlx::query("UPDATE productos SET stock_actual = 20 WHERE id = $1")
        .bind(e.producto_id)
        .execute(&pool)
        .await
        .expect("cache");

    // (a) sin ubicación de origen se valida la fila del depósito: 0 disponibles.
    let antes = foto(&pool, e.producto_id).await;
    let (status, body) = transferir(&app, &cookie, cuerpo(&e, 5)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(
        body["error"]["mensaje"]
            .as_str()
            .is_some_and(|m| m.contains("0 disponibles, 5 solicitadas")),
        "{body}"
    );
    assert_eq!(foto(&pool, e.producto_id).await, antes);

    // (b) con ubicación de origen pasa; los saldos coinciden con las líneas.
    sqlx::query("INSERT INTO producto_ubicacion_permitida (producto_id, ubicacion_id) VALUES ($1, $2)")
        .bind(e.producto_id)
        .bind(a01)
        .execute(&pool)
        .await
        .expect("asignación");
    let mut con_ubicacion = cuerpo(&e, 5);
    con_ubicacion["ubicacion_origen_id"] = json!(a01);
    let (status, body) = transferir(&app, &cookie, con_ubicacion).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    for (campo, movimiento) in [
        ("stock_origen_resultante", "movimiento_salida_id"),
        ("stock_destino_resultante", "movimiento_entrada_id"),
    ] {
        let linea: i32 = sqlx::query_scalar(
            "SELECT stock_resultante FROM movimiento_lineas WHERE movimiento_id = $1",
        )
        .bind(body[movimiento].as_i64().expect("id de movimiento") as i32)
        .fetch_one(&pool)
        .await
        .expect("línea");
        assert_eq!(body[campo], linea, "{campo}");
    }
    assert_eq!(body["stock_origen_resultante"], 15);
    assert_eq!(body["stock_destino_resultante"], 5);

    // (c) destino en i32::MAX → 422 sin escribir.
    sqlx::query(
        "UPDATE stock_por_deposito SET cantidad = $3 WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL",
    )
    .bind(e.producto_id)
    .bind(e.destino_id)
    .bind(i32::MAX)
    .execute(&pool)
    .await
    .expect("destino al máximo");
    let antes = foto(&pool, e.producto_id).await;
    let mut desborde = cuerpo(&e, 1);
    desborde["ubicacion_origen_id"] = json!(a01);
    let (status, body) = transferir(&app, &cookie, desborde).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "validacion");
    assert_eq!(foto(&pool, e.producto_id).await, antes);

    // Limpieza: el saldo en i32::MAX desbordaría los agregados de otros tests.
    sqlx::query(
        "UPDATE stock_por_deposito SET cantidad = 0 WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL",
    )
    .bind(e.producto_id)
    .bind(e.destino_id)
    .execute(&pool)
    .await
    .expect("limpiar el saldo máximo");
}

/// T008 + T030: listado, filtros, detalle y movimientos por id.
#[tokio::test]
async fn listado_detalle_y_movimientos_enlazados() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 9).await;
    let mut ids = Vec::new();
    for cantidad in [2, 3] {
        let (status, body) = transferir(&app, &cookie, cuerpo(&e, cantidad)).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        ids.push(body["id"].as_i64().expect("id"));
    }

    for filtro in [
        format!("producto_id={}", e.producto_id),
        format!("deposito_origen_id={}", e.origen_id),
        format!("deposito_destino_id={}", e.destino_id),
    ] {
        let (status, body) =
            pedir(&app, &cookie, "GET", &format!("/api/transferencias?{filtro}"), None).await;
        assert_eq!(status, StatusCode::OK, "{filtro}: {body}");
        assert_eq!(body["total"], 2, "{filtro}: {body}");
        let devueltos: Vec<i64> = body["items"]
            .as_array()
            .expect("items")
            .iter()
            .filter_map(|i| i["id"].as_i64())
            .collect();
        assert_eq!(devueltos, vec![ids[1], ids[0]], "orden por fecha descendente");
    }

    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/transferencias?producto_id={}&por_pagina=1&pagina=2", e.producto_id),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["items"][0]["id"], ids[0]);

    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/transferencias?producto_id={}&desde=2000-01-01&hasta=2000-01-02", e.producto_id),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total"], 0);

    let (status, detalle) =
        pedir(&app, &cookie, "GET", &format!("/api/transferencias/{}", ids[0]), None).await;
    assert_eq!(status, StatusCode::OK, "{detalle}");
    for (campo, tipo, deposito, delta) in [
        ("movimiento_salida_id", "SALIDA", e.origen_id, -2),
        ("movimiento_entrada_id", "ENTRADA", e.destino_id, 2),
    ] {
        let id = detalle[campo].as_i64().expect(campo);
        let (status, movimiento) =
            pedir(&app, &cookie, "GET", &format!("/api/movimientos/{id}"), None).await;
        assert_eq!(status, StatusCode::OK, "{movimiento}");
        assert_eq!(movimiento["tipo"], tipo);
        assert_eq!(movimiento["motivo"], "TRANSFERENCIA");
        assert_eq!(movimiento["deposito"]["id"], deposito);
        assert_eq!(movimiento["lineas"].as_array().map(Vec::len), Some(1));
        assert_eq!(movimiento["lineas"][0]["cantidad_delta"], delta);
    }
}

/// T033: 404 con siguiente paso.
#[tokio::test]
async fn detalle_inexistente_responde_404_con_siguiente_paso() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let (status, body) = pedir(&app, &cookie, "GET", "/api/transferencias/999999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(
        body["error"]["mensaje"],
        "No existe la transferencia 999999. Revisá el número o buscala en el listado de transferencias."
    );
}
