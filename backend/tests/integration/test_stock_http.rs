//! 010 (T004, T018) — consulta de stock: valorizado por perfil, faltante,
//! `producto_ids` y detalle.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, crear_transferencia_de_prueba, cuerpo_json, sesion_gestion, sesion_operacion,
    solicitud,
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

async fn pedir(app: &Router, cookie: &str, uri: &str) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud("GET", uri, cookie, None))
        .await
        .expect("request de prueba");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

async fn definir_minimo(pool: &PgPool, producto: i32, minimo: i32, costo: &str) {
    sqlx::query("UPDATE productos SET stock_minimo = $2, costo = $3::numeric WHERE id = $1")
        .bind(producto)
        .bind(minimo)
        .bind(costo)
        .execute(pool)
        .await
        .expect("mínimo y costo");
}

fn item(body: &Value, producto: i32) -> Value {
    body["items"]
        .as_array()
        .and_then(|items| items.iter().find(|i| i["producto_id"] == producto).cloned())
        .unwrap_or(Value::Null)
}

#[tokio::test]
async fn valorizado_solo_para_gestion_y_faltante_para_todos() {
    let (app, pool, _correo) = app_de_test().await;
    let gestion = sesion_gestion(&pool).await;
    let operacion = sesion_operacion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 3).await;
    definir_minimo(&pool, e.producto_id, 10, "150.50").await;
    let uri = format!(
        "/api/stock?producto_ids={}&deposito_id={}",
        e.producto_id, e.origen_id
    );

    let (status, body) = pedir(&app, &gestion, &uri).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let g = item(&body, e.producto_id);
    assert_eq!(g["faltante"], 7);
    assert_eq!(g["stock_deposito"], 3);
    assert_eq!(g["costo"].as_f64(), Some(150.5));
    assert_eq!(g["valorizado"].as_f64(), Some(451.5));

    let (status, body) = pedir(&app, &operacion, &uri).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let o = item(&body, e.producto_id);
    assert_eq!(o["faltante"], 7);
    assert_eq!(o["costo"], Value::Null);
    assert_eq!(o["valorizado"], Value::Null);

    // 10.4: bajo mínimo trae faltante.
    let (status, body) = pedir(
        &app,
        &gestion,
        &format!("/api/stock?producto_ids={}&bajo_minimo=true", e.producto_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(item(&body, e.producto_id)["faltante"], 7);
}

#[tokio::test]
async fn producto_ids_filtra_y_valida() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let a = crear_transferencia_de_prueba(&pool, 2).await;
    let b = crear_transferencia_de_prueba(&pool, 5).await;

    let (status, body) = pedir(
        &app,
        &cookie,
        &format!("/api/stock?producto_ids={},{}&deposito_id={}", a.producto_id, b.producto_id, b.origen_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total"], 2);
    assert_eq!(item(&body, a.producto_id)["stock_deposito"], 0);
    assert_eq!(item(&body, b.producto_id)["stock_deposito"], 5);

    let muchos: Vec<String> = (1..=51).map(|i| i.to_string()).collect();
    let (status, body) =
        pedir(&app, &cookie, &format!("/api/stock?producto_ids={}", muchos.join(","))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["mensaje"], "producto_ids admite hasta 50 ids.");

    let (status, body) = pedir(&app, &cookie, "/api/stock?producto_ids=1,abc").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(
        body["error"]["mensaje"],
        "producto_ids debe ser una lista de enteros mayores a 0 separados por coma."
    );
}

#[tokio::test]
async fn detalle_trae_faltante_y_desglose_que_suma_el_cache() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 4).await;
    definir_minimo(&pool, e.producto_id, 6, "10").await;

    let (status, body) = pedir(&app, &cookie, &format!("/api/stock/{}", e.producto_id)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["faltante"], 2);
    let suma: i64 = body["desglose"]
        .as_array()
        .map(|d| d.iter().filter_map(|f| f["cantidad"].as_i64()).sum())
        .unwrap_or(-1);
    assert_eq!(Some(suma), body["stock_actual"].as_i64());

    let (status, body) = pedir(&app, &cookie, "/api/stock/999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["mensaje"], "Producto no encontrado.");
}

/// T018 (SC-018): medición con 10.000 productos. Se corre a mano:
/// `cargo test --test test_stock_http sc018 -- --ignored --nocapture`.
#[tokio::test]
#[ignore]
async fn sc018() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase();
    let categoria: i32 = sqlx::query_scalar("INSERT INTO categorias (nombre) VALUES ($1) RETURNING id")
        .bind(format!("SC018 {marca}"))
        .fetch_one(&pool)
        .await
        .expect("categoría");
    sqlx::query(
        r#"INSERT INTO productos (nombre, sku, categoria_id, unidad_medida, costo, precio_venta)
           SELECT 'SC018 producto ' || g || ' ' || $1, 'S18' || $1 || '-' || g, $2, 'UNIDAD', 10, 20
             FROM generate_series(1, 10000) g"#,
    )
    .bind(&marca)
    .bind(categoria)
    .execute(&pool)
    .await
    .expect("10.000 productos");
    let mut depositos = Vec::new();
    for i in 0..3 {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO depositos (nombre, identificador) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!("SC018 {marca} {i}"))
        .bind(format!("S18-{marca}-{i}"))
        .fetch_one(&pool)
        .await
        .expect("depósito");
        depositos.push(id);
    }
    for (i, deposito) in depositos.iter().enumerate() {
        sqlx::query(
            r#"INSERT INTO stock_por_deposito (producto_id, deposito_id, ubicacion_id, cantidad)
               SELECT id, $1, NULL, $3 FROM productos WHERE categoria_id = $2"#,
        )
        .bind(deposito)
        .bind(categoria)
        .bind(i as i32 + 1)
        .execute(&pool)
        .await
        .expect("stock");
    }
    for uri in [
        format!("/api/stock?deposito_id={}", depositos[0]),
        format!("/api/stock?deposito_id={}&q=producto%20777", depositos[0]),
        format!("/api/stock?deposito_id={}&q=S18{marca}-777", depositos[0]),
    ] {
        let inicio = std::time::Instant::now();
        let (status, _) = pedir(&app, &cookie, &uri).await;
        println!("SC-018 {uri}: {:?} ({status})", inicio.elapsed());
    }

    // Limpieza: la BD de test es compartida y el resto de los tests pagina sobre ella.
    for sql in [
        "DELETE FROM stock_por_deposito WHERE deposito_id = ANY($1)",
        "DELETE FROM depositos WHERE id = ANY($1)",
    ] {
        sqlx::query(sql).bind(&depositos).execute(&pool).await.expect("limpieza");
    }
    sqlx::query("DELETE FROM productos WHERE categoria_id = $1")
        .bind(categoria)
        .execute(&pool)
        .await
        .expect("limpieza de productos");
    sqlx::query("DELETE FROM categorias WHERE id = $1")
        .bind(categoria)
        .execute(&pool)
        .await
        .expect("limpieza de categoría");
}

/// 010 (R1): la caja necesita los depósitos activos; cualquier sesión los lee
/// en forma breve (`id`, `nombre`), sin datos de gestión.
#[tokio::test]
async fn depositos_activos_para_cualquier_sesion() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_operacion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 1).await;
    sqlx::query("UPDATE depositos SET activo = FALSE WHERE id = $1")
        .bind(e.destino_id)
        .execute(&pool)
        .await
        .expect("desactivar destino");

    let (status, body) = pedir(&app, &cookie, "/api/depositos/activos").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let lista = body.as_array().expect("lista");
    let origen = lista.iter().find(|d| d["id"] == e.origen_id).expect("origen activo");
    assert!(origen["nombre"].is_string());
    assert!(origen.get("identificador").is_none(), "solo id y nombre: {origen}");
    assert!(!lista.iter().any(|d| d["id"] == e.destino_id), "no lista inactivos");

    let anonimo = app
        .clone()
        .oneshot(common::solicitud_anonima("GET", "/api/depositos/activos"))
        .await
        .expect("request");
    assert_eq!(anonimo.status(), StatusCode::UNAUTHORIZED);
}
