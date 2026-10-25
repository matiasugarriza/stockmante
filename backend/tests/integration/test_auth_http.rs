//! T021/T022 — Integración HTTP de /api/auth contra el router real (contracts/api.md).

#[path = "../unit/common/mod.rs"]
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::{borrar_usuario, crear_usuario, email_unico, pool_de_test};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use stockmante_backend::app::{construir_router, AppEstado};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::config::AppConfig;
use stockmante_backend::mail::envio::CorreoDev;
use std::sync::Arc;
use tower::ServiceExt;

async fn app_de_test() -> (axum::Router, sqlx::PgPool) {
    let _ = dotenvy::dotenv();
    let pool = pool_de_test().await;
    let config = AppConfig::desde_entorno().expect("config de test");
    let estado = AppEstado {
        pool: pool.clone(),
        config,
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(CorreoDev::new()),
    };
    (construir_router(estado), pool)
}

async fn cuerpo_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }
}

fn cookie_de_sesion(res: &axum::response::Response) -> String {
    res.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|c| c.starts_with("stockmante_sesion="))
        .expect("debe venir Set-Cookie de sesión")
        .split(';')
        .next()
        .expect("cookie con valor")
        .to_string()
}

fn login_request(_app: &axum::Router, email: &str, password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "password": password }).to_string(),
        ))
        .expect("request")
}

#[tokio::test]
async fn login_ok_setea_cookie_y_sesion_devuelve_usuario() {
    let (app, pool) = app_de_test().await;
    let email = email_unico("http-login-ok");
    crear_usuario(
        &pool,
        "Http Ok",
        &email,
        "Password1",
        "ADMIN",
        "GESTION",
        true,
        false,
    )
    .await;

    let res = app
        .clone()
        .oneshot(login_request(&app, &email, "Password1"))
        .await
        .expect("login");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let cookie = cookie_de_sesion(&res);
    assert!(cookie.starts_with("stockmante_sesion="));

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesion");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["usuario"]["email"], email);
    assert_eq!(body["usuario"]["rol"], "ADMIN");
    assert_eq!(body["usuario"]["familia_rol"], "GESTION");
    assert_eq!(body["usuario"]["debe_cambiar_password"], false);

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn login_fallido_responde_401_identico_para_password_y_email() {
    let (app, pool) = app_de_test().await;
    let email = email_unico("http-login-401");
    crear_usuario(
        &pool,
        "Http Fail",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let res = app
        .clone()
        .oneshot(login_request(&app, &email, "PasswordX"))
        .await
        .expect("login mal pass");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body_pass = cuerpo_json(res).await;

    let res = app
        .clone()
        .oneshot(login_request(&app, "nadie@test.local", "Password1"))
        .await
        .expect("login email fantasma");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body_email = cuerpo_json(res).await;

    assert_eq!(body_pass["error"]["codigo"], "credenciales_invalidas");
    assert_eq!(body_pass, body_email, "mensaje idéntico (anti-enumeración)");

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn sesion_sin_cookie_es_401() {
    let (app, _pool) = app_de_test().await;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesion");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "no_autenticado");
}

#[tokio::test]
async fn logout_revoca_la_sesion_y_borra_cookie() {
    let (app, pool) = app_de_test().await;
    let email = email_unico("http-logout");
    crear_usuario(
        &pool,
        "Http Logout",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let res = app
        .clone()
        .oneshot(login_request(&app, &email, "Password1"))
        .await
        .expect("login");
    let cookie = cookie_de_sesion(&res);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("logout");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // la cookie vieja ya no da acceso
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesion tras logout");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn cambiar_password_flujo_completo_y_422_por_politica() {
    let (app, pool) = app_de_test().await;
    let email = email_unico("http-cambio");
    crear_usuario(
        &pool,
        "Http Cambio",
        &email,
        "Temporal1",
        "ADMIN",
        "GESTION",
        true,
        true,
    )
    .await;

    let res = app
        .clone()
        .oneshot(login_request(&app, &email, "Temporal1"))
        .await
        .expect("login");
    let cookie = cookie_de_sesion(&res);

    // 422 por política
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/cambiar-password")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({ "password_actual": "Temporal1", "password_nueva": "corta" })
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("cambio");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 204 con política válida
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/cambiar-password")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({ "password_actual": "Temporal1", "password_nueva": "Nueva1234" })
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("cambio");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // la sesión sigue viva y ya no debe cambiar password
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesion");
    let body = cuerpo_json(res).await;
    assert_eq!(body["usuario"]["debe_cambiar_password"], false);

    borrar_usuario(&pool, &email).await;
}
