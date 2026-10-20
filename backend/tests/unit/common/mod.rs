//! Helper compartido de tests que necesitan la BD de test (`stockmante_test`).
//! Cada test usa emails únicos (uuid) para poder correr en paralelo.

use sqlx::PgPool;
use stockmante_backend::auth::password::hash_password;

/// Pool hacia `stockmante_test`: usa `DATABASE_URL_TEST` o deriva de `DATABASE_URL`.
pub async fn pool_de_test() -> PgPool {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        let base = std::env::var("DATABASE_URL")
            .expect("definí DATABASE_URL o DATABASE_URL_TEST para tests de BD");
        let trimmed = base.trim_end_matches('/');
        trimmed
            .rsplit_once("/stockmante")
            .map(|(prefix, _)| format!("{prefix}/stockmante_test"))
            .expect("DATABASE_URL debe apuntar a la base stockmante")
    });
    PgPool::connect(&url)
        .await
        .expect("no se pudo conectar a stockmante_test (¿está corriendo postgres?)")
}

pub fn email_unico(etiqueta: &str) -> String {
    format!("{etiqueta}-{}@test.local", uuid::Uuid::new_v4())
}

/// Inserta un usuario de prueba y devuelve su id. `password` en claro.
#[allow(clippy::too_many_arguments)]
pub async fn crear_usuario(
    pool: &PgPool,
    nombre: &str,
    email: &str,
    password: &str,
    rol: &str,
    familia_rol: &str,
    activo: bool,
    password_temporal: bool,
) -> i32 {
    let hash = hash_password(password).expect("hash de password de test");
    sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO usuarios (nombre, email, password_hash, password_temporal, rol, familia_rol, activo)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(nombre)
    .bind(email)
    .bind(&hash)
    .bind(password_temporal)
    .bind(rol)
    .bind(familia_rol)
    .bind(activo)
    .fetch_one(pool)
    .await
    .expect("insert de usuario de test")
}

/// Borra el usuario de prueba (las sesiones caen por CASCADE).
pub async fn borrar_usuario(pool: &PgPool, email: &str) {
    sqlx::query(r#"DELETE FROM usuarios WHERE email = $1"#)
        .bind(email)
        .execute(pool)
        .await
        .expect("borrado de usuario de test");
}
