//! T018 / T-AUTH-03 — Sesión única: el segundo login revoca el anterior.
//! T026 — Expiración absoluta de 8 h, sin sliding por actividad.
//! T027 / T-AUTH-09 — La BD impide dos sesiones vigentes y sobrevive a logins concurrentes.

mod common;

use common::{borrar_usuario, crear_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::reloj::{RelojFalso, RelojSistema};
use stockmante_backend::error::AppError;
use stockmante_backend::services::auth;
use uuid::Uuid;

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

/// T026 — la expiración es absoluta desde el login: la actividad no la renueva
/// (no hay sliding) y `expira_en` conserva el instante original.
#[tokio::test]
async fn la_actividad_no_renueva_la_sesion() {
    let pool = pool_de_test().await;
    let email = email_unico("sin-sliding");
    crear_usuario(
        &pool,
        "Usuario Sin Sliding",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let inicio = chrono::Utc::now();
    let reloj_inicio = RelojFalso { ahora: inicio };
    let (sesion, _) = auth::login(&pool, &reloj_inicio, &email, "Password1", 8)
        .await
        .expect("login con reloj falso");

    // Actividad a las 7 h 59 min: la sesión sigue vigente…
    let casi = RelojFalso {
        ahora: inicio + chrono::Duration::minutes(8 * 60 - 1),
    };
    auth::sesion_actual(&pool, &casi, sesion)
        .await
        .expect("a las 7h59m la sesión debe seguir vigente");

    // …pero a las 8 h 01 min ya no, aunque hubo actividad hace dos minutos.
    let tarde = RelojFalso {
        ahora: inicio + chrono::Duration::minutes(8 * 60 + 1),
    };
    let err = auth::sesion_actual(&pool, &tarde, sesion)
        .await
        .expect_err("la actividad no debe renovar la sesión");
    assert!(
        matches!(err, AppError::SesionExpirada),
        "esperaba SesionExpirada, vino {err:?}"
    );

    // La fila conserva el vencimiento original (nada lo corrió por actividad).
    let expira_en: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar(r#"SELECT expira_en FROM sesiones WHERE id = $1"#)
            .bind(sesion)
            .fetch_one(&pool)
            .await
            .expect("expira_en almacenado");
    let esperado = inicio + chrono::Duration::hours(8);
    assert!(
        (expira_en - esperado).num_seconds().abs() <= 1,
        "expira_en {expira_en} debe ser ~{esperado} (login + 8 h)"
    );

    borrar_usuario(&pool, &email).await;
}

/// T027 — la invariante no depende sólo del servicio: un índice único parcial
/// impide que existan dos filas vigentes para el mismo usuario.
#[tokio::test]
async fn la_bd_impide_dos_sesiones_vigentes_por_usuario() {
    let pool = pool_de_test().await;
    let email = email_unico("indice-vigente");
    let id = crear_usuario(
        &pool,
        "Usuario Índice",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    sqlx::query(
        r#"INSERT INTO sesiones (id, usuario_id, expira_en)
           VALUES ($1, $2, now() + interval '8 hours')"#,
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .execute(&pool)
    .await
    .expect("la primera sesión vigente debe entrar");

    let err = sqlx::query(
        r#"INSERT INTO sesiones (id, usuario_id, expira_en)
           VALUES ($1, $2, now() + interval '8 hours')"#,
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .execute(&pool)
    .await
    .expect_err("una segunda sesión vigente debe violar el índice único parcial");

    let db_err = err.as_database_error().expect("debe ser un error de BD");
    assert_eq!(
        db_err.code().as_deref(),
        Some("23505"),
        "esperaba violación de unicidad, vino {db_err:?}"
    );

    borrar_usuario(&pool, &email).await;
}

/// T027 / T-AUTH-09 — dos logins simultáneos dejan una sola sesión vigente y
/// ambas llamadas terminan bien (la última gana).
#[tokio::test]
async fn dos_logins_concurrentes_dejan_una_sola_sesion_vigente() {
    let pool = pool_de_test().await;
    let email = email_unico("login-concurrente");
    crear_usuario(
        &pool,
        "Usuario Concurrente",
        &email,
        "Password1",
        "OPERARIO",
        "OPERACION",
        true,
        false,
    )
    .await;

    let (a, b) = tokio::join!(
        auth::login(&pool, &RelojSistema, &email, "Password1", 8),
        auth::login(&pool, &RelojSistema, &email, "Password1", 8),
    );
    let (sesion_a, _) = a.expect("login concurrente A");
    let (sesion_b, _) = b.expect("login concurrente B");
    assert_ne!(sesion_a, sesion_b, "cada login genera un id propio");

    let vigentes: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)
           FROM sesiones s
           JOIN usuarios u ON u.id = s.usuario_id
           WHERE u.email = $1 AND s.revocada_en IS NULL"#,
    )
    .bind(&email)
    .fetch_one(&pool)
    .await
    .expect("conteo de sesiones vigentes");
    assert_eq!(vigentes, 1, "sólo puede quedar una sesión vigente");

    let viva_a = auth::sesion_actual(&pool, &RelojSistema, sesion_a)
        .await
        .is_ok();
    let viva_b = auth::sesion_actual(&pool, &RelojSistema, sesion_b)
        .await
        .is_ok();
    assert!(
        viva_a ^ viva_b,
        "exactamente una de las dos cookies debe seguir viva (a={viva_a}, b={viva_b})"
    );

    borrar_usuario(&pool, &email).await;
}
