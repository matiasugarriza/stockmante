//! 009 — integración HTTP de conteos de inventario físico (T008–T010).
//!
//! Escenarios 9.1–9.7 y SC-017: apertura con snapshot, carga, cierre con un único
//! `AJUSTE` (motivo `INVENTARIO`) que deja el stock igual a lo contado, cancelación,
//! códigos de error específicos y RBAC (solo GESTION).

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, crear_conteo_de_prueba, cuerpo_json, sesion_gestion, sesion_operacion, solicitud,
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

async fn abrir(app: &Router, cookie: &str, deposito: i32) -> Value {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/conteos",
        Some(json!({ "deposito_id": deposito, "observacion": "  conteo mensual  " })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body
}

async fn stock(pool: &PgPool, producto: i32, deposito: i32) -> i32 {
    sqlx::query_scalar(
        "SELECT cantidad FROM stock_por_deposito WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL",
    )
    .bind(producto)
    .bind(deposito)
    .fetch_one(pool)
    .await
    .expect("stock")
}

async fn cache(pool: &PgPool, producto: i32) -> i32 {
    sqlx::query_scalar("SELECT stock_actual FROM productos WHERE id = $1")
        .bind(producto)
        .fetch_one(pool)
        .await
        .expect("cache")
}

fn linea_de(conteo: &Value, producto: i32) -> Value {
    conteo["lineas"]
        .as_array()
        .and_then(|ls| ls.iter().find(|l| l["producto_id"] == producto).cloned())
        .unwrap_or(Value::Null)
}

/// T008: alta, snapshot, depósito inactivo/sin stock, consultas y 404.
#[tokio::test]
async fn alta_y_consulta_de_conteos() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_conteo_de_prueba(&pool, 10).await;

    let conteo = abrir(&app, &cookie, e.deposito_id).await;
    assert_eq!(conteo["estado"], "ABIERTO");
    assert_eq!(conteo["observacion"], "conteo mensual");
    let linea = linea_de(&conteo, e.producto_id);
    assert_eq!(linea["cantidad_sistema"], 10);
    assert_eq!(linea["cantidad_contada"], Value::Null);
    assert_eq!(conteo["lineas"].as_array().map(Vec::len), Some(1));
    let id = conteo["id"].as_i64().expect("id");

    let (status, body) = pedir(&app, &cookie, "GET", &format!("/api/conteos/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], id);

    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/conteos?deposito_id={}&estado=ABIERTO", e.deposito_id),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["id"], id);

    let (status, body) = pedir(&app, &cookie, "GET", "/api/conteos/999999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["codigo"], "conteo_no_encontrado");

    // Depósito inactivo → 422 deposito_inactivo.
    sqlx::query("UPDATE depositos SET activo = FALSE WHERE id = $1")
        .bind(e.deposito_id)
        .execute(&pool)
        .await
        .expect("desactivar depósito");
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/conteos",
        Some(json!({ "deposito_id": e.deposito_id })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "deposito_inactivo");

    // Depósito sin filas de stock → 422 validacion sin crear conteo.
    let vacio = crear_conteo_de_prueba(&pool, 0).await;
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/conteos",
        Some(json!({ "deposito_id": vacio.deposito_id })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "validacion");
    assert_eq!(
        body["error"]["mensaje"],
        "El depósito no tiene stock registrado para contar."
    );
    let creados: i64 = sqlx::query_scalar("SELECT count(*) FROM conteos WHERE deposito_id = $1")
        .bind(vacio.deposito_id)
        .fetch_one(&pool)
        .await
        .expect("conteos");
    assert_eq!(creados, 0);
}

/// T009: carga (diferencia contra snapshot) y cierre con y sin diferencias.
#[tokio::test]
async fn carga_y_cierre_ajustan_al_contado() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_conteo_de_prueba(&pool, 10).await;
    let conteo = abrir(&app, &cookie, e.deposito_id).await;
    let id = conteo["id"].as_i64().expect("id");
    let linea_id = linea_de(&conteo, e.producto_id)["id"].clone();

    // Cerrar con líneas sin contar → 422 lineas_sin_contar.
    let (status, body) =
        pedir(&app, &cookie, "POST", &format!("/api/conteos/{id}/cerrar"), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "lineas_sin_contar");

    let (status, body) = pedir(
        &app,
        &cookie,
        "PATCH",
        &format!("/api/conteos/{id}/lineas"),
        Some(json!({ "lineas": [{ "linea_id": linea_id, "cantidad_contada": 8 }] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(linea_de(&body, e.producto_id)["diferencia"], -2);

    let (status, body) =
        pedir(&app, &cookie, "POST", &format!("/api/conteos/{id}/cerrar"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["estado"], "CERRADO");
    let movimiento = body["movimiento_ajuste_id"].as_i64().expect("ajuste generado");
    assert_eq!(stock(&pool, e.producto_id, e.deposito_id).await, 8);
    assert_eq!(cache(&pool, e.producto_id).await, 8);

    let (status, mov) =
        pedir(&app, &cookie, "GET", &format!("/api/movimientos/{movimiento}"), None).await;
    assert_eq!(status, StatusCode::OK, "{mov}");
    assert_eq!(mov["tipo"], "AJUSTE");
    assert_eq!(mov["motivo"], "INVENTARIO");
    assert_eq!(mov["lineas"].as_array().map(Vec::len), Some(1));
    assert_eq!(mov["lineas"][0]["cantidad_delta"], -2);
    assert_eq!(mov["lineas"][0]["stock_resultante"], 8);

    // 9.5: un CERRADO rechaza cambios.
    let (status, body) =
        pedir(&app, &cookie, "POST", &format!("/api/conteos/{id}/cerrar"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "conteo_cerrado");

    // 9.4: sin diferencias no genera movimiento.
    let otro = abrir(&app, &cookie, e.deposito_id).await;
    let otro_id = otro["id"].as_i64().expect("id");
    let otra_linea = linea_de(&otro, e.producto_id)["id"].clone();
    let (status, _) = pedir(
        &app,
        &cookie,
        "PATCH",
        &format!("/api/conteos/{otro_id}/lineas"),
        Some(json!({ "lineas": [{ "linea_id": otra_linea, "cantidad_contada": 8 }] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) =
        pedir(&app, &cookie, "POST", &format!("/api/conteos/{otro_id}/cerrar"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["movimiento_ajuste_id"], Value::Null);
    assert_eq!(stock(&pool, e.producto_id, e.deposito_id).await, 8);
}

/// SC-017: si el stock cambia entre la apertura y el cierre, el cierre ajusta
/// contra el stock vigente y el resultado es exactamente lo contado.
#[tokio::test]
async fn cierre_ajusta_contra_el_stock_vigente() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_conteo_de_prueba(&pool, 10).await;
    let conteo = abrir(&app, &cookie, e.deposito_id).await;
    let id = conteo["id"].as_i64().expect("id");
    let linea_id = linea_de(&conteo, e.producto_id)["id"].clone();

    // Entrada de 5 después de abrir: vigente 15, snapshot 10.
    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/movimientos",
        Some(json!({
            "tipo": "ENTRADA",
            "deposito_id": e.deposito_id,
            "motivo": "COMPRA",
            "lineas": [{ "producto_id": e.producto_id, "ubicacion_id": null, "cantidad_delta": 5 }],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    // Se cuenta 10 (coincide con el snapshot pero no con el vigente).
    let (status, _) = pedir(
        &app,
        &cookie,
        "PATCH",
        &format!("/api/conteos/{id}/lineas"),
        Some(json!({ "lineas": [{ "linea_id": linea_id, "cantidad_contada": 10 }] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) =
        pedir(&app, &cookie, "POST", &format!("/api/conteos/{id}/cerrar"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(stock(&pool, e.producto_id, e.deposito_id).await, 10);
    assert_eq!(cache(&pool, e.producto_id).await, 10);
    let movimiento = body["movimiento_ajuste_id"].as_i64().expect("ajuste por la diferencia vigente");
    let (_, mov) = pedir(&app, &cookie, "GET", &format!("/api/movimientos/{movimiento}"), None).await;
    assert_eq!(mov["lineas"][0]["cantidad_delta"], -5);
    assert_eq!(mov["lineas"][0]["stock_resultante"], 10);
}

/// T010: cancelación, 409 conteo_cancelado y RBAC.
#[tokio::test]
async fn cancelacion_y_rbac() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operacion = sesion_operacion(&pool).await;
    let e = crear_conteo_de_prueba(&pool, 4).await;
    let conteo = abrir(&app, &cookie, e.deposito_id).await;
    let id = conteo["id"].as_i64().expect("id");
    let linea_id = linea_de(&conteo, e.producto_id)["id"].clone();

    let (status, body) =
        pedir(&app, &cookie, "POST", &format!("/api/conteos/{id}/cancelar"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["estado"], "CANCELADO");
    assert_eq!(body["movimiento_ajuste_id"], Value::Null);
    assert_eq!(stock(&pool, e.producto_id, e.deposito_id).await, 4);

    for (metodo, uri, cuerpo) in [
        (
            "PATCH",
            format!("/api/conteos/{id}/lineas"),
            Some(json!({ "lineas": [{ "linea_id": linea_id, "cantidad_contada": 1 }] })),
        ),
        ("POST", format!("/api/conteos/{id}/cerrar"), None),
        ("POST", format!("/api/conteos/{id}/cancelar"), None),
    ] {
        let (status, body) = pedir(&app, &cookie, metodo, &uri, cuerpo).await;
        assert_eq!(status, StatusCode::CONFLICT, "{metodo} {uri}: {body}");
        assert_eq!(body["error"]["codigo"], "conteo_cancelado", "{metodo} {uri}");
    }

    for (metodo, uri, cuerpo) in [
        ("POST", "/api/conteos".to_string(), Some(json!({ "deposito_id": e.deposito_id }))),
        ("GET", "/api/conteos".to_string(), None),
        ("GET", format!("/api/conteos/{id}"), None),
        ("PATCH", format!("/api/conteos/{id}/lineas"), Some(json!({ "lineas": [] }))),
        ("POST", format!("/api/conteos/{id}/cerrar"), None),
        ("POST", format!("/api/conteos/{id}/cancelar"), None),
    ] {
        let (status, body) = pedir(&app, &operacion, metodo, &uri, cuerpo).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{metodo} {uri}: {body}");
        assert_eq!(body["error"]["codigo"], "sin_permisos");
    }
}
