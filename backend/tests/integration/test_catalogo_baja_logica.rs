//! 003/T018 — CU-RET-01: el retiro del catálogo es lógico, nunca físico.
//!
//! La suite corre en paralelo contra `stockmante_test`, así que los conteos se
//! hacen sobre una marca única por corrida y no sobre la tabla completa.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::{header, StatusCode};
use common::{
    app_de_test, cuerpo_json, nombre_categoria_unico, sesion_gestion, sku_unico, solicitud,
};
use serde_json::json;
use tower::ServiceExt;

async fn contar(pool: &sqlx::PgPool, marca: &str) -> (i64, i64) {
    let productos: i64 = sqlx::query_scalar("SELECT count(*) FROM productos WHERE nombre LIKE $1")
        .bind(format!("Baja {marca}%"))
        .fetch_one(pool)
        .await
        .expect("conteo de productos");
    let categorias: i64 =
        sqlx::query_scalar("SELECT count(*) FROM categorias WHERE nombre LIKE $1")
            .bind(format!("Categoría baja {marca}%"))
            .fetch_one(pool)
            .await
            .expect("conteo de categorías");
    (productos, categorias)
}

#[tokio::test]
async fn el_retiro_es_logico_y_no_deja_huecos() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let marca = uuid::Uuid::new_v4().simple().to_string();
    let nombre_categoria = format!("Categoría baja {marca}");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/categorias",
            &cookie,
            Some(json!({ "nombre": nombre_categoria })),
        ))
        .await
        .expect("alta de categoría");
    assert_eq!(res.status(), StatusCode::CREATED);
    let id_categoria = cuerpo_json(res).await["id"]
        .as_i64()
        .expect("id de categoría") as i32;

    let mut ids = Vec::new();
    for indice in 0..2 {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                "/api/productos",
                &cookie,
                Some(json!({
                    "nombre": format!("Baja {marca} {indice}"),
                    "sku": sku_unico("baja"),
                    "categoria_id": id_categoria,
                    "unidad_medida": "UNIDAD",
                    "costo": 10,
                    "precio_venta": 20,
                    "stock_minimo": 0,
                })),
            ))
            .await
            .expect("alta de producto");
        assert_eq!(res.status(), StatusCode::CREATED);
        ids.push(cuerpo_json(res).await["id"].as_i64().expect("id") as i32);
    }

    let antes = contar(&pool, &marca).await;
    assert_eq!(antes, (2, 1));

    // 25 vueltas de baja + reactivación por producto: 50 bajas lógicas por producto.
    for _ in 0..25 {
        for id in &ids {
            let res = app
                .clone()
                .oneshot(solicitud(
                    "POST",
                    &format!("/api/productos/{id}/desactivar"),
                    &cookie,
                    None,
                ))
                .await
                .expect("desactivar producto");
            assert_eq!(res.status(), StatusCode::NO_CONTENT);
        }
        for id in &ids {
            let res = app
                .clone()
                .oneshot(solicitud(
                    "POST",
                    &format!("/api/productos/{id}/reactivar"),
                    &cookie,
                    None,
                ))
                .await
                .expect("reactivar producto");
            assert_eq!(res.status(), StatusCode::NO_CONTENT);
        }
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/categorias/{id_categoria}/desactivar"),
                &cookie,
                None,
            ))
            .await
            .expect("desactivar categoría");
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/categorias/{id_categoria}/reactivar"),
                &cookie,
                None,
            ))
            .await
            .expect("reactivar categoría");
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    // Ninguna fila desapareció: las 50+ bajas solo cambiaron `activo`.
    assert_eq!(contar(&pool, &marca).await, antes);

    // Los identificadores originales siguen siendo los mismos (sin huecos nuevos).
    for (indice, id) in ids.iter().enumerate() {
        let (nombre, activo): (String, bool) =
            sqlx::query_as("SELECT nombre, activo FROM productos WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .expect("producto");
        assert_eq!(nombre, format!("Baja {marca} {indice}"));
        assert!(activo);
    }
    let (nombre_categoria_db, activo): (String, bool) =
        sqlx::query_as("SELECT nombre, activo FROM categorias WHERE id = $1")
            .bind(id_categoria)
            .fetch_one(&pool)
            .await
            .expect("categoría");
    assert_eq!(nombre_categoria_db, nombre_categoria);
    assert!(activo);

    // `?activo=all` devuelve los inactivos con su id original.
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/productos/{}/desactivar", ids[0]),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar el primero");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?activo=all&q={marca}&por_pagina=100"),
            &cookie,
            None,
        ))
        .await
        .expect("listado completo");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    let items = body["items"].as_array().expect("items");
    assert!(items
        .iter()
        .any(|p| p["id"].as_i64() == Some(ids[0] as i64) && p["activo"] == false));
    assert!(items
        .iter()
        .any(|p| p["id"].as_i64() == Some(ids[1] as i64) && p["activo"] == true));

    // El detalle de un inactivo sigue respondiendo 200.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos/{}", ids[0]),
            &cookie,
            None,
        ))
        .await
        .expect("detalle inactivo");
    assert_eq!(res.status(), StatusCode::OK);

    // DELETE sigue sin estar registrado en ambos recursos.
    for uri in [
        format!("/api/productos/{}", ids[0]),
        format!("/api/categorias/{id_categoria}"),
    ] {
        let res = app
            .clone()
            .oneshot(solicitud("DELETE", &uri, &cookie, None))
            .await
            .expect("DELETE");
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED, "{uri}");
        let allow = res
            .headers()
            .get(header::ALLOW)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .replace(' ', "");
        assert!(!allow.contains("DELETE"), "{uri} → Allow: {allow}");
        assert!(cuerpo_json(res).await.is_null(), "{uri} no debe tener cuerpo");
    }

    // Las filas siguen existiendo después de los 405.
    assert_eq!(contar(&pool, &marca).await, antes);
}
