//! Envío de correos con inyección (Principio IV): lógica de negocio no ve red/SMTP.

use async_trait::async_trait;
use std::sync::Mutex;

#[async_trait]
pub trait EnvioCorreo: Send + Sync {
    async fn enviar_password_temporal(
        &self,
        destino: &str,
        nombre: &str,
        password_temporal: &str,
    ) -> Result<(), String>;
}

/// Implementación de desarrollo: no manda correo, registra el intento en memoria
/// (los tests/entorno dev leen `bandeja()`).
#[derive(Default)]
pub struct CorreoDev {
    enviados: Mutex<Vec<(String, String, String)>>,
    /// Solo el modo desarrollo imprime la temporal en consola.
    modo_desarrollo: bool,
}

impl CorreoDev {
    /// Stub silencioso (tests y producción sin SMTP): nunca expone credenciales.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stub de desarrollo local: imprime la temporal en consola para poder
    /// iniciar sesión sin SMTP.
    pub fn para_desarrollo() -> Self {
        Self {
            modo_desarrollo: true,
            ..Self::default()
        }
    }

    pub fn bandeja(&self) -> Vec<(String, String, String)> {
        self.enviados
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Texto que se imprime en consola cuando el stub está en modo desarrollo.
    /// `None` en el stub silencioso: las credenciales no se muestran ni en logs.
    pub fn aviso_credenciales(
        &self,
        destino: &str,
        nombre: &str,
        password_temporal: &str,
    ) -> Option<String> {
        self.modo_desarrollo.then(|| {
            format!(
                "[correo dev] contraseña temporal de {nombre} <{destino}>: {password_temporal}"
            )
        })
    }
}

#[async_trait]
impl EnvioCorreo for CorreoDev {
    async fn enviar_password_temporal(
        &self,
        destino: &str,
        nombre: &str,
        password_temporal: &str,
    ) -> Result<(), String> {
        if let Some(aviso) = self.aviso_credenciales(destino, nombre, password_temporal) {
            println!("{aviso}");
        }
        tracing::info!(destino, nombre, "correo dev: contraseña temporal (no se envía red)");
        if let Ok(mut v) = self.enviados.lock() {
            v.push((
                destino.to_string(),
                nombre.to_string(),
                password_temporal.to_string(),
            ));
        }
        Ok(())
    }
}
