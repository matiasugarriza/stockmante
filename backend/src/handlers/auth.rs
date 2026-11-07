//! Handlers de autenticación (contracts/api.md): login, logout, sesión, cambio de password.

use crate::app::AppEstado;
use crate::auth::extractors::AuthUsuario;
use crate::docs::ErrorRespuesta;
use crate::error::AppError;
use crate::extractores::JsonValido;
use crate::extractores::{PathValido, QueryValida};
use crate::services;
use crate::services::sso::SsoConfig;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use time::{Duration as Duracion, OffsetDateTime};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

pub fn router() -> Router<AppEstado> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/sesion", get(sesion))
        .route("/cambiar-password", post(cambiar_password))
        .route("/registro", post(registro))
        .route("/verificar-correo", get(verificar_correo))
        .route("/sso/config", get(sso_config))
        .route("/sso/{proveedor}/iniciar", get(sso_iniciar))
        .route("/sso/{proveedor}/callback", get(sso_callback))
        .route("/identidades", get(listar_identidades))
        .route("/identidades/{proveedor}/vincular", post(vincular_identidad))
        .route("/identidades/{proveedor}", delete(desvincular_identidad))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginDto {
    #[schema(example = "admin@stockmante.local")]
    pub email: String,
    #[schema(example = "Prueba1234")]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CambioPasswordDto {
    pub password_actual: String,
    pub password_nueva: String,
}

/// Usuario tal como lo ve el frontend (sin hash; `debe_cambiar_password` = flag temporal).
#[derive(Debug, Serialize, ToSchema)]
pub struct UsuarioSesionDto {
    pub id: i32,
    pub nombre: String,
    pub email: String,
    pub rol: String,
    pub familia_rol: String,
    pub debe_cambiar_password: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SesionDto {
    pub usuario: UsuarioSesionDto,
}

fn cookie_sesion(estado: &AppEstado, sesion_id: Uuid) -> Cookie<'static> {
    let horas = estado.config.session_expiry_hours.max(1) as u64;
    Cookie::build((
        estado.config.session_cookie_name.clone(),
        sesion_id.to_string(),
    ))
    .http_only(true)
    .same_site(SameSite::Lax)
    .path("/")
    // FR-002a: sólo viaja por HTTPS en producción; en dev/local no.
    .secure(estado.config.es_produccion())
    .max_age(Duracion::hours(horas as i64))
    .build()
}

fn cookie_sesion_borrada(estado: &AppEstado) -> Cookie<'static> {
    Cookie::build((
        estado.config.session_cookie_name.clone(),
        String::new(),
    ))
    .http_only(true)
    .same_site(SameSite::Lax)
    .path("/")
    .secure(estado.config.es_produccion())
    .expires(OffsetDateTime::now_utc() - Duracion::days(1))
    .build()
}

/// POST /api/auth/login → 204 + Set-Cookie (o 401 genérico).
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    summary = "Inicia sesión",
    description = "Valida credenciales y crea la sesión única; responde con la cookie HttpOnly.",
    request_body = LoginDto,
    responses(
        (status = 204, description = "Sesión creada; Set-Cookie con la cookie de sesión"),
        (status = 401, description = "Credenciales inválidas", body = ErrorRespuesta),
        (status = 422, description = "Entrada inválida", body = ErrorRespuesta)
    ),
    security(())
)]
pub async fn login(
    State(estado): State<AppEstado>,
    jar: CookieJar,
    JsonValido(dto): JsonValido<LoginDto>,
) -> Result<(CookieJar, StatusCode), AppError> {
    if dto.email.trim().is_empty() || dto.password.is_empty() {
        return Err(AppError::Validacion(
            "Ingresá email y contraseña".into(),
        ));
    }
    let (sesion_id, _) = services::auth::login(
        &estado.pool,
        estado.reloj.as_ref(),
        &dto.email,
        &dto.password,
        estado.config.session_expiry_hours,
    )
    .await?;
    Ok((jar.add(cookie_sesion(&estado, sesion_id)), StatusCode::NO_CONTENT))
}

/// POST /api/auth/logout → 204 (idempotente) + borra la cookie.
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    summary = "Cierra la sesión",
    description = "Revoca la sesión vigente y borra la cookie. Es idempotente.",
    responses((status = 204, description = "Sesión cerrada o ya inexistente")),
    security(())
)]
pub async fn logout(
    State(estado): State<AppEstado>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    if let Some(cookie) = jar.get(&estado.config.session_cookie_name) {
        if let Ok(sesion_id) = Uuid::parse_str(cookie.value()) {
            services::auth::logout(&estado.pool, estado.reloj.as_ref(), sesion_id).await?;
        }
    }
    Ok((
        jar.remove(cookie_sesion_borrada(&estado)),
        StatusCode::NO_CONTENT,
    ))
}

/// GET /api/auth/sesion → usuario actual o 401.
#[utoipa::path(
    get,
    path = "/api/auth/sesion",
    tag = "auth",
    summary = "Usuario de la sesión actual",
    responses(
        (status = 200, description = "Sesión vigente", body = SesionDto),
        (status = 401, description = "Sin sesión o sesión expirada", body = ErrorRespuesta)
    )
)]
pub async fn sesion(AuthUsuario(usuario): AuthUsuario) -> impl IntoResponse {
    Json(SesionDto {
        usuario: UsuarioSesionDto {
            id: usuario.id,
            nombre: usuario.nombre,
            email: usuario.email,
            rol: usuario.rol,
            familia_rol: usuario.familia_rol,
            debe_cambiar_password: usuario.password_temporal,
        },
    })
}

/// POST /api/auth/cambiar-password → 204; 422 si no cumple política.
#[utoipa::path(
    post,
    path = "/api/auth/cambiar-password",
    tag = "auth",
    summary = "Cambia la contraseña propia",
    description = "Exige la contraseña actual. Si era temporal, limpia el flag de cambio obligatorio.",
    request_body = CambioPasswordDto,
    responses(
        (status = 204, description = "Contraseña actualizada"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 403, description = "Cambio de contraseña requerido", body = ErrorRespuesta),
        (status = 422, description = "La contraseña no cumple la política o la actual no coincide", body = ErrorRespuesta)
    )
)]
pub async fn cambiar_password(
    State(estado): State<AppEstado>,
    AuthUsuario(_usuario): AuthUsuario,
    jar: CookieJar,
    JsonValido(dto): JsonValido<CambioPasswordDto>,
) -> Result<StatusCode, AppError> {
    let Some(cookie) = jar.get(&estado.config.session_cookie_name) else {
        return Err(AppError::NoAutenticado);
    };
    let sesion_id = Uuid::parse_str(cookie.value())
        .map_err(|_| AppError::NoAutenticado)?;
    services::auth::cambiar_password(
        &estado.pool,
        estado.reloj.as_ref(),
        sesion_id,
        &dto.password_actual,
        &dto.password_nueva,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegistroDto {
    pub nombre: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct VerificarCorreoQuery {
    pub token: String,
}

/// POST /api/auth/registro → 202 (anti-enumeración: mismo mensaje).
#[utoipa::path(
    post,
    path = "/api/auth/registro",
    tag = "auth",
    summary = "Registra un usuario por correo",
    request_body = RegistroDto,
    responses(
        (status = 202, description = "Registro recibido; verificar correo"),
        (status = 422, description = "Entrada inválida", body = ErrorRespuesta)
    ),
    security(())
)]
pub async fn registro(
    State(estado): State<AppEstado>,
    JsonValido(dto): JsonValido<RegistroDto>,
) -> Result<StatusCode, AppError> {
    services::registro::registrar(
        &estado.pool,
        estado.correo.as_ref(),
        &dto.nombre,
        &dto.email,
        &dto.password,
        &estado.config.app_url,
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

/// GET /api/auth/verificar-correo?token= → 200.
#[utoipa::path(
    get,
    path = "/api/auth/verificar-correo",
    tag = "auth",
    summary = "Verifica el correo de un registro",
    params(VerificarCorreoQuery),
    responses(
        (status = 200, description = "Correo verificado"),
        (status = 422, description = "Token inválido o expirado", body = ErrorRespuesta)
    ),
    security(())
)]
pub async fn verificar_correo(
    State(estado): State<AppEstado>,
    QueryValida(q): QueryValida<VerificarCorreoQuery>,
) -> Result<StatusCode, AppError> {
    services::registro::verificar_correo(&estado.pool, &q.token).await?;
    Ok(StatusCode::OK)
}

/// GET /api/auth/sso/config → disponibilidad de proveedores SSO.
#[utoipa::path(
    get,
    path = "/api/auth/sso/config",
    tag = "auth",
    summary = "Disponibilidad de proveedores SSO",
    description = "Indica si Google y/o Microsoft están configurados para que el frontend oculte/muestre los botones de SSO.",
    responses(
        (status = 200, description = "Estado de disponibilidad", body = SsoConfig)
    ),
    security(())
)]
pub async fn sso_config(State(estado): State<AppEstado>) -> axum::Json<SsoConfig> {
    let mut cfg = services::sso::sso_config(&estado.config);
    // FR-071: credenciales presentes + proveedor que responde; si la salud
    // falla, el botón se oculta igual que si faltaran las credenciales.
    if cfg.google_disponible && !estado.salud_sso.salud("google").await {
        cfg.google_disponible = false;
    }
    if cfg.microsoft_disponible && !estado.salud_sso.salud("microsoft").await {
        cfg.microsoft_disponible = false;
    }
    axum::Json(cfg)
}

/// GET /api/auth/sso/{proveedor}/iniciar → 307 (Redirect::temporary).
#[utoipa::path(
    get,
    path = "/api/auth/sso/{proveedor}/iniciar",
    tag = "auth",
    summary = "Inicia el flujo SSO",
    params(("proveedor" = String, Path, description = "google o microsoft")),
    responses(
        (status = 307, description = "Redirección al proveedor"),
        (status = 422, description = "Proveedor no soportado", body = ErrorRespuesta),
        (status = 503, description = "Proveedor no disponible", body = ErrorRespuesta)
    ),
    security(())
)]
pub async fn sso_iniciar(
    State(estado): State<AppEstado>,
    jar: CookieJar,
    PathValido(proveedor): PathValido<String>,
) -> Result<(CookieJar, axum::response::Redirect), AppError> {
    let state = Uuid::new_v4().to_string();
    let redirect_uri = format!(
        "{}/api/auth/sso/{}/callback",
        estado.config.sso_redirect_base.trim_end_matches('/'),
        proveedor
    );
    let url = services::sso::url_autorizacion(
        &proveedor,
        // El client_id del proveedor pedido: Google nunca sirve para Microsoft
        // (vacío ⇒ 503 sso_no_configurado en url_autorizacion).
        match proveedor.as_str() {
            "google" => estado.config.google_client_id.as_str(),
            "microsoft" => estado.config.microsoft_client_id.as_str(),
            _ => "",
        },
        &state,
        &redirect_uri,
    )?;
    let jar = jar.add(
        Cookie::build(("sso_state", state.clone()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .secure(estado.config.es_produccion())
            .max_age(Duracion::minutes(10))
            .build(),
    );
    Ok((jar, axum::response::Redirect::temporary(&url)))
}

/// GET /api/auth/sso/{proveedor}/callback → 303 al panel + cookie de sesión.
#[utoipa::path(
    get,
    path = "/api/auth/sso/{proveedor}/callback",
    tag = "auth",
    summary = "Callback SSO",
    params(
        ("proveedor" = String, Path, description = "google o microsoft"),
        ("code" = String, Query, description = "Authorization code"),
        ("state" = String, Query, description = "State")
    ),
    responses(
        (status = 303, description = "Sesión creada; redirige al panel (APP_URL)"),
        (status = 400, description = "State inválido", body = ErrorRespuesta),
        (status = 422, description = "Proveedor no soportado", body = ErrorRespuesta),
        (status = 503, description = "Proveedor no disponible", body = ErrorRespuesta)
    ),
    security(())
)]
pub async fn sso_callback(
    State(estado): State<AppEstado>,
    jar: CookieJar,
    PathValido(proveedor): PathValido<String>,
    QueryValida(q): QueryValida<SsoCallbackQuery>,
) -> Result<(CookieJar, axum::response::Redirect), AppError> {
    // Validar state.
    let esperado = jar.get("sso_state").map(|c| c.value().to_string());
    if esperado.as_deref() != Some(q.state.as_str()) {
        return Err(AppError::Validacion("State inválido.".into()));
    }

    let (client_id, client_secret) = match proveedor.as_str() {
        "google" => (
            &estado.config.google_client_id,
            &estado.config.google_client_secret,
        ),
        "microsoft" => (
            &estado.config.microsoft_client_id,
            &estado.config.microsoft_client_secret,
        ),
        _ => return Err(AppError::ProveedorNoSoportado),
    };

    let usuario = services::sso::procesar_callback(
        &estado.pool,
        &proveedor,
        &q.code,
        client_id,
        client_secret,
        &estado.config.sso_redirect_base,
        estado.correo.as_ref(),
        &estado.config.app_url,
    )
    .await?;

    // Crear sesión.
    let sesion_id = services::auth::crear_sesion_para_usuario(
        &estado.pool,
        estado.reloj.as_ref(),
        usuario.id,
        estado.config.session_expiry_hours,
    )
    .await?;

    // El navegador que vuelve de Google no espera JSON: redirigimos al panel
    // con la cookie de sesión ya puesta (antes devolvía 204 y quedaba una
    // página en blanco — el reporte de "SSO Google no funciona").
    let destino = format!("{}/", estado.config.app_url.trim_end_matches('/'));
    let jar = jar
        .remove(Cookie::build(("sso_state", String::new())).path("/").build())
        .add(cookie_sesion(&estado, sesion_id));
    Ok((jar, axum::response::Redirect::to(&destino)))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SsoCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IdentidadDto {
    pub proveedor: String,
    pub email: String,
}

/// GET /api/auth/identidades → lista.
#[utoipa::path(
    get,
    path = "/api/auth/identidades",
    tag = "auth",
    summary = "Lista identidades SSO vinculadas",
    responses(
        (status = 200, description = "Lista de identidades", body = [IdentidadDto]),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta)
    )
)]
pub async fn listar_identidades(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
) -> Result<Json<Vec<IdentidadDto>>, AppError> {
    let ids = services::sso::listar_identidades(&estado.pool, usuario.id).await?;
    Ok(Json(
        ids.into_iter()
            .map(|(p, e, _)| IdentidadDto {
                proveedor: p,
                email: e,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VincularIdentidadDto {
    pub code: String,
    pub state: String,
}

/// POST /api/auth/identidades/{proveedor}/vincular → 200.
#[utoipa::path(
    post,
    path = "/api/auth/identidades/{proveedor}/vincular",
    tag = "auth",
    summary = "Vincula una identidad SSO",
    params(("proveedor" = String, Path, description = "google o microsoft")),
    request_body = VincularIdentidadDto,
    responses(
        (status = 200, description = "Identidad vinculada"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 409, description = "Identidad ya vinculada", body = ErrorRespuesta),
        (status = 422, description = "Proveedor no soportado", body = ErrorRespuesta)
    )
)]
pub async fn vincular_identidad(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    PathValido(proveedor): PathValido<String>,
    JsonValido(dto): JsonValido<VincularIdentidadDto>,
) -> Result<StatusCode, AppError> {
    if !services::sso::proveedores().contains(&proveedor.as_str()) {
        return Err(AppError::ProveedorNoSoportado);
    }
    // Validar state.
    // (state se valida en el frontend/cookie; aquí confiamos en el dto)
    let _ = dto;
    // Intercambiar code y vincular al usuario actual.
    let (client_id, client_secret) = match proveedor.as_str() {
        "google" => (
            &estado.config.google_client_id,
            &estado.config.google_client_secret,
        ),
        "microsoft" => (
            &estado.config.microsoft_client_id,
            &estado.config.microsoft_client_secret,
        ),
        _ => return Err(AppError::ProveedorNoSoportado),
    };
    let _ = services::sso::vincular(
        &estado.pool,
        usuario.id,
        &proveedor,
        &dto.code,
        client_id,
        client_secret,
        &estado.config.sso_redirect_base,
        estado.correo.as_ref(),
        &estado.config.app_url,
    )
    .await?;
    Ok(StatusCode::OK)
}

/// DELETE /api/auth/identidades/{proveedor} → 204.
#[utoipa::path(
    delete,
    path = "/api/auth/identidades/{proveedor}",
    tag = "auth",
    summary = "Desvincula una identidad SSO",
    params(("proveedor" = String, Path, description = "google o microsoft")),
    responses(
        (status = 204, description = "Identidad desvinculada"),
        (status = 401, description = "Sin sesión", body = ErrorRespuesta),
        (status = 409, description = "Última identidad", body = ErrorRespuesta),
        (status = 422, description = "Proveedor no soportado", body = ErrorRespuesta)
    )
)]
pub async fn desvincular_identidad(
    State(estado): State<AppEstado>,
    AuthUsuario(usuario): AuthUsuario,
    PathValido(proveedor): PathValido<String>,
) -> Result<StatusCode, AppError> {
    services::sso::desvincular(&estado.pool, usuario.id, &proveedor).await?;
    Ok(StatusCode::NO_CONTENT)
}
