//! T026 — unicidad y formato de email en el alta de usuarios (US2).

mod common;

use axum::http::StatusCode;
use common::{actor_de_test, borrar_usuario, email_unico, pool_de_test};
use stockmante_backend::error::AppError;
use stockmante_backend::mail::envio::CorreoDev;
use stockmante_backend::services::usuarios;

#[tokio::test]
async fn alta_con_email_duplicado_devuelve_email_duplicado() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let email = email_unico("duplicado");
    let actor = actor_de_test(&pool).await;

    let creado = usuarios::crear(&pool, &correo, actor, "Primera Persona", &email, "OPERARIO")
        .await
        .expect("el primer alta debe funcionar");
    assert_eq!(creado.email, email);
    assert_eq!(creado.familia_rol, "OPERACION");
    assert!(creado.activo, "el alta deja al usuario activo");
    assert!(creado.password_temporal, "el alta usa una temporal");

    // La unicidad no distingue mayúsculas ni espacios sobrantes.
    let repetido = format!("  {}  ", email.to_uppercase());
    let err = usuarios::crear(&pool, &correo, actor, "Segunda Persona", &repetido, "GERENTE")
        .await
        .expect_err("el email ya está registrado");
    assert!(
        matches!(err, AppError::EmailDuplicado),
        "esperaba EmailDuplicado, vino {err:?}"
    );
    assert_eq!(err.status(), StatusCode::CONFLICT);
    assert_eq!(err.codigo(), "email_duplicado");

    borrar_usuario(&pool, &email).await;
}

#[tokio::test]
async fn alta_con_email_invalido_es_validacion() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let actor = actor_de_test(&pool).await;

    for invalido in ["sin-arroba", "sin-dominio@", "@sin-usuario.com", "a@b"] {
        let err = usuarios::crear(&pool, &correo, actor, "Persona", invalido, "OPERARIO")
            .await
            .expect_err("el email debe rechazarse");
        assert!(
            matches!(err, AppError::Validacion(_)),
            "para {invalido:?} esperaba Validacion, vino {err:?}"
        );
        assert_eq!(err.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn alta_valida_nombre_y_rol() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let actor = actor_de_test(&pool).await;

    let err = usuarios::crear(
        &pool,
        &correo,
        actor,
        "   ",
        &email_unico("sin-nombre"),
        "OPERARIO",
    )
    .await
    .expect_err("el nombre vacío no es válido");
    assert!(matches!(err, AppError::Validacion(_)));

    let err = usuarios::crear(
        &pool,
        &correo,
        actor,
        "Persona",
        &email_unico("rol-malo"),
        "SUPERVISOR",
    )
    .await
    .expect_err("el rol fuera del catálogo no es válido");
    assert!(matches!(err, AppError::Validacion(_)));
}
