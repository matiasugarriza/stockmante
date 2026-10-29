//! T027 — guarda del último usuario de gestión activo (FR-006, US2).
//!
//! La guarda es global (cuenta usuarios activos de familia GESTION), así que el
//! test neutraliza los gestores ajenos y los restaura antes de afirmar. Para no
//! dejar estado sucio si algo falla, primero ejecuta las operaciones, restaura y
//! recién después hace los `assert`.

mod common;

use axum::http::StatusCode;
use common::{actor_de_test, borrar_usuario, email_unico, pool_de_test};
use stockmante_backend::auth::reloj::RelojSistema;
use stockmante_backend::error::AppError;
use stockmante_backend::mail::envio::CorreoDev;
use stockmante_backend::services::usuarios::{self, CambiosUsuario};

#[tokio::test]
async fn desactivar_o_degradar_al_ultimo_gestor_activo_es_rechazado() {
    let pool = pool_de_test().await;
    let correo = CorreoDev::new();
    let reloj = RelojSistema;

    // Gestores activos ajenos al test: se desactivan para aislar el conteo.
    let otros: Vec<i32> = sqlx::query_scalar(
        r#"SELECT id FROM usuarios WHERE familia_rol = 'GESTION' AND activo ORDER BY id"#,
    )
    .fetch_all(&pool)
    .await
    .expect("gestores activos actuales");
    sqlx::query(r#"UPDATE usuarios SET activo = FALSE WHERE id = ANY($1::int[])"#)
        .bind(&otros)
        .execute(&pool)
        .await
        .expect("neutralizar gestores ajenos");

    let email_a = email_unico("ultimo-admin-a");
    let email_b = email_unico("ultimo-admin-b");
    let actor = actor_de_test(&pool).await;
    let gestor_a = usuarios::crear(&pool, &correo, actor, "Gestor A", &email_a, "ADMIN")
        .await
        .expect("alta gestor A");
    let gestor_b = usuarios::crear(&pool, &correo, actor, "Gestor B", &email_b, "GERENTE")
        .await
        .expect("alta gestor B");

    // Operaciones bajo prueba (sin asserts todavía, para poder restaurar).
    let baja_b = usuarios::desactivar(&pool, &reloj, gestor_b.id, actor).await;
    let estado_a = usuarios::obtener(&pool, gestor_a.id).await;
    let baja_a = usuarios::desactivar(&pool, &reloj, gestor_a.id, actor).await;
    let degradar_a = usuarios::actualizar(
        &pool,
        gestor_a.id,
        CambiosUsuario {
            rol: Some("OPERARIO".into()),
            ..CambiosUsuario::default()
        },
        actor,
    )
    .await;
    let estado_a_final = usuarios::obtener(&pool, gestor_a.id).await;

    // Restauración del entorno compartido.
    borrar_usuario(&pool, &email_a).await;
    borrar_usuario(&pool, &email_b).await;
    sqlx::query(r#"UPDATE usuarios SET activo = TRUE WHERE id = ANY($1::int[])"#)
        .bind(&otros)
        .execute(&pool)
        .await
        .expect("restaurar gestores ajenos");

    baja_b.expect("con otro gestor activo la baja debe proceder");
    assert!(
        estado_a.expect("gestor A").activo,
        "el gestor A debe seguir activo"
    );

    let err = baja_a.expect_err("no se puede desactivar al último gestor activo");
    assert!(
        matches!(err, AppError::UltimoAdministrador),
        "esperaba UltimoAdministrador, vino {err:?}"
    );
    assert_eq!(err.status(), StatusCode::CONFLICT);
    assert_eq!(err.codigo(), "ultimo_administrador");

    let err = degradar_a.expect_err("tampoco se puede degradar al último gestor activo");
    assert!(
        matches!(err, AppError::UltimoAdministrador),
        "esperaba UltimoAdministrador, vino {err:?}"
    );

    let final_a = estado_a_final.expect("gestor A tras los intentos");
    assert!(final_a.activo, "el gestor A debe seguir activo");
    assert_eq!(
        final_a.familia_rol, "GESTION",
        "el rol de gestión no debe cambiar"
    );
}
