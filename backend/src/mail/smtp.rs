//! Transporte SMTP real de la contraseña temporal (T029).
//!
//! `lettre` async sobre rustls. Se construye sólo si `SMTP_URL` está definida;
//! en desarrollo el default sigue siendo `CorreoDev`. Cualquier fallo de
//! conexión/autenticación se devuelve como `Err(String)` y el llamador
//! (`services::usuarios`) revierte el alta (fail-closed).

use crate::mail::envio::EnvioCorreo;
use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

pub struct CorreoSmtp {
    transporte: AsyncSmtpTransport<Tokio1Executor>,
    remitente: Mailbox,
}

impl CorreoSmtp {
    /// `smtp_url` en formato `smtps://usuario:password@host:465` (o `smtp://`
    /// para STARTTLS en 587). Falla si la URL o `EMAIL_FROM` no son válidos.
    pub fn desde_url(smtp_url: &str, remitente: &str) -> Result<Self, String> {
        let remitente: Mailbox = remitente
            .trim()
            .parse()
            .map_err(|e| format!("EMAIL_FROM inválido: {e}"))?;
        let transporte = AsyncSmtpTransport::<Tokio1Executor>::from_url(smtp_url.trim())
            .map_err(|e| format!("SMTP_URL inválida: {e}"))?
            .build();
        Ok(Self {
            transporte,
            remitente,
        })
    }
}

#[async_trait]
impl EnvioCorreo for CorreoSmtp {
    async fn enviar_password_temporal(
        &self,
        destino: &str,
        nombre: &str,
        password_temporal: &str,
    ) -> Result<(), String> {
        let destinatario: Mailbox = destino
            .parse()
            .map_err(|e| format!("destino inválido ({destino}): {e}"))?;
        let cuerpo = format!(
            "Hola {nombre}:\n\n\
             Tu contraseña temporal de Stockmante es: {password_temporal}\n\n\
             Debés cambiarla la primera vez que inicies sesión.\n\
             Si no esperabas este correo, avisá al administrador.\n"
        );
        let mensaje = Message::builder()
            .from(self.remitente.clone())
            .to(destinatario)
            .subject("Tu contraseña temporal de Stockmante")
            .header(ContentType::TEXT_PLAIN)
            .body(cuerpo)
            .map_err(|e| format!("no se pudo armar el correo: {e}"))?;
        self.transporte
            .send(mensaje)
            .await
            .map(|_| ())
            .map_err(|e| format!("envío SMTP falló: {e}"))
    }

    async fn enviar_verificacion(
        &self,
        destino: &str,
        nombre: &str,
        token: &str,
        base_url: &str,
    ) -> Result<(), String> {
        let destinatario: Mailbox = destino
            .parse()
            .map_err(|e| format!("destino inválido ({destino}): {e}"))?;
        let enlace = format!("{}/verificar-correo?token={}", base_url.trim_end_matches('/'), token);
        let cuerpo = format!(
            "Hola {nombre}:\n\n\
             Verificá tu correo para activar tu cuenta de Stockmante:\n{enlace}\n\n\
             El enlace expira en 24 horas.\n"
        );
        let mensaje = Message::builder()
            .from(self.remitente.clone())
            .to(destinatario)
            .subject("Verificá tu correo de Stockmante")
            .header(ContentType::TEXT_PLAIN)
            .body(cuerpo)
            .map_err(|e| format!("no se pudo armar el correo: {e}"))?;
        self.transporte
            .send(mensaje)
            .await
            .map(|_| ())
            .map_err(|e| format!("envío SMTP falló: {e}"))
    }
}
