//! 014 (T005) — reportes por HTTP: kardex (SC-022), stock valorizado, rotación, ventas,
//! CSV y 422. Cada test fecha sus datos en un día propio y lejano para no mezclarse con
//! el resto de la BD de test compartida.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, crear_transferencia_de_prueba, cuerpo_json, sesion_gestion, solicitud,
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

async fn pedir_texto(app: &Router, cookie: &str, uri: &str) -> (StatusCode, String, String, String) {
    let res = app
        .clone()
        .oneshot(solicitud("GET", uri, cookie, None))
        .await
        .expect("request de prueba");
    let status = res.status();
    let tipo = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let disposicion = res
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("cuerpo");
    (status, tipo, disposicion, String::from_utf8_lossy(&bytes).to_string())
}

/// Año único por test (2100–2899) para aislar los rangos de fecha.
fn anio_propio() -> i32 {
    let n = u32::from_str_radix(&uuid::Uuid::new_v4().simple().to_string()[..6], 16).unwrap_or(0);
    2100 + (n % 800) as i32
}

async fn movimiento(app: &Router, cookie: &str, tipo: &str, motivo: &str, deposito: i32, producto: i32, delta: i32) -> i32 {
    let (status, body) = pedir(app, cookie, "POST", "/api/movimientos", Some(json!({
        "tipo": tipo, "deposito_id": deposito, "motivo": motivo, "observacion": "reporte",
        "lineas": [{ "producto_id": producto, "ubicacion_id": null, "cantidad_delta": delta }]
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{tipo}: {body}");
    body["id"].as_i64().expect("id") as i32
}

async fn fechar(pool: &PgPool, movimiento: i32, cuando: &str) {
    sqlx::query("UPDATE movimientos SET creado_en = $2::timestamptz WHERE id = $1")
        .bind(movimiento)
        .bind(cuando)
        .execute(pool)
        .await
        .expect("fechar movimiento");
}

/// 14.1 + SC-022: saldo inicial, orden, saldo corrido y saldo final contra el stock real.
#[tokio::test]
async fn kardex_con_saldos_y_ultimo_dia_incluido() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let a = anio_propio();
    let e = crear_transferencia_de_prueba(&pool, 0).await;

    let entrada_previa = movimiento(&app, &cookie, "ENTRADA", "COMPRA", e.origen_id, e.producto_id, 20).await;
    fechar(&pool, entrada_previa, &format!("{a}-09-15T12:00:00Z")).await;
    let salida = movimiento(&app, &cookie, "SALIDA", "VENTA", e.origen_id, e.producto_id, -4).await;
    fechar(&pool, salida, &format!("{a}-10-05T12:00:00Z")).await;
    let merma = movimiento(&app, &cookie, "MERMA", "MERMA", e.origen_id, e.producto_id, -1).await;
    fechar(&pool, merma, &format!("{a}-10-10T12:00:00Z")).await;
    let ajuste = movimiento(&app, &cookie, "AJUSTE", "AJUSTE", e.origen_id, e.producto_id, 2).await;
    // 23:30 UTC del último día = 20:30 local: debe entrar.
    fechar(&pool, ajuste, &format!("{a}-10-31T23:30:00Z")).await;

    let (status, t) = pedir(&app, &cookie, "POST", "/api/transferencias", Some(json!({
        "producto_id": e.producto_id, "deposito_origen_id": e.origen_id,
        "deposito_destino_id": e.destino_id, "cantidad": 5
    }))).await;
    assert_eq!(status, StatusCode::CREATED, "{t}");
    for campo in ["movimiento_salida_id", "movimiento_entrada_id"] {
        fechar(&pool, t[campo].as_i64().expect("id") as i32, &format!("{a}-10-20T12:00:00Z")).await;
    }

    let (status, k) = pedir(
        &app,
        &cookie,
        "GET",
        &format!("/api/reportes/kardex?producto_id={}&desde={a}-10-01&hasta={a}-10-31", e.producto_id),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{k}");
    assert_eq!(k["saldo_inicial"], 20);
    let saldos: Vec<i64> = k["lineas"].as_array().expect("lineas").iter().filter_map(|l| l["saldo"].as_i64()).collect();
    assert_eq!(saldos, vec![16, 15, 10, 15, 17], "{k}");
    assert_eq!(k["saldo_final"], 17);
    assert_eq!(k["desde"], format!("{a}-10-01"));
    assert_eq!(k["hasta"], format!("{a}-10-31"));

    // Sin filtros: saldo final = stock_actual. Con depósito: = stock_por_deposito.
    let (_, total) = pedir(&app, &cookie, "GET", &format!("/api/reportes/kardex?producto_id={}", e.producto_id), None).await;
    let cache: i32 = sqlx::query_scalar("SELECT stock_actual FROM productos WHERE id = $1")
        .bind(e.producto_id).fetch_one(&pool).await.expect("cache");
    assert_eq!(total["saldo_final"], cache);
    let (_, origen) = pedir(
        &app, &cookie, "GET",
        &format!("/api/reportes/kardex?producto_id={}&deposito_id={}", e.producto_id, e.origen_id),
        None,
    ).await;
    let en_origen: i32 = sqlx::query_scalar("SELECT cantidad FROM stock_por_deposito WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL")
        .bind(e.producto_id).bind(e.origen_id).fetch_one(&pool).await.expect("stock");
    assert_eq!(origen["saldo_final"], en_origen);
    assert_eq!(origen["deposito_id"], e.origen_id);

    let (status, body) = pedir(&app, &cookie, "GET", "/api/reportes/kardex?producto_id=999999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

/// 14.2: stock valorizado desde `stock_por_deposito`, sin productos en cero, total = Σ valor.
#[tokio::test]
async fn stock_valorizado_suma_el_stock_fisico() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let con_stock = crear_transferencia_de_prueba(&pool, 4).await;
    let sin_stock = crear_transferencia_de_prueba(&pool, 0).await;
    sqlx::query("UPDATE productos SET costo = 12.5, activo = FALSE WHERE id = $1")
        .bind(con_stock.producto_id).execute(&pool).await.expect("costo");

    let (status, r) = pedir(&app, &cookie, "GET", "/api/reportes/stock-valorizado", None).await;
    assert_eq!(status, StatusCode::OK);
    let items = r["items"].as_array().expect("items");
    let item = items.iter().find(|i| i["producto_id"] == con_stock.producto_id).expect("inactivo con stock aparece");
    assert_eq!(item["cantidad"], 4);
    assert_eq!(item["valor"].as_f64(), Some(50.0));
    assert!(!items.iter().any(|i| i["producto_id"] == sin_stock.producto_id), "sin stock no aparece");
    let suma: f64 = items.iter().filter_map(|i| i["valor"].as_f64()).sum();
    assert!((suma - r["total"].as_f64().unwrap_or(-1.0)).abs() < 0.01);
}

/// 14.3: rotación excluye transferencias y mermas; exige el rango.
#[tokio::test]
async fn rotacion_cuenta_solo_salidas_sin_transferencias() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let a = anio_propio();
    let e = crear_transferencia_de_prueba(&pool, 0).await;
    let entrada = movimiento(&app, &cookie, "ENTRADA", "COMPRA", e.origen_id, e.producto_id, 20).await;
    fechar(&pool, entrada, &format!("{a}-02-01T12:00:00Z")).await;
    let venta = movimiento(&app, &cookie, "SALIDA", "VENTA", e.origen_id, e.producto_id, -4).await;
    fechar(&pool, venta, &format!("{a}-03-10T12:00:00Z")).await;
    let merma = movimiento(&app, &cookie, "MERMA", "MERMA", e.origen_id, e.producto_id, -2).await;
    fechar(&pool, merma, &format!("{a}-03-11T12:00:00Z")).await;
    let (_, t) = pedir(&app, &cookie, "POST", "/api/transferencias", Some(json!({
        "producto_id": e.producto_id, "deposito_origen_id": e.origen_id,
        "deposito_destino_id": e.destino_id, "cantidad": 3
    }))).await;
    for campo in ["movimiento_salida_id", "movimiento_entrada_id"] {
        fechar(&pool, t[campo].as_i64().expect("id") as i32, &format!("{a}-03-12T12:00:00Z")).await;
    }

    let (status, r) = pedir(&app, &cookie, "GET", &format!("/api/reportes/rotacion?desde={a}-03-01&hasta={a}-03-31"), None).await;
    assert_eq!(status, StatusCode::OK, "{r}");
    let item = r["items"].as_array().expect("items").iter().find(|i| i["producto_id"] == e.producto_id).cloned().expect("item");
    assert_eq!(item["unidades_salidas"], 4, "{item}");
    assert_eq!(item["stock_inicial"], 20);
    assert_eq!(item["stock_final"], 14);
    assert_eq!(r["desde"], format!("{a}-03-01"));

    let (status, body) = pedir(&app, &cookie, "GET", "/api/reportes/rotacion", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["mensaje"], "Indicá las fechas desde y hasta del período (formato AAAA-MM-DD).");
}

/// 14.4: ventas confirmadas por día local (01:00 UTC cuenta el día anterior) y por producto.
#[tokio::test]
async fn ventas_por_dia_local_sin_anuladas() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let a = anio_propio();
    let e = crear_transferencia_de_prueba(&pool, 10).await;
    sqlx::query("UPDATE productos SET precio_venta = 100 WHERE id = $1")
        .bind(e.producto_id).execute(&pool).await.expect("precio");
    let mut ids = Vec::new();
    for cantidad in [2, 1, 3] {
        let (status, v) = pedir(&app, &cookie, "POST", "/api/ventas", Some(json!({
            "deposito_id": e.origen_id, "medio_pago": "EFECTIVO",
            "lineas": [{ "producto_id": e.producto_id, "cantidad": cantidad }]
        }))).await;
        assert_eq!(status, StatusCode::CREATED, "{v}");
        ids.push(v["venta"]["id"].as_i64().expect("id") as i32);
    }
    let (status, _) = pedir(&app, &cookie, "POST", &format!("/api/ventas/{}/anular", ids[2]), None).await;
    assert_eq!(status, StatusCode::OK);
    for (id, cuando) in [
        (ids[0], format!("{a}-05-10T15:00:00Z")),
        (ids[1], format!("{a}-05-11T01:00:00Z")),
        (ids[2], format!("{a}-05-10T16:00:00Z")),
    ] {
        sqlx::query("UPDATE ventas SET creado_en = $2::timestamptz WHERE id = $1")
            .bind(id).bind(cuando).execute(&pool).await.expect("fechar venta");
    }

    let (status, r) = pedir(&app, &cookie, "GET", &format!("/api/reportes/ventas?desde={a}-05-01&hasta={a}-05-31"), None).await;
    assert_eq!(status, StatusCode::OK, "{r}");
    assert_eq!(r["cantidad_ventas"], 2);
    assert_eq!(r["total"].as_f64(), Some(300.0));
    let dias: Vec<(String, i64)> = r["por_dia"].as_array().expect("por_dia").iter()
        .map(|d| (d["dia"].as_str().unwrap_or_default().to_string(), d["cantidad_ventas"].as_i64().unwrap_or(-1)))
        .collect();
    assert_eq!(dias, vec![(format!("{a}-05-10"), 2)], "la venta de las 01:00 UTC es del día anterior");
    assert_eq!(r["por_producto"][0]["unidades"], 3);
}

/// 14.5: CSV de los cuatro reportes con tipo, nombre de archivo y encabezados.
#[tokio::test]
async fn csv_de_los_cuatro_reportes() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let e = crear_transferencia_de_prueba(&pool, 1).await;
    for (uri, archivo, encabezado) in [
        (format!("/api/reportes/kardex?producto_id={}&desde=2026-10-01&hasta=2026-10-31&formato=csv", e.producto_id), "kardex_2026-10-01_2026-10-31.csv", "fecha,tipo,motivo,deposito,cantidad,saldo"),
        ("/api/reportes/stock-valorizado?formato=csv".to_string(), "stock-valorizado", "producto,sku,cantidad,costo_unitario,valor"),
        ("/api/reportes/rotacion?desde=2026-10-01&hasta=2026-10-31&formato=csv".to_string(), "rotacion_2026-10-01_2026-10-31.csv", "producto,unidades_salidas,stock_inicial,stock_final,stock_promedio,indice_rotacion"),
        ("/api/reportes/ventas?desde=2026-10-01&hasta=2026-10-31&formato=csv".to_string(), "ventas_2026-10-01_2026-10-31.csv", "dia,cantidad_ventas,total"),
        ("/api/reportes/ventas?desde=2026-10-01&hasta=2026-10-31&formato=csv&agrupar=producto".to_string(), "ventas_2026-10-01_2026-10-31.csv", "producto,unidades,total"),
    ] {
        let (status, tipo, disposicion, cuerpo) = pedir_texto(&app, &cookie, &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {cuerpo}");
        assert_eq!(tipo, "text/csv; charset=utf-8", "{uri}");
        assert!(disposicion.contains("attachment") && disposicion.contains(archivo), "{uri}: {disposicion}");
        assert!(cuerpo.starts_with(&format!("\u{FEFF}{encabezado}\r\n")), "{uri}: {cuerpo}");
    }
}

/// 14.6 + casos borde: 422 por rango invertido, fecha mal formada, formato y agrupar inválidos.
#[tokio::test]
async fn parametros_invalidos_responden_422() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    for (uri, mensaje) in [
        ("/api/reportes/ventas?desde=2026-11-02&hasta=2026-11-01", "La fecha desde no puede ser posterior a la fecha hasta."),
        ("/api/reportes/ventas?desde=2026-11-01", "Indicá las fechas desde y hasta del período (formato AAAA-MM-DD)."),
        ("/api/reportes/rotacion?desde=01-11-2026&hasta=2026-11-30", "El parámetro desde debe estar en formato ISO 8601 (AAAA-MM-DD)."),
        ("/api/reportes/stock-valorizado?formato=pdf", "El parámetro formato acepta json o csv."),
        ("/api/reportes/ventas?desde=2026-11-01&hasta=2026-11-30&agrupar=mes", "El parámetro agrupar acepta dia o producto."),
    ] {
        let (status, body) = pedir(&app, &cookie, "GET", uri, None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{uri}: {body}");
        assert_eq!(body["error"]["mensaje"], mensaje, "{uri}");
    }
}
