//! Hash Argon2id y generación de contraseñas temporales (M1-A).

use crate::error::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::distr::{Alphanumeric, SampleString};
use rand::Rng;

/// Hashea con Argon2id (parámetros por defecto de la librería).
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifica en texto plano contra un hash Argon2id almacenado.
/// Nunca distingue entre "hash inválido" y "password incorrecta".
pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

const TEMP_LEN: usize = 14;

/// Política de contraseña de usuario (validation.md): ≥8, ≥1 mayúscula, ≥1 dígito.
pub fn validar_politica_password(password: &str) -> Result<(), AppError> {
    let cumple = password.chars().count() >= 8
        && password.chars().any(|c| c.is_ascii_uppercase())
        && password.chars().any(|c| c.is_ascii_digit());
    if cumple {
        Ok(())
    } else {
        Err(AppError::Validacion(
            "La contraseña debe tener al menos 8 caracteres, una mayúscula y un dígito".into(),
        ))
    }
}

/// Password temporal: >= 12 chars, 1 mayúscula, 1 dígito (CSPRNG).
pub fn generar_password_temporal() -> String {
    let mut rng = rand::rng();
    loop {
        let base: String = Alphanumeric.sample_string(&mut rng, TEMP_LEN);
        let mut chars: Vec<char> = base.chars().collect();
        // garantiza 1 mayúscula y 1 dígito en posiciones aleatorias
        let pos_upper = rng.random_range(0..TEMP_LEN);
        let pos_digit = rng.random_range(0..TEMP_LEN);
        chars[pos_upper] = chars[pos_upper].to_ascii_uppercase();
        chars[pos_digit] = rng.random_range(b'0'..=b'9') as char;
        let candidate: String = chars.into_iter().collect();
        if candidate.chars().any(|c| c.is_ascii_uppercase())
            && candidate.chars().any(|c| c.is_ascii_digit())
            && candidate.len() >= 12
        {
            return candidate;
        }
    }
}
