//! Helper compartido de tests que necesitan la BD de test (`stockmante_test`).
//! Cada test usa emails únicos (uuid) para poder correr en paralelo.
//!
//! Cada binario de test incluye este módulo completo y usa solo parte de los
//! helpers, por eso se silencia `dead_code` acá.
#![allow(dead_code)]

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use stockmante_backend::app::{construir_router, AppEstado};
use stockmante_backend::auth::password::hash_password;
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::config::AppConfig;
use stockmante_backend::mail::envio::CorreoDev;
use tower::ServiceExt;

/// Pool hacia `stockmante_test`: usa `DATABASE_URL_TEST` o deriva de `DATABASE_URL`.
pub async fn pool_de_test() -> PgPool {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        let base = std::env::var("DATABASE_URL")
            .expect("definí DATABASE_URL o DATABASE_URL_TEST para tests de BD");
        let trimmed = base.trim_end_matches('/');
        trimmed
            .rsplit_once("/stockmante")
            .map(|(prefix, _)| format!("{prefix}/stockmante_test"))
            .expect("DATABASE_URL debe apuntar a la base stockmante")
    });
    PgPool::connect(&url)
        .await
        .expect("no se pudo conectar a stockmante_test (¿está corriendo postgres?)")
}

pub fn email_unico(etiqueta: &str) -> String {
    format!("{etiqueta}-{}@test.local", uuid::Uuid::new_v4())
}

/// Router real armado sobre el pool recibido (mismo `AppEstado` que `main`).
pub fn app_con(pool: PgPool) -> axum::Router {
    let _ = dotenvy::dotenv();
    let config = AppConfig::desde_entorno().expect("config de test");
    let estado = AppEstado {
        pool,
        config,
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(CorreoDev::new()),
    };
    construir_router(estado)
}

/// Router + pool + correo stub, listo para `oneshot`.
pub async fn app_de_test() -> (axum::Router, PgPool, Arc<CorreoDev>) {
    let pool = pool_de_test().await;
    (app_con(pool.clone()), pool, Arc::new(CorreoDev::new()))
}

/// Request con cookie de sesión y body JSON opcional.
pub fn solicitud(metodo: &str, uri: &str, cookie: &str, body: Option<Value>) -> Request<Body> {
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

/// Request sin cookie (para probar 401).
pub fn solicitud_anonima(metodo: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(metodo)
        .uri(uri)
        .body(Body::empty())
        .expect("request anónima")
}

pub async fn cuerpo_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }
}

pub fn cookie_de_sesion(res: &axum::response::Response) -> String {
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

pub fn login_request(email: &str, password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "email": email, "password": password }).to_string(),
        ))
        .expect("request de login")
}

/// Inserta un usuario de prueba y devuelve su id. `password` en claro.
#[allow(clippy::too_many_arguments)]
pub async fn crear_usuario(
    pool: &PgPool,
    nombre: &str,
    email: &str,
    password: &str,
    rol: &str,
    familia_rol: &str,
    activo: bool,
    password_temporal: bool,
) -> i32 {
    let hash = hash_password(password).expect("hash de password de test");
    sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO usuarios (nombre, email, password_hash, password_temporal, rol, familia_rol, activo)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(nombre)
    .bind(email)
    .bind(&hash)
    .bind(password_temporal)
    .bind(rol)
    .bind(familia_rol)
    .bind(activo)
    .fetch_one(pool)
    .await
    .expect("insert de usuario de test")
}

/// Borra el usuario de prueba (las sesiones caen por CASCADE).
pub async fn borrar_usuario(pool: &PgPool, email: &str) {
    sqlx::query(r#"DELETE FROM usuarios WHERE email = $1"#)
        .bind(email)
        .execute(pool)
        .await
        .expect("borrado de usuario de test");
}

/// Crea un usuario con la familia indicada y devuelve `(email, cookie)`.
pub async fn sesion_con_familia(pool: &PgPool, rol: &str, familia_rol: &str) -> (String, String) {
    let email = email_unico("sesion");
    crear_usuario(
        pool,
        "Sesión Test",
        &email,
        "Password1",
        rol,
        familia_rol,
        true,
        false,
    )
    .await;
    let app = app_con(pool.clone());
    let res = app
        .oneshot(login_request(&email, "Password1"))
        .await
        .expect("login de sesión de test");
    assert_eq!(
        res.status(),
        StatusCode::NO_CONTENT,
        "login de la sesión de test ({familia_rol})"
    );
    (email, cookie_de_sesion(&res))
}

/// Cookie de una sesión con familia GESTION.
pub async fn sesion_gestion(pool: &PgPool) -> String {
    sesion_con_familia(pool, "ADMIN", "GESTION").await.1
}

/// Cookie de una sesión con familia OPERACION.
pub async fn sesion_operacion(pool: &PgPool) -> String {
    sesion_con_familia(pool, "CAJERO", "OPERACION").await.1
}

/// SKU válido y único (3–30, `^[A-Z0-9][A-Z0-9-]{2,29}$`).
pub fn sku_unico(etiqueta: &str) -> String {
    let hex = uuid::Uuid::new_v4().simple().to_string().to_uppercase();
    let mut prefijo: String = etiqueta
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .take(6)
        .collect();
    if prefijo.is_empty() {
        prefijo.push_str("SKU");
    }
    let disponibles = 30usize.saturating_sub(prefijo.len() + 1);
    format!("{prefijo}-{}", &hex[..disponibles])
}

/// Código de barras válido y único (8–64 alfanuméricos).
pub fn codigo_barras_unico() -> String {
    uuid::Uuid::new_v4().simple().to_string().to_uppercase()
}

/// Nombre de categoría válido y único (1–80).
pub fn nombre_categoria_unico(etiqueta: &str) -> String {
    let sufijo = &uuid::Uuid::new_v4().simple().to_string()[..8];
    let etiqueta: String = etiqueta.chars().take(60).collect();
    if etiqueta.is_empty() {
        format!("Categoría {sufijo}")
    } else {
        format!("{etiqueta} {sufijo}")
    }
}
