//! 016 (T006, T016) — importación CSV por HTTP: dry-run, aplicación atómica, errores
//! con fila/columna, límites, RBAC y SC-024.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use common::{app_de_test, cuerpo_json, sesion_gestion, sesion_operacion, sku_unico};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

const LIMITE: &str = "XLIMITEX";
const ENCABEZADO: &str = "nombre,sku,categoria,unidad_medida,costo,precio_venta,stock_minimo,codigo_barras,stock_inicial,deposito";

fn multipart(cookie: &str, uri: &str, archivo: Option<&[u8]>) -> Request<Body> {
    let mut cuerpo: Vec<u8> = Vec::new();
    if let Some(bytes) = archivo {
        cuerpo.extend_from_slice(
            format!("--{LIMITE}\r\nContent-Disposition: form-data; name=\"archivo\"; filename=\"p.csv\"\r\nContent-Type: text/csv\r\n\r\n").as_bytes(),
        );
        cuerpo.extend_from_slice(bytes);
        cuerpo.extend_from_slice(b"\r\n");
    } else {
        cuerpo.extend_from_slice(
            format!("--{LIMITE}\r\nContent-Disposition: form-data; name=\"otro\"\r\n\r\nx\r\n").as_bytes(),
        );
    }
    cuerpo.extend_from_slice(format!("--{LIMITE}--\r\n").as_bytes());
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={LIMITE}"))
        .body(Body::from(cuerpo))
        .expect("request multipart")
}

async fn subir(app: &Router, cookie: &str, modo: &str, csv: Option<&str>) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(multipart(cookie, &format!("/api/importaciones/productos?modo={modo}"), csv.map(str::as_bytes)))
        .await
        .expect("request");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

async fn categoria(pool: &PgPool) -> String {
    let nombre = format!("Import {}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
    sqlx::query("INSERT INTO categorias (nombre) VALUES ($1)")
        .bind(&nombre)
        .execute(pool)
        .await
        .expect("categoría");
    nombre
}

async fn deposito(pool: &PgPool) -> (i32, String) {
    let nombre = format!("Depósito import {}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
    let id: i32 = sqlx::query_scalar("INSERT INTO depositos (nombre, identificador) VALUES ($1, $2) RETURNING id")
        .bind(&nombre)
        .bind(sku_unico("IMP"))
        .fetch_one(pool)
        .await
        .expect("depósito");
    (id, nombre)
}

async fn existe_sku(pool: &PgPool, sku: &str) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM productos WHERE sku = $1)")
        .bind(sku)
        .fetch_one(pool)
        .await
        .expect("sku")
}

#[tokio::test]
async fn validar_no_escribe_y_aplicar_crea_productos_stock_y_movimiento() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let cat = categoria(&pool).await;
    let (dep_id, dep) = deposito(&pool).await;
    let (a, b) = (sku_unico("IMPA"), sku_unico("IMPB"));
    let csv = format!("{ENCABEZADO}\nYerba import,{a},{cat},UNIDAD,100.50,150,2,,7,{dep}\nArena import,{b},{cat},KG,10,20,0,,,\n");

    let (status, r) = subir(&app, &cookie, "validar", Some(&csv)).await;
    assert_eq!(status, StatusCode::OK, "{r}");
    assert_eq!((r["filas_leidas"].as_i64(), r["validas"].as_i64(), r["con_error"].as_i64()), (Some(2), Some(2), Some(0)));
    assert_eq!(r["aplicado"], false);
    assert!(!existe_sku(&pool, &a).await, "el dry-run no escribe");

    let (status, r) = subir(&app, &cookie, "aplicar", Some(&csv)).await;
    assert_eq!(status, StatusCode::CREATED, "{r}");
    assert_eq!(r["creados"], 2);
    assert_eq!(r["aplicado"], true);

    let (pid, stock_actual): (i32, i32) = sqlx::query_as("SELECT id, stock_actual FROM productos WHERE sku = $1")
        .bind(&a).fetch_one(&pool).await.expect("producto");
    assert_eq!(stock_actual, 7);
    let en_deposito: i32 = sqlx::query_scalar("SELECT cantidad FROM stock_por_deposito WHERE producto_id = $1 AND deposito_id = $2 AND ubicacion_id IS NULL")
        .bind(pid).bind(dep_id).fetch_one(&pool).await.expect("stock");
    assert_eq!(en_deposito, 7);
    let (tipo, motivo, delta): (String, String, i32) = sqlx::query_as(
        "SELECT m.tipo, m.motivo, ml.cantidad_delta FROM movimientos m JOIN movimiento_lineas ml ON ml.movimiento_id = m.id WHERE ml.producto_id = $1",
    )
    .bind(pid).fetch_one(&pool).await.expect("movimiento");
    assert_eq!((tipo.as_str(), motivo.as_str(), delta), ("ENTRADA", "IMPORTACION", 7));
    let sin_mov: i64 = sqlx::query_scalar("SELECT count(*) FROM movimiento_lineas ml JOIN productos p ON p.id = ml.producto_id WHERE p.sku = $1")
        .bind(&b).fetch_one(&pool).await.expect("sin movimiento");
    assert_eq!(sin_mov, 0, "stock_inicial vacío no genera movimiento");
}

#[tokio::test]
async fn una_fila_invalida_en_aplicar_no_crea_nada() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let cat = categoria(&pool).await;
    let (_, dep) = deposito(&pool).await;
    let existente = sku_unico("IMPE");
    let csv_previo = format!("{ENCABEZADO}\nPrevio,{existente},{cat},UNIDAD,1,1,0,,,\n");
    let (status, _) = subir(&app, &cookie, "aplicar", Some(&csv_previo)).await;
    assert_eq!(status, StatusCode::CREATED);

    let (ok, rep) = (sku_unico("IMPO"), sku_unico("IMPR"));
    let csv = format!(
        "{ENCABEZADO}\nBueno,{ok},{cat},UNIDAD,1,1,0,,,\nExistente,{existente},{cat},UNIDAD,1,1,0,,,\nRep1,{rep},{cat},UNIDAD,1,1,0,,,\nRep2,{rep},{cat},UNIDAD,1,1,0,,,\nSinCat,{},Categoría que no existe,UNIDAD,1,1,0,,,\nSinDep,{},{cat},UNIDAD,1,1,0,,3,Depósito inexistente XYZ\n",
        sku_unico("IMPC"),
        sku_unico("IMPD")
    );
    let (status, e) = subir(&app, &cookie, "aplicar", Some(&csv)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["error"]["codigo"], "importacion_invalida");
    let detalles = e["error"]["detalles"].as_array().expect("detalles");
    let mensajes: Vec<String> = detalles.iter().filter_map(|d| d["mensaje"].as_str().map(String::from)).collect();
    assert!(mensajes.contains(&format!("El SKU '{existente}' ya existe.")), "{mensajes:?}");
    assert!(mensajes.contains(&format!("El SKU '{rep}' está repetido en el archivo (filas 4 y 5).")), "{mensajes:?}");
    assert!(mensajes.contains(&"La categoría 'Categoría que no existe' no existe.".to_string()), "{mensajes:?}");
    assert!(mensajes.contains(&"El depósito 'Depósito inexistente XYZ' no existe.".to_string()), "{mensajes:?}");
    assert!(!existe_sku(&pool, &ok).await, "atomicidad: ninguna fila se crea");
    let _ = dep;

    // El dry-run del mismo archivo responde 200 con los mismos errores.
    let (status, r) = subir(&app, &cookie, "validar", Some(&csv)).await;
    assert_eq!(status, StatusCode::OK, "{r}");
    // La primera aparición del SKU repetido (fila 4) es válida; el error va en la fila 5.
    assert_eq!(r["validas"], 2);
    assert_eq!(r["con_error"], 4);
}

#[tokio::test]
async fn principal_por_defecto_solo_con_stock() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let cat = categoria(&pool).await;
    let hay_principal: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM depositos WHERE lower(nombre) = 'principal' AND activo)")
        .fetch_one(&pool).await.expect("principal");
    let csv = format!("{ENCABEZADO}\nCon stock,{},{cat},UNIDAD,1,1,0,,4,\n", sku_unico("IMPP"));
    let (status, r) = subir(&app, &cookie, "validar", Some(&csv)).await;
    assert_eq!(status, StatusCode::OK, "{r}");
    if hay_principal {
        assert_eq!(r["con_error"], 0, "{r}");
    } else {
        assert_eq!(r["errores"][0]["mensaje"], "No se encontró el depósito Principal.", "{r}");
    }
    // Sin stock, el depósito no importa.
    let csv = format!("{ENCABEZADO}\nSin stock,{},{cat},UNIDAD,1,1,0,,,\n", sku_unico("IMPS"));
    let (_, r) = subir(&app, &cookie, "validar", Some(&csv)).await;
    assert_eq!(r["con_error"], 0, "{r}");
}

#[tokio::test]
async fn limites_modo_archivo_y_permisos() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operacion = sesion_operacion(&pool).await;

    let mut grande = format!("{ENCABEZADO}\n");
    for i in 0..5001 {
        grande.push_str(&format!("P{i},SKUX-{i:05},Cat,UNIDAD,1,1,0,,,\n"));
    }
    let (status, e) = subir(&app, &cookie, "validar", Some(&grande)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["error"]["codigo"], "importacion_invalida");
    assert_eq!(e["error"]["detalles"][0]["mensaje"], "El archivo supera el límite de 5.000 filas o 5 MB.");

    let mut pesado = format!("{ENCABEZADO}\n");
    let relleno = "x".repeat(150);
    while pesado.len() <= 5 * 1024 * 1024 {
        pesado.push_str(&format!("{relleno},SKU-AAA,Cat,UNIDAD,1,1,0,,,\n"));
    }
    let (status, e) = subir(&app, &cookie, "validar", Some(&pesado)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["error"]["detalles"][0]["mensaje"], "El archivo supera el límite de 5.000 filas o 5 MB.");

    let (status, e) = subir(&app, &cookie, "probar", Some("a")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["error"]["codigo"], "validacion");
    assert_eq!(e["error"]["mensaje"], "modo debe ser validar o aplicar.");

    let (status, e) = subir(&app, &cookie, "validar", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["error"]["codigo"], "importacion_invalida");
    assert_eq!(e["error"]["detalles"][0]["mensaje"], "Falta el archivo.");

    let (status, e) = subir(&app, &operacion, "validar", Some(ENCABEZADO)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{e}");
    assert_eq!(e["error"]["codigo"], "sin_permisos");
}

/// T016 (SC-024): 5.000 filas en menos de 60 s. Pesado para la BD compartida: se corre a
/// mano con `cargo test --test test_importacion_http sc024 -- --ignored --nocapture`.
#[tokio::test]
#[ignore]
async fn sc024() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let cat = categoria(&pool).await;
    let (dep_id, dep) = deposito(&pool).await;
    let marca = uuid::Uuid::new_v4().simple().to_string()[..6].to_uppercase();
    let mut csv = format!("{ENCABEZADO}\n");
    for i in 0..5000 {
        csv.push_str(&format!("Producto SC024 {i},S24{marca}-{i:04},{cat},UNIDAD,10,20,0,,3,{dep}\n"));
    }
    let inicio = std::time::Instant::now();
    let (status, r) = subir(&app, &cookie, "aplicar", Some(&csv)).await;
    let duracion = inicio.elapsed();
    println!("SC-024: 5.000 filas en {duracion:?}");
    assert_eq!(status, StatusCode::CREATED, "{r}");
    assert_eq!(r["creados"], 5000);
    assert!(duracion.as_secs() < 60, "SC-024 superado: {duracion:?}");
    let movimientos: i64 = sqlx::query_scalar("SELECT count(*) FROM movimientos WHERE deposito_id = $1 AND motivo = 'IMPORTACION'")
        .bind(dep_id).fetch_one(&pool).await.expect("movimientos");
    assert_eq!(movimientos, 5000);

    // Limpieza de la BD compartida.
    for sql in [
        "DELETE FROM movimiento_lineas WHERE movimiento_id IN (SELECT id FROM movimientos WHERE deposito_id = $1)",
        "DELETE FROM movimientos WHERE deposito_id = $1",
        "DELETE FROM stock_por_deposito WHERE deposito_id = $1",
        "DELETE FROM depositos WHERE id = $1",
    ] {
        sqlx::query(sql).bind(dep_id).execute(&pool).await.expect("limpieza");
    }
    sqlx::query("DELETE FROM productos WHERE sku LIKE $1")
        .bind(format!("S24{marca}-%"))
        .execute(&pool)
        .await
        .expect("limpieza de productos");
}
