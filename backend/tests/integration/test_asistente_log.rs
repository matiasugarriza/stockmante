//! 017 — FR-078: el log de cada consulta tiene solo metadatos, nunca texto.
//!
//! Binario propio: el suscriptor de captura es por hilo (`set_default`) y otros tests en
//! paralelo podrían cachear el interés de los callsites sin él.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use common::{cuerpo_json, pool_de_test, sesion_con_familia, solicitud};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use stockmante_backend::app::{construir_router, AppEstado};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::config::AppConfig;
use stockmante_backend::mail::envio::CorreoDev;
use stockmante_backend::services::asistente::{Asistente, ConfigAsistente};
use stockmante_backend::services::sso::SondaSso;
use tower::ServiceExt;

const URI: &str = "/api/asistente/consulta";

/// Lo que el mock recibió: header de sesión reenviado + body.
#[derive(Clone, Debug)]
struct Recibido {
    cookie: Option<String>,
    body: Value,
}

#[derive(Clone)]
struct Mock {
    recibidos: Arc<Mutex<Vec<Recibido>>>,
    demora: Duration,
    respuesta: Value,
}

async fn atender(
    State(mock): State<Mock>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let cookie = headers
        .get("x-stockmante-cookie")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    if let Ok(mut lista) = mock.recibidos.lock() {
        lista.push(Recibido { cookie, body });
    }
    tokio::time::sleep(mock.demora).await;
    (StatusCode::OK, Json(mock.respuesta.clone()))
}

/// Levanta el mock en un puerto libre y devuelve `(url, recibidos)`.
async fn levantar_mock(demora: Duration, respuesta: Value) -> (String, Arc<Mutex<Vec<Recibido>>>) {
    let recibidos = Arc::new(Mutex::new(Vec::new()));
    let mock = Mock { recibidos: recibidos.clone(), demora, respuesta };
    let app = Router::new().route("/consulta", post(atender)).with_state(mock);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind del mock");
    let addr = listener.local_addr().expect("addr del mock");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), recibidos)
}

fn respuesta_simple() -> Value {
    json!({ "respuesta": "Hay 12 unidades en Principal.", "accion_propuesta": null, "tools": ["consultar_stock"] })
}

fn app_con_asistente(pool: PgPool, asistente: Asistente) -> Router {
    let _ = dotenvy::dotenv();
    let config = AppConfig::desde_entorno().expect("config de test");
    construir_router(AppEstado {
        pool,
        config,
        reloj: Arc::new(RelojSistema),
        correo: Arc::new(CorreoDev::new()),
        salud_sso: Arc::new(SondaSso::fija(true, true)),
        asistente: Arc::new(asistente),
    })
}

fn configurado(url: &str, timeout: Duration) -> Asistente {
    Asistente::con_config(ConfigAsistente { url: url.to_owned(), timeout })
}

async fn consultar(app: &Router, cookie: &str, body: Value) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud("POST", URI, cookie, Some(body)))
        .await
        .expect("request al asistente");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, datos: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut b) = self.0.lock() {
            b.extend_from_slice(datos);
        }
        Ok(datos.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn el_log_tiene_metadatos_y_nunca_el_texto() {
    let buffer = Buffer::default();
    let escritor = buffer.clone();
    let suscriptor = tracing_subscriber::fmt()
        .with_writer(move || escritor.clone())
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .finish();
    let _guardia = tracing::subscriber::set_default(suscriptor);

    let pool = pool_de_test().await;
    let (_, cookie) = sesion_con_familia(&pool, "CAJERO", "OPERACION").await;
    let (url, _) = levantar_mock(Duration::ZERO, respuesta_simple()).await;
    let app = app_con_asistente(pool, configurado(&url, Duration::from_secs(2)));
    let (status, _) = consultar(
        &app,
        &cookie,
        json!({
            "mensaje": "SECRETO-MENSAJE yerba",
            "historial": [{ "rol": "usuario", "texto": "SECRETO-HISTORIAL" }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let log = String::from_utf8(buffer.0.lock().expect("lock").clone()).expect("utf8");
    let linea = log
        .lines()
        .find(|l| l.contains("consulta al asistente"))
        .unwrap_or_else(|| panic!("debe haber una línea de log de la consulta; log: {log}"));
    assert!(linea.contains("usuario_id="), "{linea}");
    assert!(linea.contains("duracion_ms="), "{linea}");
    assert!(linea.contains("codigo=200"), "{linea}");
    assert!(linea.contains("consultar_stock"), "tools invocadas: {linea}");
    assert!(linea.contains("hubo_propuesta=false"), "{linea}");
    assert!(!log.contains("SECRETO-MENSAJE"), "el texto del mensaje no se loguea");
    assert!(!log.contains("SECRETO-HISTORIAL"), "el historial no se loguea");
    assert!(!log.contains("Hay 12 unidades"), "la respuesta no se loguea");
}
