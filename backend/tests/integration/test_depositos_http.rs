//! 005 — integración HTTP de depósitos, ubicaciones y asignaciones.
//!
//! Cubre CRUD, unicidad, jerarquía (mismo depósito, ciclos, duplicados),
//! borrado con dependencias, asignación producto–ubicación y RBAC.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, cuerpo_json, nombre_categoria_unico, sesion_gestion, sesion_operacion, sku_unico,
    solicitud, solicitud_anonima,
};
use serde_json::{json, Value};
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

async fn crear_deposito(app: &Router, cookie: &str, nombre: &str, identificador: &str) -> Value {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/depositos",
        Some(json!({ "nombre": nombre, "identificador": identificador })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de depósito: {body}");
    body
}

async fn crear_ubicacion(
    app: &Router,
    cookie: &str,
    deposito_id: i32,
    padre_id: Option<i32>,
    nombre: &str,
    tipo: &str,
) -> (StatusCode, Value) {
    let mut cuerpo = json!({
        "deposito_id": deposito_id,
        "nombre": nombre,
        "tipo": tipo,
    });
    if let Some(padre) = padre_id {
        cuerpo["padre_id"] = json!(padre);
    }
    pedir(app, cookie, "POST", "/api/ubicaciones", Some(cuerpo)).await
}

async fn crear_producto(app: &Router, cookie: &str, categoria_id: i32) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/productos",
        Some(json!({
            "nombre": "Producto de depósitos",
            "sku": sku_unico("deposito"),
            "categoria_id": categoria_id,
            "unidad_medida": "UNIDAD",
            "costo": 100,
            "precio_venta": 150,
            "stock_minimo": 1,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de producto: {body}");
    body["id"].as_i64().expect("id de producto") as i32
}

async fn crear_categoria(app: &Router, cookie: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/categorias",
        Some(json!({ "nombre": nombre_categoria_unico("Depositos") })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta de categoría: {body}");
    body["id"].as_i64().expect("id de categoría") as i32
}

fn sufijo() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase()
}

#[tokio::test]
async fn crud_de_deposito_unicidad_y_borrado() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = sufijo();
    let nombre = format!("Depósito {marca}");
    let identificador = format!("DEP-{marca}");

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/depositos",
        Some(json!({
            "nombre": format!("  {nombre}  "),
            "identificador": identificador,
            "direccion": "  Calle 1 742 ",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["nombre"], nombre);
    assert_eq!(body["direccion"], "Calle 1 742");
    assert_eq!(body["activo"], true);
    let id = body["id"].as_i64().expect("id") as i32;

    let (status, lista) = pedir(&app, &cookie, "GET", "/api/depositos?activo=true", None).await;
    assert_eq!(status, StatusCode::OK, "{lista}");
    assert!(
        lista
            .as_array()
            .expect("lista")
            .iter()
            .any(|d| d["id"].as_i64() == Some(id as i64)),
        "el depósito recién creado debe aparecer: {lista}"
    );

    let (status, body) = pedir(&app, &cookie, "GET", &format!("/api/depositos/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["identificador"], identificador);

    // PUT parcial: el identificador ausente se conserva.
    let nuevo_nombre = format!("{nombre} II");
    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/depositos/{id}"),
        Some(json!({ "nombre": nuevo_nombre })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["nombre"], nuevo_nombre);
    assert_eq!(body["identificador"], identificador);

    // `direccion: ""` la limpia.
    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/depositos/{id}"),
        Some(json!({ "direccion": "" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["direccion"].is_null(), "{body}");

    for (cuerpo, esperado) in [
        (json!({ "activo": true }), "desactivar"),
        (json!({}), "al menos un campo"),
        (json!({ "nombre": "   " }), "obligatorio"),
        (json!({ "identificador": "X".repeat(41) }), "40"),
    ] {
        let (status, body) = pedir(
            &app,
            &cookie,
            "PUT",
            &format!("/api/depositos/{id}"),
            Some(cuerpo),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        let mensaje = body["error"]["mensaje"].as_str().unwrap_or_default();
        assert!(mensaje.contains(esperado), "{body}");
    }

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/depositos",
        Some(json!({ "nombre": nuevo_nombre, "identificador": format!("OTRO-{marca}") })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "deposito_nombre_duplicado");

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/depositos",
        Some(json!({ "nombre": format!("Otro {marca}"), "identificador": identificador })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "deposito_identificador_duplicado");

    let (status, body) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/depositos",
        Some(json!({ "nombre": "   ", "identificador": format!("VACIO-{marca}") })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = pedir(&app, &cookie, "GET", &format!("/api/depositos/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[tokio::test]
async fn borrado_de_deposito_con_ubicaciones_es_409() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = sufijo();
    let deposito = crear_deposito(
        &app,
        &cookie,
        &format!("Con dependencias {marca}"),
        &format!("DEP-DEP-{marca}"),
    )
    .await;
    let deposito_id = deposito["id"].as_i64().expect("id") as i32;

    let (status, ubicacion) =
        crear_ubicacion(&app, &cookie, deposito_id, None, "Pasillo 1", "PASILLO").await;
    assert_eq!(status, StatusCode::CREATED, "{ubicacion}");
    let ubicacion_id = ubicacion["id"].as_i64().expect("id ubicación") as i32;

    let (status, body) =
        pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{deposito_id}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "deposito_con_dependencias");

    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/ubicaciones/{ubicacion_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) =
        pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{deposito_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

#[tokio::test]
async fn ubicaciones_jerarquia_duplicados_y_ciclos() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = sufijo();
    let a = crear_deposito(
        &app,
        &cookie,
        &format!("Depósito A {marca}"),
        &format!("DEP-A-{marca}"),
    )
    .await["id"]
        .as_i64()
        .expect("id A") as i32;
    let b = crear_deposito(
        &app,
        &cookie,
        &format!("Depósito B {marca}"),
        &format!("DEP-B-{marca}"),
    )
    .await["id"]
        .as_i64()
        .expect("id B") as i32;

    let (status, raiz) = crear_ubicacion(&app, &cookie, a, None, "Pasillo 1", "PASILLO").await;
    assert_eq!(status, StatusCode::CREATED, "{raiz}");
    assert!(raiz["padre_id"].is_null(), "{raiz}");
    let raiz_id = raiz["id"].as_i64().expect("id raíz") as i32;

    let (status, hija) = crear_ubicacion(
        &app,
        &cookie,
        a,
        Some(raiz_id),
        "Estantería 1",
        "estanteria",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{hija}");
    assert_eq!(hija["tipo"], "ESTANTERIA");
    assert_eq!(hija["padre_id"].as_i64(), Some(raiz_id as i64));
    let hija_id = hija["id"].as_i64().expect("id hija") as i32;

    let (status, body) =
        crear_ubicacion(&app, &cookie, a, Some(raiz_id), "Estantería 1", "NIVEL").await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "ubicacion_duplicada");

    let (status, body) =
        crear_ubicacion(&app, &cookie, b, Some(raiz_id), "Prestada", "NIVEL").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "ubicacion_huerfana");

    let (status, body) =
        crear_ubicacion(&app, &cookie, a, Some(999_999), "Huérfana", "NIVEL").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["codigo"], "ubicacion_huerfana");

    let (status, body) = crear_ubicacion(&app, &cookie, a, None, "Tipo raro", "DEPOSITO").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["error"]["mensaje"]
        .as_str()
        .unwrap_or_default()
        .contains("PASILLO"));

    let (status, lista) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/depositos/{a}/ubicaciones"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{lista}");
    assert_eq!(
        lista["items"].as_array().expect("items").len(),
        2,
        "{lista}"
    );

    // Ciclo: la raíz no puede colgar de su propia hija.
    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/ubicaciones/{raiz_id}"),
        Some(json!({ "padre_id": hija_id })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["error"]["mensaje"]
        .as_str()
        .unwrap_or_default()
        .contains("dentro de sí misma"));

    // Mover a la raíz del árbol con `padre_id: null`.
    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/ubicaciones/{hija_id}"),
        Some(json!({ "padre_id": null })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["padre_id"].is_null(), "{body}");

    // Cambiar de depósito una hoja es válido y la vuelve raíz.
    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/ubicaciones/{hija_id}"),
        Some(json!({ "deposito_id": b })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deposito_id"].as_i64(), Some(b as i64));
    assert!(body["padre_id"].is_null(), "{body}");

    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/ubicaciones/{raiz_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{b}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");

    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/ubicaciones/{hija_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{b}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

#[tokio::test]
async fn mover_ubicacion_de_deposito_exige_no_tener_dependencias() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = sufijo();
    let a = crear_deposito(
        &app,
        &cookie,
        &format!("Mover A {marca}"),
        &format!("MOV-A-{marca}"),
    )
    .await["id"]
        .as_i64()
        .expect("id A") as i32;
    let b = crear_deposito(
        &app,
        &cookie,
        &format!("Mover B {marca}"),
        &format!("MOV-B-{marca}"),
    )
    .await["id"]
        .as_i64()
        .expect("id B") as i32;

    let raiz_id = crear_ubicacion(&app, &cookie, a, None, "Nivel 1", "NIVEL")
        .await
        .1["id"]
        .as_i64()
        .expect("id raíz") as i32;
    let hija_id = crear_ubicacion(&app, &cookie, a, Some(raiz_id), "Posición 1", "POSICION")
        .await
        .1["id"]
        .as_i64()
        .expect("id hija") as i32;

    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/ubicaciones/{raiz_id}"),
        Some(json!({ "deposito_id": b })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["codigo"], "ubicacion_con_dependencias");

    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/ubicaciones/{hija_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/ubicaciones/{raiz_id}"),
        Some(json!({ "deposito_id": b })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deposito_id"].as_i64(), Some(b as i64));

    let (status, body) = pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{a}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (status, body) = pedir(&app, &cookie, "DELETE", &format!("/api/depositos/{b}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    let (status, _) = pedir(&app, &cookie, "DELETE", &format!("/api/ubicaciones/{raiz_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn asignacion_de_ubicaciones_a_producto() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = sufijo();
    let categoria = crear_categoria(&app, &cookie).await;
    let producto = crear_producto(&app, &cookie, categoria).await;

    let deposito = crear_deposito(
        &app,
        &cookie,
        &format!("Asignaciones {marca}"),
        &format!("ASIG-{marca}"),
    )
    .await["id"]
        .as_i64()
        .expect("id depósito") as i32;
    let u1 = crear_ubicacion(&app, &cookie, deposito, None, "Estantería A", "ESTANTERIA")
        .await
        .1["id"]
        .as_i64()
        .expect("u1") as i32;
    let u2 = crear_ubicacion(&app, &cookie, deposito, None, "Estantería B", "ESTANTERIA")
        .await
        .1["id"]
        .as_i64()
        .expect("u2") as i32;

    let ruta = format!("/api/productos/{producto}/ubicaciones");
    let (status, body) =
        pedir(&app, &cookie, "PUT", &ruta, Some(json!({ "ubicacion_ids": [u1, u2] }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    let (status, body) = pedir(&app, &cookie, "GET", &ruta, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let ids: Vec<i64> = body["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|u| u["id"].as_i64().expect("id ubicación"))
        .collect();
    assert_eq!(ids.len(), 2, "{body}");
    assert!(ids.contains(&(u1 as i64)) && ids.contains(&(u2 as i64)));

    // Duplicados en la entrada se deduplican.
    let (status, body) =
        pedir(&app, &cookie, "PUT", &ruta, Some(json!({ "ubicacion_ids": [u1, u1] }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (status, body) = pedir(&app, &cookie, "GET", &ruta, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1, "{body}");

    // Conjunto vacío limpia.
    let (status, _) = pedir(&app, &cookie, "PUT", &ruta, Some(json!({ "ubicacion_ids": [] }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = pedir(&app, &cookie, "GET", &ruta, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["items"].as_array().expect("items").is_empty(), "{body}");

    // Ubicación inexistente o inactiva no se acepta.
    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &ruta,
        Some(json!({ "ubicacion_ids": [999_999] })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/ubicaciones/{u2}/desactivar"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) =
        pedir(&app, &cookie, "PUT", &ruta, Some(json!({ "ubicacion_ids": [u2] }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let (status, body) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/productos/999999/ubicaciones"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/ubicaciones/{u2}/reactivar"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rbac_de_depositos_ubicaciones_y_asignaciones() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;

    for (metodo, uri, body) in [
        ("GET", "/api/depositos".to_string(), None),
        (
            "POST",
            "/api/depositos".to_string(),
            Some(json!({ "nombre": "No permitido", "identificador": "NO-PERM" })),
        ),
        ("GET", "/api/ubicaciones/1".to_string(), None),
        (
            "POST",
            "/api/ubicaciones".to_string(),
            Some(json!({
                "deposito_id": 1,
                "nombre": "No permitida",
                "tipo": "PASILLO",
            })),
        ),
        (
            "PUT",
            "/api/productos/1/ubicaciones".to_string(),
            Some(json!({ "ubicacion_ids": [] })),
        ),
    ] {
        let (status, respuesta) = pedir(&app, &cookie, metodo, &uri, body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{metodo} {uri}: {respuesta}");
        assert_eq!(respuesta["error"]["codigo"], "sin_permisos", "{uri}");
    }

    for uri in [
        "/api/depositos",
        "/api/depositos/1/ubicaciones",
        "/api/ubicaciones/1",
    ] {
        let res = app
            .clone()
            .oneshot(solicitud_anonima("GET", uri))
            .await
            .expect("request anónima");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
}
