//! T017 / T-AUTH-01 / T-AUTH-02 — Login: fallos indistinguibles (anti-enumeración).

mod common;

use common::{borrar_usuario, crear_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::error::AppError;
use stockmante_backend::services::auth;

#[tokio::test]
async fn password_incorrecta_y_email_inexistente_mismo_error() {
    let pool = pool_de_test().await;
    let email = email_unico("login-mal-pass");
    crear_usuario(
        &pool,
        "Usuario Login",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let err_pass =
        auth::login(&pool, &RelojSistema, &email, "Password2", 8)
            .await
            .expect_err("password incorrecta debe fallar");
    let err_email = auth::login(
        &pool,
        &RelojSistema,
        "nobody@test.local",
        "Password1",
        8,
    )
    .await
    .expect_err("email inexistente debe fallar");

    assert!(matches!(err_pass, AppError::CredencialesInvalidas));
    assert!(matches!(err_email, AppError::CredencialesInvalidas));
    // mismo mensaje público exacto (T-AUTH-01)
    assert_eq!(err_pass.mensaje_publico(), err_email.mensaje_publico());

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn cuenta_inactiva_mismo_error_generico() {
    let pool = pool_de_test().await;
    let email = email_unico("login-inactivo");
    crear_usuario(
        &pool,
        "Usuario Inactivo",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        false,
        false,
    )
    .await;

    let err = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect_err("cuenta inactiva debe fallar con el mismo 401 genérico");
    assert!(matches!(err, AppError::CredencialesInvalidas));
    assert_eq!(err.mensaje_publico(), "Credenciales inválidas");

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn login_valido_devuelve_sesion_y_usuario() {
    let pool = pool_de_test().await;
    let email = email_unico("login-ok");
    crear_usuario(
        &pool,
        "Usuario Ok",
        &email,
        "Password1",
        "CAJERO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let (sesion_id, usuario) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("login válido debe devolver sesión");
    assert_eq!(usuario.email, email);
    assert_eq!(usuario.rol, "CAJERO");
    assert_eq!(usuario.familia_rol, "OPERACION");

    // la sesión existe y es vigente
    let vigente = auth::sesion_actual(&pool, &RelojSistema, sesion_id)
        .await
        .expect("sesión creada debe ser válida");
    assert_eq!(vigente.id, usuario.id);

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn email_se_busca_sin_mayusculas_de_mas() {
    let pool = pool_de_test().await;
    let email = email_unico("login-case");
    crear_usuario(
        &pool,
        "Usuario Case",
        &email,
        "Password1",
        "ADMIN",
        "GESTION",
        true,
        false,
    )
    .await;

    let upper = email.to_uppercase();
    let resultado = auth::login(&pool, &RelojSistema, &upper, "Password1", 8).await;
    assert!(
        resultado.is_ok(),
        "el lookup debe normalizar a minúsculas: {resultado:?}"
    );

    borrar_usuario(&pool, &email).await;
}
