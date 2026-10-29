//! 004/T006 — integración HTTP de `GET /api/productos/buscar`.
//!
//! Cubre exactitud, orden, filtro por categoría, inactivos y RBAC.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use common::{
    app_de_test, codigo_barras_unico, cuerpo_json, nombre_categoria_unico, sesion_gestion,
    sesion_operacion, sku_unico, solicitud, solicitud_anonima,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn crear_categoria(app: &axum::Router, cookie: &str, nombre: &str) -> i32 {
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/categorias",
            cookie,
            Some(json!({ "nombre": nombre })),
        ))
        .await
        .expect("alta de categoría");
    assert_eq!(res.status(), StatusCode::CREATED, "categoría {nombre}");
    cuerpo_json(res).await["id"]
        .as_i64()
        .expect("id de categoría") as i32
}

async fn crear_producto(
    app: &axum::Router,
    cookie: &str,
    nombre: &str,
    sku: &str,
    codigo: Option<&str>,
    categoria_id: i32,
) -> i32 {
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/productos",
            cookie,
            Some(json!({
                "nombre": nombre,
                "sku": sku,
                "codigo_barras": codigo,
                "categoria_id": categoria_id,
                "unidad_medida": "UNIDAD",
                "costo": 100,
                "precio_venta": 150,
                "stock_minimo": 1,
            })),
        ))
        .await
        .expect("alta de producto");
    let status = res.status();
    let body = cuerpo_json(res).await;
    assert_eq!(status, StatusCode::CREATED, "producto {nombre}: {body}");
    body["id"].as_i64().expect("id de producto") as i32
}

async fn buscar(app: &axum::Router, cookie: &str, query: &str) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos/buscar?{query}"),
            cookie,
            None,
        ))
        .await
        .expect("búsqueda");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

fn ids(body: &Value) -> Vec<i32> {
    body["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["id"].as_i64().expect("id") as i32)
        .collect()
}

fn nombre_de(body: &Value, id: i32) -> String {
    body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["id"].as_i64() == Some(id as i64))
        .and_then(|item| item["nombre"].as_str())
        .expect("producto en el resultado")
        .to_string()
}

#[tokio::test]
async fn sku_y_codigo_exactos_encuentran_el_producto() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie, &nombre_categoria_unico("Busqueda")).await;
    let sku = sku_unico("exacto");
    let codigo = codigo_barras_unico();
    let id = crear_producto(
        &app,
        &cookie,
        "Producto exacto de prueba",
        &sku,
        Some(&codigo),
        categoria,
    )
    .await;

    let (status, body) = buscar(&app, &cookie, &format!("q={sku}&campo=sku")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![id]);

    // El SKU se guarda en mayúsculas: la caja puede tipear minúsculas.
    let (status, body) = buscar(
        &app,
        &cookie,
        &format!("q={}&campo=sku", sku.to_lowercase()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![id]);

    let (status, body) = buscar(&app, &cookie, &format!("q={codigo}&campo=codigo")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![id]);
}

#[tokio::test]
async fn nombre_ordena_por_similitud_y_limita_resultados() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie, &nombre_categoria_unico("Orden")).await;
    let sufijo = uuid::Uuid::new_v4().simple().to_string();
    let fragmento = &sufijo[..8];
    let nombre_exacto = format!("Yerba {fragmento} Mate");
    let nombre_largo = format!("Yerba {fragmento} Mate Compuesta Extra");
    let esperado = crear_producto(
        &app,
        &cookie,
        &nombre_exacto,
        &sku_unico("orden-a"),
        None,
        categoria,
    )
    .await;
    let otro = crear_producto(
        &app,
        &cookie,
        &nombre_largo,
        &sku_unico("orden-b"),
        None,
        categoria,
    )
    .await;

    let consulta = format!("q=yerba+{fragmento}+mate&campo=nombre");
    let (status, body) = buscar(&app, &cookie, &consulta).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let encontrados = ids(&body);
    assert!(encontrados.contains(&esperado), "falta el exacto: {body}");
    assert!(encontrados.contains(&otro), "falta el compuesto: {body}");
    assert_eq!(
        encontrados.first().copied(),
        Some(esperado),
        "el nombre más parecido va primero: {}",
        nombre_de(&body, encontrados[0])
    );

    let (status, body) = buscar(
        &app,
        &cookie,
        &format!("{consulta}&limite=1"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![esperado]);
}

#[tokio::test]
async fn categoria_filtra_y_permite_listar_sin_texto() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let sufijo = uuid::Uuid::new_v4().simple().to_string();
    let fragmento = sufijo[..8].to_string();
    let categoria = crear_categoria(
        &app,
        &cookie,
        &nombre_categoria_unico(&format!("Busqueda {fragmento}")),
    )
    .await;
    let otra = crear_categoria(&app, &cookie, &nombre_categoria_unico("Otra categoria")).await;
    let esperado = crear_producto(
        &app,
        &cookie,
        "Producto de la categoría buscada",
        &sku_unico("cat-a"),
        None,
        categoria,
    )
    .await;
    crear_producto(
        &app,
        &cookie,
        "Producto de otra categoría",
        &sku_unico("cat-b"),
        None,
        otra,
    )
    .await;

    // Sin texto: lista la categoría elegida (escenario 4).
    let (status, body) = buscar(&app, &cookie, &format!("categoria_id={categoria}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![esperado]);

    // Con un fragmento único del nombre de la categoría.
    let (status, body) = buscar(
        &app,
        &cookie,
        &format!("q={fragmento}&campo=categoria&categoria_id={categoria}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![esperado]);

    let (status, body) = buscar(
        &app,
        &cookie,
        &format!("q=zzzz&campo=categoria&categoria_id={categoria}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(ids(&body).is_empty(), "el término debe filtrar: {body}");
}

#[tokio::test]
async fn inactivos_quedan_fuera_salvo_gestion_explicita() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;
    let categoria = crear_categoria(&app, &cookie, &nombre_categoria_unico("Inactivos")).await;
    let sku = sku_unico("inactivo");
    let id = crear_producto(
        &app,
        &cookie,
        "Producto que se dará de baja",
        &sku,
        None,
        categoria,
    )
    .await;

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/productos/{id}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let (status, body) = buscar(&app, &cookie, &format!("q={sku}&campo=sku")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(ids(&body).is_empty(), "un inactivo no sale en caja: {body}");

    let (status, body) = buscar(
        &app,
        &cookie,
        &format!("q={sku}&campo=sku&incluir_inactivos=true"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), vec![id]);

    let (status, body) = buscar(
        &app,
        &operario,
        &format!("q={sku}&campo=sku&incluir_inactivos=true"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["codigo"], "sin_permisos");
}

#[tokio::test]
async fn parametros_invalidos_son_422_con_mensaje() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;

    for (query, esperado) in [
        ("q=a&campo=nombre", "al menos 2 caracteres"),
        ("q=a&campo=categoria", "al menos 2 caracteres"),
        ("q=abc&campo=precio", "campo de búsqueda"),
        ("q=abc&limite=0", "límite"),
        ("q=abc&limite=51", "límite"),
        ("q=abc&categoria_id=0", "categoria_id"),
        ("campo=sku", "término de búsqueda"),
    ] {
        let (status, body) = buscar(&app, &cookie, query).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{query}: {body}");
        assert_eq!(body["error"]["codigo"], "validacion", "{query}: {body}");
        let mensaje = body["error"]["mensaje"].as_str().unwrap_or_default();
        assert!(
            mensaje.contains(esperado),
            "{query}: se esperaba «{esperado}» en «{mensaje}»"
        );
    }

    let (status, body) = buscar(&app, &cookie, &format!("q={}&campo=sku", "A".repeat(161))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["error"]["mensaje"]
        .as_str()
        .unwrap_or_default()
        .contains("160"));
}

#[tokio::test]
async fn sin_sesion_es_401() {
    let (app, _pool, _correo) = app_de_test().await;
    let res = app
        .oneshot(solicitud_anonima(
            "GET",
            "/api/productos/buscar?q=yerba",
        ))
        .await
        .expect("búsqueda anónima");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
