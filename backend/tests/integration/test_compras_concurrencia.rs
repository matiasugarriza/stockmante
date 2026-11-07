//! 012 (T026, SC-021) — anular una compra en paralelo con una SALIDA que consume
//! el mismo stock: una gana, la otra recibe 409 `stock_insuficiente`, nunca 500
//! ni saldo negativo.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{app_de_test, crear_transferencia_de_prueba, cuerpo_json, sesion_gestion, solicitud, sku_unico};
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

#[tokio::test]
async fn anulacion_y_salida_concurrentes_no_dejan_saldo_negativo() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let (status, prov) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/proveedores",
        Some(json!({ "nombre": format!("Proveedor {}", sku_unico("conc")) })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{prov}");
    let proveedor = prov["id"].as_i64().expect("id");

    for ronda in 0..6 {
        // Depósito y producto nuevos por ronda, sin stock previo.
        let e = crear_transferencia_de_prueba(&pool, 0).await;
        let (status, compra) = pedir(
            &app,
            &cookie,
            "POST",
            "/api/compras",
            Some(json!({
                "proveedor_id": proveedor,
                "deposito_id": e.origen_id,
                "lineas": [{ "producto_id": e.producto_id, "cantidad": 10, "costo_unitario": 5.0 }]
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{compra}");
        let id = compra["compra"]["id"].as_i64().or(compra["id"].as_i64()).expect("id de compra");
        let (status, body) =
            pedir(&app, &cookie, "POST", &format!("/api/compras/{id}/confirmar"), None).await;
        assert_eq!(status, StatusCode::OK, "{body}");

        let uri_anular = format!("/api/compras/{id}/anular");
        let anular = pedir(&app, &cookie, "POST", &uri_anular, None);
        let salida = pedir(
            &app,
            &cookie,
            "POST",
            "/api/movimientos",
            Some(json!({
                "tipo": "SALIDA",
                "deposito_id": e.origen_id,
                "motivo": "VENTA",
                "lineas": [{ "producto_id": e.producto_id, "ubicacion_id": null, "cantidad_delta": -5 }]
            })),
        );
        let ((s_anular, b_anular), (s_salida, b_salida)) = tokio::join!(anular, salida);

        assert_ne!(s_anular, StatusCode::INTERNAL_SERVER_ERROR, "ronda {ronda}: {b_anular}");
        assert_ne!(s_salida, StatusCode::INTERNAL_SERVER_ERROR, "ronda {ronda}: {b_salida}");
        let exitos = [s_anular == StatusCode::OK, s_salida == StatusCode::CREATED]
            .iter()
            .filter(|ok| **ok)
            .count();
        assert_eq!(exitos, 1, "ronda {ronda}: una sola gana ({s_anular} / {s_salida})");
        let perdedor = if s_anular == StatusCode::OK { &b_salida } else { &b_anular };
        assert_eq!(perdedor["error"]["codigo"], "stock_insuficiente", "ronda {ronda}: {perdedor}");

        let saldo: i32 = sqlx::query_scalar(
            "SELECT cantidad FROM stock_por_deposito WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL",
        )
        .bind(e.producto_id)
        .bind(e.origen_id)
        .fetch_one(&pool)
        .await
        .expect("saldo");
        assert!(saldo >= 0, "ronda {ronda}: saldo {saldo}");
        assert_eq!(saldo, if s_anular == StatusCode::OK { 0 } else { 5 });
    }
}
