use stockmante_backend::config::AppConfig;

/// `APP_ENV=production` es el único valor que desactiva ayudas de desarrollo
/// (impresión de credenciales, resets del bootstrap).
#[test]
fn solo_production_se_considera_produccion() {
    assert!(AppConfig::es_entorno_produccion("production"));
    assert!(AppConfig::es_entorno_produccion(" Production "));
    assert!(AppConfig::es_entorno_produccion("PRODUCTION"));
    assert!(!AppConfig::es_entorno_produccion("development"));
    assert!(!AppConfig::es_entorno_produccion("staging"));
    assert!(!AppConfig::es_entorno_produccion(""));
}
