//! Estado de la aplicación y armado del router (compartido por `main` y tests).

use crate::auth::reloj::Reloj;
use crate::config::AppConfig;
use crate::handlers;
use crate::mail::envio::EnvioCorreo;
use axum::{
    http::{header, Method},
    middleware,
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Estado compartido de la aplicación (Principio II: centralizado en un solo struct).
#[derive(Clone)]
pub struct AppEstado {
    pub pool: PgPool,
    pub config: AppConfig,
    pub reloj: Arc<dyn Reloj>,
    pub correo: Arc<dyn EnvioCorreo>,
}

/// Router completo bajo `/api` con CORS y rutas del Módulo 1.
pub fn construir_router(estado: AppEstado) -> Router {
    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);
    match estado.config.app_url.parse::<header::HeaderValue>() {
        Ok(origin) => cors = cors.allow_origin(origin),
        Err(_) => tracing::warn!(
            app_url = %estado.config.app_url,
            "APP_URL no es un origin válido; CORS sin allow_origin"
        ),
    }

    // Superficie autenticada: todo router que exija sesión vigente se suma acá
    // y queda detrás del gate de contraseña temporal (FR-008a). Las excepciones
    // son las rutas del router de auth (login, logout, sesión propia y cambio de
    // password) y `/api/salud`, que es pública.
    let autenticado = Router::new()
        .nest("/api/usuarios", handlers::usuarios::router())
        .nest("/api/categorias", handlers::categorias::router())
        .nest("/api/conteos", handlers::conteos::router())
        .nest("/api/productos", handlers::productos::router())
        .nest("/api/depositos", handlers::depositos::router())
        .nest("/api/ubicaciones", handlers::ubicaciones::router())
        .nest("/api/movimientos", handlers::movimientos::router())
        .nest("/api/transferencias", handlers::transferencias::router())
        .nest("/api/stock", handlers::stock::router())
        .layer(middleware::from_fn_with_state(
            estado.clone(),
            crate::auth::gate::exigir_password_cambiada,
        ));

    let mut router = Router::new()
        .route("/api/salud", get(handlers::salud))
        .nest("/api/auth", handlers::auth::router())
        .nest("/api/auditoria", handlers::auditoria::router())
        .merge(autenticado);

    // 000: Swagger UI + OpenAPI. Públicas y fuera del gate de sesión; se
    // apagan por configuración (`DOCS_ENABLED=false`, default en producción).
    if estado.config.docs_enabled {
        router = router.merge(crate::docs::router());
    }

    router.layer(cors).with_state(estado)
}
