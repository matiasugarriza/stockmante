//! T023 — Bootstrap: depósito "Principal" + admin inicial (research R7).
//! En DEV imprime la contraseña temporal en consola; en producción jamás.

use stockmante_backend::auth::password::{generar_password_temporal, hash_password};
use stockmante_backend::config::AppConfig;
use stockmante_backend::db;
use stockmante_backend::error::AppError;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    if let Err(e) = correr().await {
        eprintln!("Bootstrap falló: {e}");
        std::process::exit(1);
    }
}

async fn correr() -> Result<(), AppError> {
    let config = AppConfig::desde_entorno()?;
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

    // La contraseña temporal solo se imprime fuera de producción (SIR).
    let entorno = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
    if entorno == "production" {
        tracing::warn!("admin creado en producción SIN imprimir credenciales; regenerá la contraseña temporal desde una sesión de gestión");
    } else {
        println!("──────────────────────────────────────────────");
        println!(" Admin inicial creado (solo esta vez se imprime)");
        println!("   email:    {email_admin}");
        println!("   password: {password_temporal}");
        println!("   (login y forzar cambio de contraseña en /login)");
        println!("──────────────────────────────────────────────");
    }
    Ok(())
}
