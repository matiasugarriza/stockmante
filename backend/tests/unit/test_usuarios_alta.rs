//! T029/T008/T009 — el alta entrega la contraseña (automática o manual) por
//! correo y en claro al llamador, y nunca expone el hash.

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

    async fn enviar_verificacion(
        &self,
        _destino: &str,
        _nombre: &str,
        _token: &str,
        _base_url: &str,
    ) -> Result<(), String> {
        Err("SMTP no disponible".into())
    }

    async fn enviar_alerta_stock(
        &self,
        _destino: &str,
        _nombre: &str,
        _cuerpo: &str,
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

    let (creado, _automatica) = usuarios::crear(
        &pool,
        &correo,
        actor,
        "Nueva Persona",
        &email,
        "CAJERO",
        None,
    )
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

    let error = usuarios::crear(
        &pool,
        &CorreoQueFalla,
        actor,
        "Sin Correo",
        &email,
        "CAJERO",
        None,
    )
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

/// T008(a) — contraseña manual: se devuelve **esa misma** clave en claro para
/// la modal, va igual al correo y permite el primer login (FR-008b).
#[tokio::test]
async fn alta_con_password_manual_devuelve_esa_clave() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let email = email_unico("alta-manual");
    let actor = actor_de_test(&pool).await;

    let (creado, clave) = usuarios::crear(
        &pool,
        &correo,
        actor,
        "Manual Persona",
        &email,
        "CAJERO",
        Some("Manual99"),
    )
    .await
    .expect("alta manual");
    assert_eq!(clave, "Manual99", "la modal debe mostrar la clave elegida");
    assert!(creado.password_temporal);

    let (_, _, temporal_correo) = correo.bandeja().last().expect("correo").clone();
    assert_eq!(temporal_correo, "Manual99", "el correo lleva la misma clave");

    let (_sesion, login) = auth::login(&pool, &RelojSistema, &email, "Manual99", 8)
        .await
        .expect("login con la clave manual");
    assert!(login.password_temporal, "cambio obligatorio en el primer ingreso");

    let hash: String = sqlx::query_scalar("SELECT password_hash FROM usuarios WHERE id = $1")
        .bind(creado.id)
        .fetch_one(&pool)
        .await
        .expect("hash almacenado");
    assert_ne!(hash, "Manual99", "jamás se persiste la clave en claro");
    assert!(hash.starts_with("$argon2id$"));

    borrar_usuario(&pool, &email).await;
}

/// T008(b) — clave manual fuera de política → 422 y nada persistido.
#[tokio::test]
async fn alta_con_password_manual_debil_es_validacion() {
    let pool = pool_de_test().await;
    let email = email_unico("alta-manual-debil");
    let actor = actor_de_test(&pool).await;

    let error = usuarios::crear(
        &pool,
        &CorreoDev::new(),
        actor,
        "Clave Débil",
        &email,
        "CAJERO",
        Some("pass1"),
    )
    .await
    .expect_err("una clave sin mayúscula/dígito debe rechazarse");
    assert!(
        matches!(error, AppError::Validacion(_)),
        "esperaba Validacion(422), vino {error:?}"
    );

    let existentes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usuarios WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .expect("conteo de usuarios");
    assert_eq!(existentes, 0, "no debe persistirse nada con clave inválida");

    borrar_usuario(&pool, &email).await;
}

/// T009(a) — sin `password` el backend genera la automática: 14 caracteres,
/// ≥1 mayúscula y ≥1 dígito (los 14 de `TEMP_LEN`).
#[tokio::test]
async fn alta_sin_password_genera_automatica_de_14_caracteres() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let email = email_unico("alta-auto");
    let actor = actor_de_test(&pool).await;

    let (creado, clave) = usuarios::crear(
        &pool,
        &correo,
        actor,
        "Auto Persona",
        &email,
        "CAJERO",
        None,
    )
    .await
    .expect("alta automática");
    assert_eq!(clave.chars().count(), 14, "la automática tiene 14 caracteres");
    assert!(
        clave.chars().any(|c| c.is_ascii_uppercase()),
        "debe tener mayúscula"
    );
    assert!(
        clave.chars().any(|c| c.is_ascii_digit()),
        "debe tener dígito"
    );
    assert!(creado.password_temporal);

    let (_, _, temporal_correo) = correo.bandeja().last().expect("correo").clone();
    assert_eq!(temporal_correo, clave, "correo y modal muestran lo mismo");

    let (_sesion, login) = auth::login(&pool, &RelojSistema, &email, &clave, 8)
        .await
        .expect("login con la automática");
    assert!(login.password_temporal);

    borrar_usuario(&pool, &email).await;
}
