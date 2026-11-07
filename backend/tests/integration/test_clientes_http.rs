//! 013 (T008) — clientes: lectura para cualquier sesión, escritura solo GESTION,
//! documento único y baja lógica.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{app_de_test, cuerpo_json, sesion_gestion, sesion_operacion, solicitud};
use serde_json::{json, Value};
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

fn documento_unico() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..12].to_string()
}

#[tokio::test]
async fn alta_lectura_permisos_y_documento_duplicado() {
    let (app, pool, _correo) = app_de_test().await;
    let gestion = sesion_gestion(&pool).await;
    let operacion = sesion_operacion(&pool).await;
    let documento = documento_unico();

    let (status, c) = pedir(&app, &gestion, "POST", "/api/clientes", Some(json!({
        "nombre": "Cliente de prueba", "documento": documento
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{c}");
    let id = c["id"].as_i64().expect("id");

    let (status, body) = pedir(&app, &operacion, "GET", "/api/clientes", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = pedir(&app, &operacion, "GET", &format!("/api/clientes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    for (metodo, uri, cuerpo) in [
        ("POST", "/api/clientes".to_string(), Some(json!({ "nombre": "x" }))),
        ("PUT", format!("/api/clientes/{id}"), Some(json!({ "nombre": "x" }))),
        ("POST", format!("/api/clientes/{id}/desactivar"), None),
        ("POST", format!("/api/clientes/{id}/activar"), None),
    ] {
        let (status, e) = pedir(&app, &operacion, metodo, &uri, cuerpo).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{metodo} {uri}: {e}");
        assert_eq!(e["error"]["codigo"], "sin_permisos");
    }

    let (status, e) = pedir(&app, &gestion, "POST", "/api/clientes", Some(json!({
        "nombre": "Otro", "documento": documento
    }))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{e}");
    assert_eq!(e["error"]["codigo"], "documento_duplicado");
    assert_eq!(e["error"]["mensaje"], "Ya existe un cliente con ese documento.");
}

#[tokio::test]
async fn baja_logica_conserva_las_ventas() {
    let (app, pool, _correo) = app_de_test().await;
    let gestion = sesion_gestion(&pool).await;
    let (status, c) = pedir(&app, &gestion, "POST", "/api/clientes", Some(json!({ "nombre": "Cliente histórico" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{c}");
    let id = c["id"].as_i64().expect("id");
    let e = common::crear_transferencia_de_prueba(&pool, 3).await;
    let (status, v) = pedir(&app, &gestion, "POST", "/api/ventas", Some(json!({
        "deposito_id": e.origen_id, "cliente_id": id, "medio_pago": "EFECTIVO",
        "lineas": [{ "producto_id": e.producto_id, "cantidad": 1 }]
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");

    let (status, _) = pedir(&app, &gestion, "POST", &format!("/api/clientes/{id}/desactivar"), None).await;
    assert!(status.is_success());
    let (status, lista) = pedir(&app, &gestion, "GET", &format!("/api/ventas?cliente_id={id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{lista}");
    assert_eq!(lista["total"], 1);
    assert_eq!(lista["items"][0]["cliente_nombre"], "Cliente histórico");

    let (status, body) = pedir(&app, &gestion, "POST", &format!("/api/clientes/{id}/activar"), None).await;
    assert!(status.is_success(), "{body}");
}
