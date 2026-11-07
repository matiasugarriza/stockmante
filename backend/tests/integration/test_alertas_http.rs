//! 011 — alertas de stock mínimo: listado (11.1, 11.2, 11.4, 11.7) y ciclo de
//! correo (11.3, 11.5, 11.6, SC-019).
//!
//! La BD de test es compartida y acumula cientos de gestores y productos bajo
//! mínimo; por eso los tests del ciclo usan `AlcanceCiclo` acotado a sus propios
//! usuarios y productos y una fecha propia (reloj falso). El `POST` sin alcance
//! corre sobre toda la base y queda `#[ignore]` (se ejecuta a mano).

#[path = "../unit/common/mod.rs"]
mod common;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::Router;
use chrono::{TimeZone, Utc};
use common::{
    app_de_test, crear_transferencia_de_prueba, cuerpo_json, sesion_con_familia, sesion_gestion,
    sesion_operacion, solicitud,
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Mutex;
use stockmante_backend::auth::reloj::RelojFalso;
use stockmante_backend::mail::envio::{CorreoDev, EnvioCorreo};
use stockmante_backend::services::alertas::{ciclo, AlcanceCiclo};
use tower::ServiceExt;

async fn pedir(app: &Router, cookie: &str, metodo: &str, uri: &str) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud(metodo, uri, cookie, None))
        .await
        .expect("request de prueba");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

/// Producto activo con `stock` unidades y mínimo `minimo`.
async fn producto(pool: &PgPool, stock: i32, minimo: i32) -> i32 {
    let e = crear_transferencia_de_prueba(pool, stock).await;
    sqlx::query("UPDATE productos SET stock_minimo = $2 WHERE id = $1")
        .bind(e.producto_id)
        .bind(minimo)
        .execute(pool)
        .await
        .expect("mínimo");
    e.producto_id
}

/// Busca el producto recorriendo todas las páginas del listado.
async fn alerta_de(app: &Router, cookie: &str, producto_id: i32) -> Option<Value> {
    for pagina in 1..=200 {
        let (status, body) = pedir(
            app,
            cookie,
            "GET",
            &format!("/api/alertas?pagina={pagina}&por_pagina=100"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let items = body["items"].as_array().cloned().unwrap_or_default();
        if let Some(item) = items.iter().find(|i| i["producto_id"] == producto_id) {
            return Some(item.clone());
        }
        if items.len() < 100 {
            return None;
        }
    }
    None
}

#[tokio::test]
async fn listado_de_alertas_y_rbac() {
    let (app, pool, _correo) = app_de_test().await;
    let gestion = sesion_gestion(&pool).await;
    let operacion = sesion_operacion(&pool).await;

    // 11.1: stock 4, mínimo 10 → faltante 6.
    let bajo = producto(&pool, 4, 10).await;
    let item = alerta_de(&app, &gestion, bajo).await.expect("aparece bajo mínimo");
    assert_eq!(item["stock_actual"], 4);
    assert_eq!(item["stock_minimo"], 10);
    assert_eq!(item["faltante"], 6);

    // 11.2: repuesto a 12 deja de aparecer.
    let repuesto = producto(&pool, 12, 10).await;
    assert!(alerta_de(&app, &gestion, repuesto).await.is_none());

    // 11.4: mínimo 0 nunca alerta.
    let sin_minimo = producto(&pool, 0, 0).await;
    assert!(alerta_de(&app, &gestion, sin_minimo).await.is_none());

    // 11.7: OPERACION → 403 en ambas rutas.
    for (metodo, uri) in [("GET", "/api/alertas"), ("POST", "/api/alertas/enviar")] {
        let (status, body) = pedir(&app, &operacion, metodo, uri).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{metodo} {uri}: {body}");
        assert_eq!(body["error"]["codigo"], "sin_permisos");
    }

    let (status, body) = pedir(&app, &gestion, "GET", "/api/alertas?por_pagina=101").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

/// Reloj en un día propio por test: la deduplicación es por fecha.
fn reloj_del_dia(dia: u32) -> RelojFalso {
    RelojFalso {
        ahora: Utc
            .with_ymd_and_hms(2030, 1, dia, 15, 0, 0)
            .single()
            .expect("fecha válida"),
    }
}

async fn gestor(pool: &PgPool) -> (i32, String) {
    let (email, _) = sesion_con_familia(pool, "GERENTE", "GESTION").await;
    let id: i32 = sqlx::query_scalar("SELECT id FROM usuarios WHERE email = $1")
        .bind(&email)
        .fetch_one(pool)
        .await
        .expect("id del gestor");
    (id, email)
}

async fn filas(pool: &PgPool, usuarios: &[i32]) -> Vec<(i32, String, i32, Option<String>)> {
    sqlx::query_as(
        r#"SELECT producto_id, estado, intentos, error FROM notificaciones_stock
            WHERE usuario_id = ANY($1) ORDER BY producto_id, usuario_id"#,
    )
    .bind(usuarios)
    .fetch_all(pool)
    .await
    .expect("filas de notificaciones")
}

/// 11.3 + SC-019: dos ciclos el mismo día → un correo por usuario con los tres
/// productos y una sola fila por (producto, usuario, fecha).
#[tokio::test]
async fn dos_ciclos_el_mismo_dia_no_duplican() {
    let (_app, pool, _correo) = app_de_test().await;
    let productos = vec![
        producto(&pool, 1, 5).await,
        producto(&pool, 2, 5).await,
        producto(&pool, 0, 3).await,
    ];
    let (g1, email1) = gestor(&pool).await;
    let (g2, _) = gestor(&pool).await;
    let alcance = AlcanceCiclo {
        productos: Some(productos.clone()),
        usuarios: Some(vec![g1, g2]),
    };
    let correo = CorreoDev::new();
    let reloj = reloj_del_dia(10);

    let primera = ciclo(&pool, &correo, &reloj, &alcance).await.expect("primer ciclo");
    assert_eq!(primera.productos, 3);
    assert_eq!(primera.destinatarios, 2);
    assert_eq!(primera.enviados, 6);
    assert_eq!(primera.fallidos, 0);

    let segunda = ciclo(&pool, &correo, &reloj, &alcance).await.expect("segundo ciclo");
    assert_eq!(segunda.enviados, 0);
    assert_eq!(segunda.fallidos, 0);

    let bandeja = correo.bandeja();
    assert_eq!(bandeja.len(), 2, "un correo por usuario: {bandeja:?}");
    let del_primero = bandeja.iter().find(|(d, _, _)| *d == email1).expect("correo de g1");
    assert_eq!(del_primero.2.matches("\n- ").count(), 3, "{}", del_primero.2);

    let registro = filas(&pool, &[g1, g2]).await;
    assert_eq!(registro.len(), 6);
    assert!(registro.iter().all(|f| f.1 == "enviada"));

    // Al día siguiente se repite el aviso.
    let otro_dia = ciclo(&pool, &correo, &reloj_del_dia(11), &alcance).await.expect("día 2");
    assert_eq!(otro_dia.enviados, 6);
}

/// 11.5: sin destinatarios GESTION activos el ciclo no falla.
#[tokio::test]
async fn sin_destinatarios_responde_cero() {
    let (_app, pool, _correo) = app_de_test().await;
    let p = producto(&pool, 1, 5).await;
    let alcance = AlcanceCiclo {
        productos: Some(vec![p]),
        usuarios: Some(vec![]),
    };
    let correo = CorreoDev::new();
    let r = ciclo(&pool, &correo, &reloj_del_dia(12), &alcance).await.expect("ciclo");
    assert_eq!((r.productos, r.destinatarios, r.enviados, r.fallidos), (1, 0, 0, 0));
    assert!(correo.bandeja().is_empty());
}

/// Transporte que falla siempre (11.6).
struct CorreoQueFalla;

#[async_trait]
impl EnvioCorreo for CorreoQueFalla {
    async fn enviar_password_temporal(&self, _: &str, _: &str, _: &str) -> Result<(), String> {
        Err("SMTP no disponible".into())
    }
    async fn enviar_verificacion(&self, _: &str, _: &str, _: &str, _: &str) -> Result<(), String> {
        Err("SMTP no disponible".into())
    }
    async fn enviar_alerta_stock(&self, _: &str, _: &str, _: &str) -> Result<(), String> {
        Err("SMTP no disponible".into())
    }
}

/// Transporte que registra los destinos (para verificar el reintento).
#[derive(Default)]
struct CorreoQueAnota(Mutex<Vec<String>>);

#[async_trait]
impl EnvioCorreo for CorreoQueAnota {
    async fn enviar_password_temporal(&self, _: &str, _: &str, _: &str) -> Result<(), String> {
        Ok(())
    }
    async fn enviar_verificacion(&self, _: &str, _: &str, _: &str, _: &str) -> Result<(), String> {
        Ok(())
    }
    async fn enviar_alerta_stock(&self, destino: &str, _: &str, cuerpo: &str) -> Result<(), String> {
        if let Ok(mut v) = self.0.lock() {
            v.push(format!("{destino}|{}", cuerpo.matches("\n- ").count()));
        }
        Ok(())
    }
}

/// 11.6: un fallo de transporte deja filas `fallida` y el ciclo siguiente las reintenta.
#[tokio::test]
async fn fallo_de_transporte_se_reintenta() {
    let (_app, pool, _correo) = app_de_test().await;
    let productos = vec![producto(&pool, 1, 5).await, producto(&pool, 0, 2).await];
    let (g, email) = gestor(&pool).await;
    let alcance = AlcanceCiclo {
        productos: Some(productos.clone()),
        usuarios: Some(vec![g]),
    };
    let reloj = reloj_del_dia(13);

    let fallido = ciclo(&pool, &CorreoQueFalla, &reloj, &alcance).await.expect("ciclo con fallo");
    assert_eq!(fallido.enviados, 0);
    assert_eq!(fallido.fallidos, 2);
    let registro = filas(&pool, &[g]).await;
    assert!(registro
        .iter()
        .all(|f| f.1 == "fallida" && f.2 == 1 && f.3.as_deref() == Some("SMTP no disponible")));

    let anota = CorreoQueAnota::default();
    let reintento = ciclo(&pool, &anota, &reloj, &alcance).await.expect("reintento");
    assert_eq!(reintento.enviados, 2);
    assert_eq!(reintento.fallidos, 0);
    let enviados = anota.0.lock().map(|v| v.clone()).unwrap_or_default();
    assert_eq!(enviados, vec![format!("{email}|2")]);
    assert!(filas(&pool, &[g]).await.iter().all(|f| f.1 == "enviada"));
}

/// 11.5 por HTTP sobre toda la base (pesado en la BD compartida): se corre a mano con
/// `cargo test --test test_alertas_http enviar_por_http -- --ignored`.
#[tokio::test]
#[ignore]
async fn enviar_por_http() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let (status, body) = pedir(&app, &cookie, "POST", "/api/alertas/enviar").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for campo in ["productos", "destinatarios", "enviados", "fallidos"] {
        assert!(body[campo].is_i64(), "{campo}: {body}");
    }
    assert!(body["fecha"].is_string());
    let (status, segunda) = pedir(&app, &cookie, "POST", "/api/alertas/enviar").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(segunda["enviados"], 0, "idempotente en el día: {segunda}");
}
