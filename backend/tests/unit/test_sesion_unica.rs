//! T018 / T-AUTH-03 — Sesión única: el segundo login revoca el anterior.

mod common;

use common::{borrar_usuario, crear_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::error::AppError;
use stockmante_backend::services::auth;

#[tokio::test]
async fn segundo_login_revoca_la_sesion_anterior() {
    let pool = pool_de_test().await;
    let email = email_unico("sesion-unica");
    crear_usuario(
        &pool,
        "Usuario Sesión",
        &email,
        "Password1",
        "GERENTE",
        "GESTION",
        true,
        false,
    )
    .await;

    let (primera, _) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("primer login");
    let (segunda, _) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("segundo login");

    assert_ne!(primera, segunda, "cada login debe generar un id distinto");

    // la cookie vieja deja de servir (401), la nueva sigue viva
    let err_vieja = auth::sesion_actual(&pool, &RelojSistema, primera)
        .await
        .expect_err("la sesión revocada debe rechazarse");
    assert!(matches!(err_vieja, AppError::NoAutenticado));

    let vigente = auth::sesion_actual(&pool, &RelojSistema, segunda)
        .await
        .expect("la sesión nueva debe quedar vigente");
    assert_eq!(vigente.email, email);

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn logout_revoca_y_es_idempotente() {
    let pool = pool_de_test().await;
    let email = email_unico("logout");
    crear_usuario(
        &pool,
        "Usuario Logout",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let (sesion, _) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("login");

    auth::logout(&pool, &RelojSistema, sesion)
        .await
        .expect("logout debe salir bien");
    // idempotente: segundo logout con la misma cookie no rompe
    auth::logout(&pool, &RelojSistema, sesion)
        .await
        .expect("logout repetido sigue siendo 204");

    let err = auth::sesion_actual(&pool, &RelojSistema, sesion)
        .await
        .expect_err("tras logout la sesión no sirve");
    assert!(matches!(err, AppError::NoAutenticado));

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn sesion_expirada_segun_reloj_inyectado() {
    let pool = pool_de_test().await;
    let email = email_unico("expiracion");
    crear_usuario(
        &pool,
        "Usuario Expira",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let reloj_fijo = stockmante_backend::auth::reloj::RelojFalso {
        ahora: chrono::Utc::now(),
    };
    let (sesion, _) = auth::login(&pool, &reloj_fijo, &email, "Password1", 8)
        .await
        .expect("login con reloj falso");

    // 9 horas después (expiry = 8h): expirada
    let reloj_tarde = stockmante_backend::auth::reloj::RelojFalso {
        ahora: reloj_fijo.ahora + chrono::Duration::hours(9),
    };
    let err = auth::sesion_actual(&pool, &reloj_tarde, sesion)
        .await
        .expect_err("a las 9h la sesión de 8h debe estar expirada");
    assert!(
        matches!(err, AppError::SesionExpirada),
        "esperaba SesionExpirada, vino {err:?}"
    );

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn desactivar_usuario_deja_sesiones_invalidas() {
    let pool = pool_de_test().await;
    let email = email_unico("desactiva-sesion");
    let id = crear_usuario(
        &pool,
        "Usuario Desactiva",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let (sesion, _) = auth::login(&pool, &RelojSistema, &email, "Password1", 8)
        .await
        .expect("login");

    sqlx::query(r#"UPDATE usuarios SET activo = FALSE WHERE id = $1"#)
        .bind(id)
        .execute(&pool)
        .await
        .expect("desactivar usuario");

    // T-AUTH-04: la sesión existente deja de servir
    let err = auth::sesion_actual(&pool, &RelojSistema, sesion)
        .await
        .expect_err("usuario inactivo con sesión vieja → 401");
    assert!(matches!(err, AppError::NoAutenticado));

    borrar_usuario(&pool, &email).await;
}
