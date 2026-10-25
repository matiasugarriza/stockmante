use stockmante_backend::mail::envio::{CorreoDev, EnvioCorreo};

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
