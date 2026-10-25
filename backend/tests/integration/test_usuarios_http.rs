//! T030 — Integración HTTP de /api/usuarios contra el router real (contracts/api.md).

#[path = "../unit/common/mod.rs"]
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::{borrar_usuario, crear_usuario, email_unico, pool_de_test};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use stockmante_backend::app::{construir_router, AppEstado};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::config::AppConfig;
use stockmante_backend::mail::envio::CorreoDev;
use tower::ServiceExt;

async fn app_de_test() -> (axum::Router, sqlx::PgPool, Arc<CorreoDev>) {
    let _ = dotenvy::dotenv();
    let pool = pool_de_test().await;
    let config = AppConfig::desde_entorno().expect("config de test");
    let correo = Arc::new(CorreoDev::new());
    let estado = AppEstado {
        pool: pool.clone(),
        config,
        reloj: Arc::new(RelojSistema),
        correo: correo.clone(),
    };
    (construir_router(estado), pool, correo)
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

fn login_request(email: &str, password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "password": password }).to_string(),
        ))
        .expect("request de login")
}

fn solicitud(metodo: &str, uri: &str, cookie: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(metodo)
        .uri(uri)
        .header(header::COOKIE, cookie);
    let cuerpo = match body {
        Some(valor) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(valor.to_string())
        }
        None => Body::empty(),
    };
    builder.body(cuerpo).expect("request")
}

async fn intentar_login(
    app: &axum::Router,
    email: &str,
    password: &str,
) -> (StatusCode, Option<String>) {
    let res = app
        .clone()
        .oneshot(login_request(email, password))
        .await
        .expect("login");
    if res.status() == StatusCode::NO_CONTENT {
        let cookie = cookie_de_sesion(&res);
        (res.status(), Some(cookie))
    } else {
        (res.status(), None)
    }
}

async fn crear_gestor_y_loguear(
    app: &axum::Router,
    pool: &sqlx::PgPool,
    etiqueta: &str,
) -> (String, String) {
    let email = email_unico(etiqueta);
    crear_usuario(
        pool,
        "Gestor Test",
        &email,
        "Password1",
        "ADMIN",
        "GESTION",
        true,
        false,
    )
    .await;
    let (status, cookie) = intentar_login(app, &email, "Password1").await;
    assert_eq!(status, StatusCode::NO_CONTENT, "login del gestor de test");
    (email, cookie.expect("cookie del gestor"))
}

async fn id_de(pool: &sqlx::PgPool, email: &str) -> i32 {
    sqlx::query_scalar("SELECT id FROM usuarios WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("id del usuario de test")
}

fn temporal_enviada(correo: &CorreoDev, destino: &str) -> String {
    correo
        .bandeja()
        .into_iter()
        .rev()
        .find(|(email, _, _)| email == destino)
        .map(|(_, _, temporal)| temporal)
        .expect("el correo con la contraseña temporal debe existir")
}

#[tokio::test]
async fn alta_devuelve_201_sin_secretos_y_la_temporal_sirve_para_login() {
    let (app, pool, correo) = app_de_test().await;
    let (admin_email, cookie) = crear_gestor_y_loguear(&app, &pool, "http-alta-admin").await;
    let email = email_unico("http-alta-nuevo");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/usuarios",
            &cookie,
            Some(json!({ "nombre": "Nuevo Operario", "email": email, "rol": "OPERARIO" })),
        ))
        .await
        .expect("alta de usuario");
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = cuerpo_json(res).await;
    assert_eq!(body["email"], email);
    assert_eq!(body["rol"], "OPERARIO");
    assert_eq!(body["familia_rol"], "OPERACION");
    assert_eq!(body["activo"], true);
    assert_eq!(body["password_temporal"], true);
    assert!(body["id"].is_number());
    assert!(
        body.get("password_hash").is_none(),
        "la respuesta nunca expone el hash"
    );
    assert!(
        body.get("password").is_none(),
        "la respuesta nunca expone la temporal"
    );

    let temporal = temporal_enviada(&correo, &email);
    let (status, cookie_nuevo) = intentar_login(&app, &email, &temporal).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "la temporal debe servir");

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .header(
                    header::COOKIE,
                    cookie_nuevo.expect("cookie del nuevo usuario"),
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesión del nuevo usuario");
    let body = cuerpo_json(res).await;
    assert_eq!(body["usuario"]["email"], email);
    assert_eq!(body["usuario"]["debe_cambiar_password"], true);

    borrar_usuario(&pool, &email).await;
    borrar_usuario(&pool, &admin_email).await;
}

#[tokio::test]
async fn email_duplicado_es_409_y_las_entradas_invalidas_son_422() {
    let (app, pool, _correo) = app_de_test().await;
    let (admin_email, cookie) = crear_gestor_y_loguear(&app, &pool, "http-dup-admin").await;
    let email = email_unico("http-dup");

    let alta = json!({ "nombre": "Persona", "email": email, "rol": "OPERARIO" });
    let res = app
        .clone()
        .oneshot(solicitud("POST", "/api/usuarios", &cookie, Some(alta)))
        .await
        .expect("primer alta");
    assert_eq!(res.status(), StatusCode::CREATED);

    // Duplicado con mayúsculas: la comparación normaliza.
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/usuarios",
            &cookie,
            Some(json!({ "nombre": "Otra", "email": email.to_uppercase(), "rol": "GERENTE" })),
        ))
        .await
        .expect("alta duplicada");
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "email_duplicado");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/usuarios",
            &cookie,
            Some(json!({ "nombre": "Persona", "email": "no-es-email", "rol": "OPERARIO" })),
        ))
        .await
        .expect("email inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "validacion");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/usuarios",
            &cookie,
            Some(json!({ "nombre": "Persona", "email": email_unico("http-rol"), "rol": "SUPERVISOR" })),
        ))
        .await
        .expect("rol inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    borrar_usuario(&pool, &email).await;
    borrar_usuario(&pool, &admin_email).await;
}

#[tokio::test]
async fn operario_recibe_403_en_la_gestion_de_usuarios() {
    let (app, pool, _correo) = app_de_test().await;
    let email = email_unico("http-operario");
    crear_usuario(
        &pool,
        "Operario Test",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;
    let (status, cookie) = intentar_login(&app, &email, "Password1").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let cookie = cookie.expect("cookie del operario");

    let casos: [(&str, &str, Option<Value>); 7] = [
        ("GET", "/api/usuarios", None),
        ("GET", "/api/usuarios/1", None),
        (
            "POST",
            "/api/usuarios",
            Some(json!({ "nombre": "Intruso", "email": "intruso@test.local", "rol": "OPERARIO" })),
        ),
        (
            "PUT",
            "/api/usuarios/1",
            Some(json!({ "nombre": "Intruso" })),
        ),
        ("POST", "/api/usuarios/1/desactivar", None),
        ("POST", "/api/usuarios/1/reactivar", None),
        ("POST", "/api/usuarios/1/password-temporal", None),
    ];
    for (metodo, uri, body) in casos {
        let res = app
            .clone()
            .oneshot(solicitud(metodo, uri, &cookie, body))
            .await
            .expect("request de operario");
        assert_eq!(res.status(), StatusCode::FORBIDDEN, "{metodo} {uri}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "sin_permisos");
    }

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn sin_sesion_la_gestion_de_usuarios_es_401() {
    let (app, _pool, _correo) = app_de_test().await;

    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/usuarios")
                .body(Body::empty())
                .expect("request sin sesión"),
        )
        .await
        .expect("listado sin sesión");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "no_autenticado");
}

#[tokio::test]
async fn listar_pagina_filtra_por_estado_y_el_detalle_faltante_es_404() {
    let (app, pool, _correo) = app_de_test().await;
    let (admin_email, cookie) = crear_gestor_y_loguear(&app, &pool, "http-lista-admin").await;
    let email_activo = email_unico("http-lista-activo");
    let email_inactivo = email_unico("http-lista-inactivo");
    crear_usuario(
        &pool,
        "Activo",
        &email_activo,
        "Password1",
        "CAJERO",
        "OPERACION",
        true,
        false,
    )
    .await;
    crear_usuario(
        &pool,
        "Inactivo",
        &email_inactivo,
        "Password1",
        "CAJERO",
        "OPERACION",
        false,
        false,
    )
    .await;

    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            "/api/usuarios?activo=true&por_pagina=100",
            &cookie,
            None,
        ))
        .await
        .expect("listado activo");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["pagina"].as_i64(), Some(1));
    assert_eq!(body["por_pagina"].as_i64(), Some(100));
    let items = body["items"].as_array().expect("items de la página");
    assert!(
        items
            .iter()
            .any(|u| u["email"].as_str() == Some(email_activo.as_str())),
        "el activo debe aparecer"
    );
    assert!(
        !items
            .iter()
            .any(|u| u["email"].as_str() == Some(email_inactivo.as_str())),
        "el inactivo no debe aparecer con activo=true"
    );
    assert!(items.iter().all(|u| u["activo"] == true));
    assert!(items.iter().all(|u| u.get("password_hash").is_none()));

    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            "/api/usuarios?activo=false&por_pagina=100",
            &cookie,
            None,
        ))
        .await
        .expect("listado inactivo");
    let body = cuerpo_json(res).await;
    let items = body["items"].as_array().expect("items de la página");
    assert!(items
        .iter()
        .any(|u| u["email"].as_str() == Some(email_inactivo.as_str())));

    // `activo=all` devuelve ambas poblaciones.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            "/api/usuarios?activo=all&por_pagina=100",
            &cookie,
            None,
        ))
        .await
        .expect("listado completo");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    let items = body["items"].as_array().expect("items de la página");
    assert!(items
        .iter()
        .any(|u| u["email"].as_str() == Some(email_activo.as_str())));
    assert!(items
        .iter()
        .any(|u| u["email"].as_str() == Some(email_inactivo.as_str())));

    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            "/api/usuarios?activo=todos",
            &cookie,
            None,
        ))
        .await
        .expect("filtro inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Límites de paginación: fuera de rango es 422, no 500.
    for query in [
        "pagina=0",
        "por_pagina=0",
        "por_pagina=101",
        "pagina=9223372036854775807&por_pagina=100",
    ] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "GET",
                &format!("/api/usuarios?{query}"),
                &cookie,
                None,
            ))
            .await
            .expect("paginación inválida");
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{query}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "validacion", "{query}");
    }

    let id = id_de(&pool, &email_activo).await;
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/usuarios/{id}"),
            &cookie,
            None,
        ))
        .await
        .expect("detalle");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["email"], email_activo);
    assert!(body.get("password_hash").is_none());

    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/usuarios/999999999", &cookie, None))
        .await
        .expect("detalle inexistente");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "recurso_no_encontrado");

    borrar_usuario(&pool, &email_activo).await;
    borrar_usuario(&pool, &email_inactivo).await;
    borrar_usuario(&pool, &admin_email).await;
}

#[tokio::test]
async fn actualizar_recalcula_familia_y_valida_unicidad() {
    let (app, pool, _correo) = app_de_test().await;
    let (admin_email, cookie) = crear_gestor_y_loguear(&app, &pool, "http-put-admin").await;
    let email = email_unico("http-put");
    let email_ocupado = email_unico("http-put-ocupado");
    crear_usuario(
        &pool,
        "Persona",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;
    crear_usuario(
        &pool,
        "Ocupado",
        &email_ocupado,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;
    let id = id_de(&pool, &email).await;

    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/usuarios/{id}"),
            &cookie,
            Some(json!({ "rol": "CAJERO", "nombre": "  Persona Renombrada  " })),
        ))
        .await
        .expect("actualización");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["rol"], "CAJERO");
    assert_eq!(body["familia_rol"], "OPERACION");
    assert_eq!(body["nombre"], "Persona Renombrada");

    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/usuarios/{id}"),
            &cookie,
            Some(json!({ "email": email_ocupado })),
        ))
        .await
        .expect("email ocupado");
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "email_duplicado");

    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/usuarios/{id}"),
            &cookie,
            Some(json!({ "rol": "SUPERVISOR" })),
        ))
        .await
        .expect("rol inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            "/api/usuarios/999999999",
            &cookie,
            Some(json!({ "nombre": "Nadie" })),
        ))
        .await
        .expect("usuario inexistente");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    borrar_usuario(&pool, &email).await;
    borrar_usuario(&pool, &email_ocupado).await;
    borrar_usuario(&pool, &admin_email).await;
}

#[tokio::test]
async fn desactivar_revoca_sesiones_reactiva_y_regenera_temporal() {
    let (app, pool, correo) = app_de_test().await;
    let (admin_email, cookie) = crear_gestor_y_loguear(&app, &pool, "http-baja-admin").await;
    let email = email_unico("http-baja");
    crear_usuario(
        &pool,
        "Baja Test",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;
    let id = id_de(&pool, &email).await;
    let (_, cookie_usuario) = intentar_login(&app, &email, "Password1").await;
    let cookie_usuario = cookie_usuario.expect("cookie del usuario");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/usuarios/{id}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .header(header::COOKIE, &cookie_usuario)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesión tras baja");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let (status, _) = intentar_login(&app, &email, "Password1").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/usuarios/{id}"),
            &cookie,
            None,
        ))
        .await
        .expect("detalle inactivo");
    let body = cuerpo_json(res).await;
    assert_eq!(body["activo"], false);

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/usuarios/{id}/reactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("reactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let (status, cookie_reactivado) = intentar_login(&app, &email, "Password1").await;
    assert_eq!(status, StatusCode::NO_CONTENT, "reactivado puede entrar");
    let cookie_reactivado = cookie_reactivado.expect("cookie tras reactivar");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/usuarios/{id}/password-temporal"),
            &cookie,
            None,
        ))
        .await
        .expect("regenerar temporal");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // La temporal nueva revoca la sesión vigente y reemplaza la anterior.
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/sesion")
                .header(header::COOKIE, &cookie_reactivado)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("sesión revocada");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let (status, _) = intentar_login(&app, &email, "Password1").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "la clave vieja no sirve");

    let temporal = temporal_enviada(&correo, &email);
    let (status, _) = intentar_login(&app, &email, &temporal).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "la temporal nueva sirve");

    for accion in ["desactivar", "reactivar", "password-temporal"] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/usuarios/999999999/{accion}"),
                &cookie,
                None,
            ))
            .await
            .expect("acción sobre usuario inexistente");
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{accion}");
    }

    borrar_usuario(&pool, &email).await;
    borrar_usuario(&pool, &admin_email).await;
}

/// Regresión del blocker de verificación: `split_once('@')` aceptaba emails con
/// más de una arroba, puntos consecutivos o puntos en los bordes (FR-008).
#[tokio::test]
async fn email_con_multiples_arrobas_es_422_en_alta_y_put() {
    let (app, pool, _correo) = app_de_test().await;
    let (admin_email, cookie) = crear_gestor_y_loguear(&app, &pool, "http-email-admin").await;

    let invalidos = [
        "a@b@c.com",
        "a@@b.com",
        "a@b..com",
        "a@.b.com",
        "a@b.com.",
        ".a@b.com",
        "a.@b.com",
        "a..b@c.com",
    ];
    // Limpia restos de una corrida RED previa para que la aserción de
    // persistencia mida solo lo que hizo este test.
    sqlx::query(r#"DELETE FROM usuarios WHERE email = ANY($1::text[])"#)
        .bind(&invalidos[..])
        .execute(&pool)
        .await
        .expect("limpieza de restos de corridas RED");

    let email = email_unico("http-email-valido");

    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/usuarios",
            &cookie,
            Some(json!({ "nombre": "Válida", "email": email, "rol": "OPERARIO" })),
        ))
        .await
        .expect("alta válida");
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = cuerpo_json(res).await;
    let id = body["id"].as_i64().expect("id del usuario creado") as i32;

    for invalido in invalidos {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                "/api/usuarios",
                &cookie,
                Some(json!({ "nombre": "Inválida", "email": invalido, "rol": "OPERARIO" })),
            ))
            .await
            .expect("alta con email inválido");
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "POST {invalido}"
        );
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "validacion", "POST {invalido}");

        let res = app
            .clone()
            .oneshot(solicitud(
                "PUT",
                &format!("/api/usuarios/{id}"),
                &cookie,
                Some(json!({ "email": invalido })),
            ))
            .await
            .expect("actualización con email inválido");
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "PUT {invalido}"
        );
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "validacion", "PUT {invalido}");
    }

    // Ninguno de los emails inválidos quedó persistido.
    let persistidos: i64 =
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM usuarios WHERE email = ANY($1::text[])"#)
            .bind(&invalidos[..])
            .fetch_one(&pool)
            .await
            .expect("conteo de emails inválidos");
    assert_eq!(persistidos, 0, "ningún email inválido debe persistirse");

    // El email válido del usuario sigue intacto tras los PUT rechazados.
    let guardado: String = sqlx::query_scalar("SELECT email FROM usuarios WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("email del usuario");
    assert_eq!(guardado, email);

    borrar_usuario(&pool, &email).await;
    borrar_usuario(&pool, &admin_email).await;
}
