//! T023 — Bootstrap: depósito "Principal" + admin inicial (research R7).
//! En DEV imprime la contraseña temporal en consola; en producción jamás.
//!
//! Uso:
//!   cargo run --bin bootstrap                          # seed idempotente
//!   cargo run --bin bootstrap -- --reset-admin-password  # DEV: regenera temporal del admin

use stockmante_backend::auth::password::{generar_password_temporal, hash_password};
use stockmante_backend::config::AppConfig;
use stockmante_backend::db;
use stockmante_backend::error::AppError;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    let reset_admin = std::env::args().any(|a| a == "--reset-admin-password");

    if let Err(e) = correr(reset_admin).await {
        eprintln!("Bootstrap falló: {e}");
        std::process::exit(1);
    }
}

async fn correr(reset_admin: bool) -> Result<(), AppError> {
    let config = AppConfig::desde_entorno()?;
    let produccion = config.es_produccion();
    let pool = db::crear_pool(&config.database_url).await?;

    // 1) Depósito inicial "Principal" (idempotente)
    let deposito_id = sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO depositos (nombre, identificador)
           VALUES ('Principal', 'PRINCIPAL')
           ON CONFLICT (nombre) DO UPDATE SET nombre = EXCLUDED.nombre
           RETURNING id"#,
    )
    .fetch_one(&pool)
    .await?;
    tracing::info!(id = deposito_id, "depósito Principal listo");

    // 2) Admin inicial (solo si no existe)
    let email_admin = std::env::var("ADMIN_EMAIL")
        .unwrap_or_else(|_| "admin@stockmante.local".into())
        .trim()
        .to_lowercase();

    let existe = sqlx::query_scalar::<_, bool>(r#"SELECT EXISTS(SELECT 1 FROM usuarios WHERE email = $1)"#)
        .bind(&email_admin)
        .fetch_one(&pool)
        .await?;

    if existe {
        if reset_admin {
            return resetear_admin(&pool, &email_admin, produccion).await;
        }
        tracing::info!(email = %email_admin, "el admin ya existe; no se toca");
        return Ok(());
    }

    let password_temporal = generar_password_temporal();
    let hash = hash_password(&password_temporal)?;
    let admin_id = sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO usuarios (nombre, email, password_hash, password_temporal, rol, familia_rol, activo)
           VALUES ($1, $2, $3, TRUE, 'ADMIN', 'GESTION', TRUE)
           RETURNING id"#,
    )
    .bind("Administrador")
    .bind(&email_admin)
    .bind(&hash)
    .fetch_one(&pool)
    .await?;
    tracing::info!(id = admin_id, email = %email_admin, "admin inicial creado");

    imprimir_credenciales(
        &email_admin,
        &password_temporal,
        "Admin inicial creado",
        produccion,
    );
    Ok(())
}

/// Regenera la contraseña temporal del admin y revoca sus sesiones.
/// Solo DEV: en producción se rechaza (evita resetear credenciales sin trazabilidad).
async fn resetear_admin(
    pool: &sqlx::PgPool,
    email: &str,
    produccion: bool,
) -> Result<(), AppError> {
    if produccion {
        return Err(AppError::Validacion(
            "--reset-admin-password no está permitido con APP_ENV=production".into(),
        ));
    }

    let admin_id = sqlx::query_scalar::<_, i32>("SELECT id FROM usuarios WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await?;

    let password_temporal = generar_password_temporal();
    let hash = hash_password(&password_temporal)?;

    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE sesiones SET revocada_en = now() WHERE usuario_id = $1 AND revocada_en IS NULL",
    )
    .bind(admin_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE usuarios SET password_hash = $1, password_temporal = TRUE, actualizado_en = now() WHERE id = $2",
    )
    .bind(&hash)
    .bind(admin_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    tracing::info!(email = %email, "contraseña temporal regenerada; sesiones revocadas");
    imprimir_credenciales(
        email,
        &password_temporal,
        "Contraseña temporal regenerada",
        produccion,
    );
    Ok(())
}

fn imprimir_credenciales(email: &str, password_temporal: &str, titulo: &str, produccion: bool) {
    // La contraseña temporal solo se imprime fuera de producción (SIR).
    if produccion {
        tracing::warn!(
            email = %email,
            "credenciales generadas en producción SIN imprimir; entregarlas por canal seguro"
        );
    } else {
        println!("──────────────────────────────────────────────");
        println!(" {titulo}");
        println!("   email:    {email}");
        println!("   password: {password_temporal}");
        println!("   (login y forzar cambio de contraseña en /login)");
        println!("──────────────────────────────────────────────");
    }
}
