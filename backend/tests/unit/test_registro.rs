//! T033 — Registro: anti-enumeración, política de contraseña, 202.

mod common;

use common::{borrar_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::password::validar_politica_password;
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::error::AppError;
use stockmante_backend::mail::envio::CorreoDev;
use stockmante_backend::services::{auth, registro};

/// FR-070 / escenario 5 de US1: la cuenta de auto-registro queda **pendiente de
/// verificación** y no debe poder iniciar sesión hasta confirmar el correo.
/// Reproducción del reporte del usuario 2026-11-02 ("sin verificar igual pude entrar").
#[tokio::test]
async fn registro_pendiente_no_puede_iniciar_sesion_hasta_verificar() {
    let pool = pool_de_test().await;
    let email = email_unico("registro-pendiente");

    registro::registrar(
        &pool,
        &CorreoDev::new(),
        "Pendiente Verificacion",
        &email,
        "Password1",
        "http://localhost:5173",
    )
    .await
    .expect("el registro previo debe ser Ok");

    // Estado esperado: inactiva y sin verificar.
    let (activo, verificado): (bool, bool) =
        sqlx::query_as("SELECT activo, email_verificado FROM usuarios WHERE email = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .expect("fila del usuario registrado");
    assert!(
        !activo,
        "el auto-registro debe crear la cuenta INACTIVA (pendiente de verificación)"
    );
    assert!(!verificado, "sin verificar, email_verificado debe ser false");

    // Login con la password correcta debe fallar (indistinguible: anti-enumeración).
    let err = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect_err("no debe poder iniciar sesión sin verificar el correo");
    assert!(
        matches!(err, AppError::CredencialesInvalidas),
        "esperaba CredencialesInvalidas; llegó: {err:?}"
    );

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn registro_valido_devuelve_202_y_crea_usuario() {
    let pool = pool_de_test().await;
    let email = email_unico("registro-ok");
    let correo = CorreoDev::new();

    let resultado = registro::registrar(
        &pool,
        &correo,
        "Usuario Registro",
        &email,
        "Password1",
        "http://localhost:5173",
    )
    .await;

    assert!(resultado.is_ok(), "registro válido debe retornar 202");

    // El correo ya existe: segundo registro debe seguir siendo Ok (anti-enumeración).
    let resultado2 = registro::registrar(
        &pool,
        &correo,
        "Usuario Registro",
        &email,
        "Password1",
        "http://localhost:5173",
    )
    .await;
    assert!(
        resultado2.is_ok(),
        "re-registro del mismo correo debe ser Ok (anti-enumeración)"
    );

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn contraseña_corta_devuelve_422() {
    let pool = pool_de_test().await;
    let email = email_unico("registro-corta");

    let resultado = registro::registrar(
        &pool,
        &CorreoDev::new(),
        "Usuario Corto",
        &email,
        "corta", // < 8 chars, no mayúscula, no dígito
        "http://localhost:5173",
    )
    .await;

    assert!(resultado.is_err());
    assert!(matches!(resultado.unwrap_err(), AppError::Validacion(_)));

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn politica_password_rechaza_sin_mayuscula() {
    let err = validar_politica_password("password1");
    assert!(err.is_err());
    assert!(matches!(err.unwrap_err(), AppError::Validacion(_)));
}

#[tokio::test]
async fn politica_password_rechaza_sin_digito() {
    let err = validar_politica_password("Password");
    assert!(err.is_err());
    assert!(matches!(err.unwrap_err(), AppError::Validacion(_)));
}

#[tokio::test]
async fn politica_password_acepta_valida() {
    let err = validar_politica_password("Password1");
    assert!(err.is_ok());
}
