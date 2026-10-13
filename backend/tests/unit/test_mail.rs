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
