use stockmante_backend::auth::password::{generar_password_temporal, hash_password, verify_password};

#[test]
fn hash_y_verify_roundtrip() {
    let hash = hash_password("Se$ura123").expect("hash debe generarse");
    assert!(verify_password("Se$ura123", &hash));
    assert!(!verify_password("Se$ura124", &hash));
    assert!(!verify_password("", &hash));
}

#[test]
fn hash_no_contiene_password_en_plano() {
    let hash = hash_password("Se$ura123").expect("hash debe generarse");
    assert!(!hash.contains("Se$ura123"));
}

#[test]
fn password_temporal_cumple_politica() {
    // >= 12 chars, 1 mayúscula, 1 dígito (M1-A / contracts/validation.md)
    for _ in 0..50 {
        let p = generar_password_temporal();
        assert!(p.len() >= 12, "temporal demasiado corta: {p}");
        assert!(p.chars().any(|c| c.is_ascii_uppercase()), "sin mayúscula: {p}");
        assert!(p.chars().any(|c| c.is_ascii_digit()), "sin dígito: {p}");
    }
}

#[test]
fn temporales_son_aleatorias_entre_si() {
    let a = generar_password_temporal();
    let b = generar_password_temporal();
    assert_ne!(a, b, "CSPRNG no está generando valores únicos");
}
