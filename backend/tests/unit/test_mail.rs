use stockmante_backend::mail::{
    envio::{CorreoDev, EnvioCorreo},
    smtp::CorreoSmtp,
};

#[tokio::test]
async fn correo_dev_registra_intentos_sin_red() {
    let correo = CorreoDev::new();
    correo
        .enviar_password_temporal("ana@mail.com", "Ana", "Temp123AbcDef")
        .await
        .expect("stub no falla");
    let bandeja = correo.bandeja();
    assert_eq!(bandeja.len(), 1);
    assert_eq!(bandeja[0].0, "ana@mail.com");
    assert_eq!(bandeja[0].2, "Temp123AbcDef");
}

/// El aviso con la temporal es una herramienta de desarrollo: el stub por
/// defecto (tests y producción sin SMTP) nunca debe exponer credenciales.
#[test]
fn correo_dev_solo_expone_la_temporal_en_modo_desarrollo() {
    let silencioso = CorreoDev::new();
    assert!(
        silencioso
            .aviso_credenciales("ana@mail.com", "Ana", "Temp123AbcDef")
            .is_none(),
        "el stub por defecto nunca imprime credenciales"
    );

    let desarrollo = CorreoDev::para_desarrollo();
    let aviso = desarrollo
        .aviso_credenciales("ana@mail.com", "Ana", "Temp123AbcDef")
        .expect("en desarrollo sí se imprime el aviso");
    assert!(aviso.contains("ana@mail.com"));
    assert!(aviso.contains("Temp123AbcDef"));
}

/// T029 — el transporte SMTP se construye desde `SMTP_URL`; una URL o un
/// remitente inválidos fallan al construirlo (el arranque no sigue a ciegas).
#[test]
fn smtp_rechaza_url_o_remitente_invalidos() {
    assert!(
        CorreoSmtp::desde_url("no-es-una-url", "soporte@stockmante.local").is_err(),
        "una SMTP_URL sin esquema válido debe fallar"
    );
    assert!(
        CorreoSmtp::desde_url(
            "smtps://usuario:secreto@smtp.example.com:465",
            "no-es-email"
        )
        .is_err(),
        "un EMAIL_FROM inválido debe fallar"
    );
    assert!(
        CorreoSmtp::desde_url(
            "smtps://usuario:secreto@smtp.example.com:465",
            "Stockmante <soporte@stockmante.local>"
        )
        .is_ok(),
        "una URL smtps y un remitente con nombre son válidos"
    );
}
