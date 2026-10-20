//! T019 / T-AUTH-05 — Cambio de password temporal: el viejo queda inservible.

mod common;

use common::{borrar_usuario, crear_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::password::verify_password;
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::error::AppError;
use stockmante_backend::services::auth;

#[tokio::test]
async fn cambio_de_password_temporal_limpia_el_flag_y_deja_viejo_inservible() {
    let pool = pool_de_test().await;
    let email = email_unico("cambio-temp");
    let id = crear_usuario(
        &pool,
        "Usuario Temporal",
        &email,
        "Temporal1",
        "ADMIN",
        "GESTION",
        true,
        true,
    )
    .await;

    let (sesion, usuario) = auth::login(&pool, &RelojSistema, &email, "Temporal1", 8)
        .await
        .expect("login con password temporal");
    assert!(usuario.password_temporal, "arranca como temporal");

    auth::cambiar_password(&pool, &RelojSistema, sesion, "Temporal1", "Nueva1234")
        .await
        .expect("cambio de password temporal debe funcionar");

    // flag limpio
    let fila = sqlx::query_as::<_, (String, bool)>(
        r#"SELECT password_hash, password_temporal FROM usuarios WHERE id = $1"#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("fetch usuario");
    assert!(!fila.1, "password_temporal debe ser FALSE tras el cambio");

    // el hash viejo ya no verifica
    assert!(
        !verify_password("Temporal1", &fila.0),
        "el hash viejo no debe verificar"
    );
    assert!(
        verify_password("Nueva1234", &fila.0),
        "el hash nuevo debe verificar"
    );

    // el login viejo ya no sirve (T-AUTH-05 "vieja no sirve")
    let err = auth::login(&pool, &RelojSistema, &email, "Temporal1", 8)
        .await
        .expect_err("la password vieja no debe loguear");
    assert!(matches!(err, AppError::CredencialesInvalidas));

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn cambio_rechaza_password_que_no_cumple_politica() {
    let pool = pool_de_test().await;
    let email = email_unico("cambio-policy");
    let _id = crear_usuario(
        &pool,
        "Usuario Policy",
        &email,
        "Password1",
        "ADMIN",
        "GESTION",
        true,
        false,
    )
    .await;
    let (sesion, _) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("login");

    for mala in ["corta1A", "sinmayuscula1", "SINNUMEROSAQUI", "12345678"] {
        let err = auth::cambiar_password(&pool, &RelojSistema, sesion, "Password1", mala)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validacion(_)),
            "esperaba Validacion para {mala:?}, vino {err:?}"
        );
    }

    // la password actual no cambió
    assert!(
        auth::login(&pool, &RelojSistema, &email, "Password1", 8)
            .await
            .is_ok(),
        "un cambio rechazado no debe tocar el hash"
    );

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn cambio_con_password_actualIncorrecta_da_credenciales_invalidas() {
    let pool = pool_de_test().await;
    let email = email_unico("cambio-actual-mal");
    let _id = crear_usuario(
        &pool,
        "Usuario Actual",
        &email,
        "Password1",
        "ADMIN",
        "GESTION",
        true,
        false,
    )
    .await;
    let (sesion, _) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("login");

    let err = auth::cambiar_password(&pool, &RelojSistema, sesion, "PasswordX", "Nueva1234")
        .await
        .expect_err("password actual incorrecta debe fallar");
    assert!(matches!(err, AppError::CredencialesInvalidas));

    borrar_usuario(&pool, &email).await;
}
