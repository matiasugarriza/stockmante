//! T029 — el alta entrega una contraseña temporal por correo y nunca expone el hash.

mod common;

use common::{actor_de_test, borrar_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::error::AppError;
use stockmante_backend::mail::envio::{CorreoDev, EnvioCorreo};
use stockmante_backend::services::{auth, usuarios};

/// Transporte que siempre falla: simula un SMTP caído en el alta (T029).
struct CorreoQueFalla;

#[async_trait::async_trait]
impl EnvioCorreo for CorreoQueFalla {
    async fn enviar_password_temporal(
        &self,
        _destino: &str,
        _nombre: &str,
        _password_temporal: &str,
    ) -> Result<(), String> {
        Err("SMTP no disponible".into())
    }
}

#[tokio::test]
async fn alta_entrega_temporal_y_permite_el_primer_login() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let email = email_unico("alta-temporal");
    let actor = actor_de_test(&pool).await;

    let creado = usuarios::crear(&pool, &correo, actor, "Nueva Persona", &email, "CAJERO")
        .await
        .expect("alta");
    assert!(creado.password_temporal);

    let bandeja = correo.bandeja();
    let (destino, nombre, temporal) = bandeja
        .last()
        .expect("el alta debe enviar la contraseña temporal")
        .clone();
    assert_eq!(destino, email);
    assert_eq!(nombre, "Nueva Persona");
    assert!(
        temporal.chars().count() >= 12,
        "la temporal debe tener al menos 12 caracteres"
    );
    assert!(temporal.chars().any(|c| c.is_ascii_uppercase()));
    assert!(temporal.chars().any(|c| c.is_ascii_digit()));

    // El hash almacenado jamás es la temporal en claro.
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM usuarios WHERE id = $1")
        .bind(creado.id)
        .fetch_one(&pool)
        .await
        .expect("hash almacenado");
    assert_ne!(hash, temporal);
    assert!(hash.starts_with("$argon2id$"));

    // La temporal sirve para el primer login y obliga el cambio.
    let (_sesion, login) = auth::login(&pool, &RelojSistema, &email, &temporal, 8)
        .await
        .expect("login con la temporal");
    assert!(
        login.password_temporal,
        "el primer login debe seguir marcado como temporal"
    );

    borrar_usuario(&pool, &email).await;
}

/// T029 — fail-closed: si el correo con la temporal no se puede entregar, el
/// alta se revierte y no queda una cuenta con credenciales que nadie recibió.
#[tokio::test]
async fn si_el_correo_falla_no_queda_usuario() {
    let pool = pool_de_test().await;
    let email = email_unico("alta-sin-correo");
    let actor = actor_de_test(&pool).await;

    let error = usuarios::crear(&pool, &CorreoQueFalla, actor, "Sin Correo", &email, "CAJERO")
        .await
        .expect_err("el alta debe fallar si no se entrega la temporal");
    assert!(
        matches!(error, AppError::Interno(_)),
        "esperaba error interno genérico, vino {error:?}"
    );

    let existentes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usuarios WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .expect("conteo de usuarios");
    assert_eq!(existentes, 0, "la transacción del alta debe revertirse");
}
