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
}

impl CorreoDev {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bandeja(&self) -> Vec<(String, String, String)> {
        self.enviados
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default()
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
