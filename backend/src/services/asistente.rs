//! Asistente de inventario (017): proxy autenticado hacia el servicio LangChain.
//!
//! El backend no habla con el LLM: reenvía la consulta al servicio de `ASISTENTE_URL`
//! junto con la cookie de sesión del usuario, para que las tools llamen a esta misma
//! API con sus permisos (FR-066/FR-068). El asistente nunca escribe: a lo sumo
//! devuelve una `accion_propuesta` que el usuario confirma desde el frontend (FR-067).

use crate::error::AppError;
use crate::models::usuario::Usuario;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use utoipa::ToSchema;

/// FR-068: cupo por usuario dentro de la ventana de un minuto.
pub const MAX_CONSULTAS_POR_MINUTO: usize = 20;
/// FR-077: mensajes previos que se reenvían como contexto.
pub const MAX_HISTORIAL: usize = 10;
/// FR-068: tiempo máximo por consulta.
pub const TIMEOUT_POR_DEFECTO: Duration = Duration::from_secs(30);
const MAX_LARGO_MENSAJE: usize = 2000;
/// Si el servicio ni siquiera acepta la conexión en este plazo, está caído (503), no
/// lento (504). En Windows un puerto cerrado reintenta el SYN durante segundos.
const TIMEOUT_CONEXION: Duration = Duration::from_secs(1);
/// Header con el que viaja la cookie de sesión al servicio (nunca por el texto).
pub const HEADER_COOKIE_SESION: &str = "x-stockmante-cookie";

#[derive(Debug, Clone)]
pub struct ConfigAsistente {
    pub url: String,
    pub timeout: Duration,
}

impl ConfigAsistente {
    /// `ASISTENTE_URL` vacía o ausente = asistente deshabilitado (503, escenario 17.4).
    pub fn desde_entorno() -> Option<Self> {
        let url = std::env::var("ASISTENTE_URL").ok()?;
        let url = url.trim().trim_end_matches('/').to_owned();
        if url.is_empty() {
            return None;
        }
        let timeout = std::env::var("ASISTENTE_TIMEOUT_SEGUNDOS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|s| *s > 0)
            .map(Duration::from_secs)
            .unwrap_or(TIMEOUT_POR_DEFECTO);
        Some(Self { url, timeout })
    }
}

/// Ventana deslizante en memoria por usuario. Una consulta rechazada no consume cupo.
/// Se reinicia con el proceso y no se comparte entre réplicas (plan.md, riesgos).
pub struct LimiteConsultas {
    max: usize,
    ventana: chrono::Duration,
    registros: Mutex<HashMap<i32, VecDeque<DateTime<Utc>>>>,
}

impl LimiteConsultas {
    pub fn nuevo(max: usize, ventana: chrono::Duration) -> Self {
        Self { max, ventana, registros: Mutex::new(HashMap::new()) }
    }

    pub fn admitir(&self, usuario_id: i32, ahora: DateTime<Utc>) -> bool {
        // Un panic previo con el lock tomado no invalida los contadores: se siguen usando.
        let mut registros = self.registros.lock().unwrap_or_else(|e| e.into_inner());
        let marcas = registros.entry(usuario_id).or_default();
        while marcas.front().is_some_and(|t| ahora - *t >= self.ventana) {
            marcas.pop_front();
        }
        if marcas.len() >= self.max {
            return false;
        }
        marcas.push_back(ahora);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RolMensaje {
    Usuario,
    Asistente,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MensajeHistorial {
    pub rol: RolMensaje,
    pub texto: String,
}

/// FR-077: solo los `MAX_HISTORIAL` mensajes más recientes, en orden.
pub fn recortar_historial(mut historial: Vec<MensajeHistorial>) -> Vec<MensajeHistorial> {
    if historial.len() > MAX_HISTORIAL {
        historial.drain(..historial.len() - MAX_HISTORIAL);
    }
    historial
}

fn validar_mensaje(mensaje: &str) -> Result<String, AppError> {
    let mensaje = mensaje.trim();
    if mensaje.is_empty() {
        return Err(AppError::Validacion(
            "Escribí una pregunta o un pedido para el asistente.".into(),
        ));
    }
    if mensaje.chars().count() > MAX_LARGO_MENSAJE {
        return Err(AppError::Validacion(format!(
            "El mensaje puede tener hasta {MAX_LARGO_MENSAJE} caracteres. Acortalo y volvé a enviarlo."
        )));
    }
    Ok(mensaje.to_owned())
}

/// Propuesta de movimiento: mismo formato que acepta `POST /api/movimientos`, más los
/// nombres para mostrarla. Nunca se ejecuta desde el backend (FR-067).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccionPropuesta {
    #[schema(example = "MERMA")]
    pub tipo: String,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    #[schema(example = -2)]
    pub cantidad_delta: i32,
    #[schema(example = "MERMA")]
    pub motivo: String,
    pub observacion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RespuestaAsistente {
    pub respuesta: String,
    pub accion_propuesta: Option<AccionPropuesta>,
}

#[derive(Serialize)]
struct UsuarioServicio<'a> {
    id: i32,
    nombre: &'a str,
    familia_rol: &'a str,
}

#[derive(Serialize)]
struct PedidoServicio<'a> {
    mensaje: &'a str,
    historial: &'a [MensajeHistorial],
    usuario: UsuarioServicio<'a>,
}

#[derive(Deserialize)]
struct RespuestaServicio {
    respuesta: String,
    accion_propuesta: Option<AccionPropuesta>,
    #[serde(default)]
    tools: Vec<String>,
}

pub struct Asistente {
    config: Option<ConfigAsistente>,
    limite: LimiteConsultas,
    http: reqwest::Client,
}

impl Asistente {
    fn armar(config: Option<ConfigAsistente>) -> Self {
        Self {
            config,
            limite: LimiteConsultas::nuevo(MAX_CONSULTAS_POR_MINUTO, chrono::Duration::minutes(1)),
            http: cliente_http(),
        }
    }

    pub fn deshabilitado() -> Self {
        Self::armar(None)
    }

    pub fn desde_entorno() -> Self {
        let config = ConfigAsistente::desde_entorno();
        if config.is_none() {
            tracing::info!("ASISTENTE_URL no definida: el asistente responde 503");
        }
        Self::armar(config)
    }

    pub fn con_config(config: ConfigAsistente) -> Self {
        Self::armar(Some(config))
    }

    /// Orden: configuración (503) → validación (422) → cupo (429) → servicio (504/503).
    pub async fn consultar(
        &self,
        ahora: DateTime<Utc>,
        usuario: &Usuario,
        cookie_sesion: &str,
        mensaje: String,
        historial: Vec<MensajeHistorial>,
    ) -> Result<RespuestaAsistente, AppError> {
        let inicio = Instant::now();
        let resultado = self
            .consultar_sin_log(ahora, usuario, cookie_sesion, &mensaje, historial)
            .await;
        registrar_consulta(usuario.id, inicio.elapsed(), &resultado);
        resultado.map(|(respuesta, _)| respuesta)
    }

    async fn consultar_sin_log(
        &self,
        ahora: DateTime<Utc>,
        usuario: &Usuario,
        cookie_sesion: &str,
        mensaje: &str,
        historial: Vec<MensajeHistorial>,
    ) -> Result<(RespuestaAsistente, Vec<String>), AppError> {
        let Some(config) = &self.config else {
            return Err(AppError::AsistenteNoDisponible);
        };
        let mensaje = validar_mensaje(mensaje)?;
        if !self.limite.admitir(usuario.id, ahora) {
            return Err(AppError::AsistenteLimiteExcedido);
        }
        let historial = recortar_historial(historial);
        let pedido = PedidoServicio {
            mensaje: &mensaje,
            historial: &historial,
            usuario: UsuarioServicio {
                id: usuario.id,
                nombre: &usuario.nombre,
                familia_rol: &usuario.familia_rol,
            },
        };
        let respuesta = self
            .http
            .post(format!("{}/consulta", config.url))
            .header(HEADER_COOKIE_SESION, cookie_sesion)
            .timeout(config.timeout)
            .json(&pedido)
            .send()
            .await
            .map_err(error_de_red)?;
        // 503 del servicio = le falta configuración del LLM (17.4); otro error = caído.
        if respuesta.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            return Err(AppError::AsistenteNoDisponible);
        }
        if !respuesta.status().is_success() {
            tracing::warn!(status = %respuesta.status(), "el servicio de asistente respondió con error");
            return Err(AppError::AsistenteCaido);
        }
        let cuerpo: RespuestaServicio = respuesta.json().await.map_err(error_de_red)?;
        Ok((
            RespuestaAsistente {
                respuesta: cuerpo.respuesta,
                accion_propuesta: cuerpo.accion_propuesta,
            },
            cuerpo.tools,
        ))
    }
}

fn cliente_http() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(TIMEOUT_CONEXION)
        .build()
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "cliente HTTP del asistente sin connect_timeout");
            reqwest::Client::new()
        })
}

fn error_de_red(e: reqwest::Error) -> AppError {
    // Primero la conexión: un connect timeout también reporta `is_timeout()`.
    if e.is_connect() {
        tracing::warn!(error = %e, "el servicio de asistente no acepta conexiones");
        return AppError::AsistenteCaido;
    }
    if e.is_timeout() {
        return AppError::AsistenteTiempoAgotado;
    }
    // El detalle va al log; al cliente, un mensaje accionable (Principio III).
    tracing::warn!(error = %e, "no se pudo hablar con el servicio de asistente");
    AppError::AsistenteCaido
}

/// FR-078: solo metadatos. El texto del mensaje, del historial y de la respuesta no
/// llegan a esta función, así que no pueden filtrarse al log.
fn registrar_consulta(
    usuario_id: i32,
    duracion: Duration,
    resultado: &Result<(RespuestaAsistente, Vec<String>), AppError>,
) {
    let (codigo, tools, hubo_propuesta) = match resultado {
        Ok((respuesta, tools)) => (200, tools.join(","), respuesta.accion_propuesta.is_some()),
        Err(e) => (e.status().as_u16(), String::new(), false),
    };
    tracing::info!(
        usuario_id,
        duracion_ms = duracion.as_millis() as u64,
        tools = %tools,
        hubo_propuesta,
        codigo,
        "consulta al asistente"
    );
}
