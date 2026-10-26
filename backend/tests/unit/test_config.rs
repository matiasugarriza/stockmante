use stockmante_backend::config::AppConfig;

fn config_base() -> AppConfig {
    AppConfig {
        database_url: "postgres://localhost/stockmante".into(),
        app_url: "http://localhost:5173".into(),
        port: 3000,
        session_cookie_name: "stockmante_sesion".into(),
        session_expiry_hours: 8,
        email_from: "soporte@stockmante.local".into(),
        smtp_url: None,
        app_env: "development".into(),
        docs_enabled: true,
    }
}

/// 000 — la documentación se enciende sola fuera de producción y se apaga en
/// producción; `DOCS_ENABLED` explícito siempre gana.
#[test]
fn docs_enabled_tiene_default_por_entorno() {
    assert!(AppConfig::docs_por_defecto("development"));
    assert!(AppConfig::docs_por_defecto("staging"));
    assert!(!AppConfig::docs_por_defecto("production"));
    assert!(!AppConfig::docs_por_defecto(" Production "));
}

#[test]
fn docs_enabled_acepta_booleanos_textuales() {
    assert!(AppConfig::parsear_interruptor("DOCS_ENABLED", "true").expect("true"));
    assert!(AppConfig::parsear_interruptor("DOCS_ENABLED", " TRUE ").expect("TRUE"));
    assert!(AppConfig::parsear_interruptor("DOCS_ENABLED", "1").expect("1"));
    assert!(!AppConfig::parsear_interruptor("DOCS_ENABLED", "false").expect("false"));
    assert!(!AppConfig::parsear_interruptor("DOCS_ENABLED", "0").expect("0"));
    assert!(AppConfig::parsear_interruptor("DOCS_ENABLED", "quizas").is_err());
}

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

/// T029 — producción sin SMTP_URL no valida: no habría forma de entregar la
/// temporal y el alta quedaría con credenciales que nadie recibe.
#[test]
fn produccion_sin_smtp_url_no_valida() {
    let mut config = config_base();
    config.app_env = "production".into();
    assert!(config.validar().is_err());

    config.smtp_url = Some("   ".into());
    assert!(
        config.validar().is_err(),
        "una SMTP_URL en blanco no cuenta como configurada"
    );
}

/// T029 — producción con SMTP_URL valida, y desarrollo funciona sin SMTP.
#[test]
fn produccion_con_smtp_valida_y_desarrollo_sin_smtp_tambien() {
    let mut produccion = config_base();
    produccion.app_env = " Production ".into();
    produccion.smtp_url = Some("smtps://usuario:secreto@smtp.example.com:465".into());
    assert!(produccion.validar().is_ok());

    let desarrollo = config_base();
    assert!(desarrollo.validar().is_ok());
}
