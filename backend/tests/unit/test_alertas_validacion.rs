//! 011 (T009) — reglas puras de alertas: fecha local, paginación y cuerpo del correo.

use chrono::{NaiveDate, TimeZone, Utc};
use stockmante_backend::auth::reloj::RelojFalso;
use stockmante_backend::config::AppConfig;
use stockmante_backend::services::alertas::{
    cuerpo_correo, fecha_local, paginacion_alertas, LineaCorreo,
};

fn reloj(hora_utc: u32) -> RelojFalso {
    RelojFalso {
        ahora: Utc
            .with_ymd_and_hms(2026, 11, 7, hora_utc, 0, 0)
            .single()
            .unwrap_or_default(),
    }
}

#[test]
fn fecha_local_usa_utc_menos_tres() {
    assert_eq!(fecha_local(&reloj(2)), NaiveDate::from_ymd_opt(2026, 11, 6).unwrap_or_default());
    assert_eq!(fecha_local(&reloj(12)), NaiveDate::from_ymd_opt(2026, 11, 7).unwrap_or_default());
    assert_eq!(fecha_local(&reloj(3)), NaiveDate::from_ymd_opt(2026, 11, 7).unwrap_or_default());
}

#[test]
fn paginacion_por_defecto_50_y_tope_100() {
    assert_eq!(paginacion_alertas(None, None).ok(), Some((1, 50, 0)));
    assert_eq!(paginacion_alertas(Some(3), Some(100)).ok(), Some((3, 100, 200)));
    for (pagina, por_pagina) in [(Some(0), None), (None, Some(0)), (None, Some(101))] {
        let error = paginacion_alertas(pagina, por_pagina).err();
        assert_eq!(error.map(|e| e.codigo()), Some("validacion"));
    }
}

#[test]
fn intervalo_de_alertas_por_defecto_cero_y_invalido() {
    assert_eq!(AppConfig::parsear_intervalo_alertas(None).ok(), Some(60));
    assert_eq!(AppConfig::parsear_intervalo_alertas(Some("0")).ok(), Some(0));
    assert_eq!(AppConfig::parsear_intervalo_alertas(Some(" 15 ")).ok(), Some(15));
    let error = AppConfig::parsear_intervalo_alertas(Some("abc")).err();
    assert_eq!(error.map(|e| e.codigo()), Some("validacion"));
}

#[test]
fn el_cuerpo_lista_cada_producto_con_su_faltante() {
    let cuerpo = cuerpo_correo(
        "Gerente",
        &[
            LineaCorreo {
                nombre: "Cemento 50 kg".into(),
                sku: "CEM-50".into(),
                stock_actual: 4,
                stock_minimo: 10,
                faltante: 6,
            },
            LineaCorreo {
                nombre: "Arena".into(),
                sku: "AR-1".into(),
                stock_actual: 0,
                stock_minimo: 3,
                faltante: 3,
            },
        ],
    );
    assert!(cuerpo.starts_with("Hola Gerente:"), "{cuerpo}");
    assert!(
        cuerpo.contains("- Cemento 50 kg (SKU CEM-50): stock 4, mínimo 10, faltan 6"),
        "{cuerpo}"
    );
    assert!(cuerpo.contains("- Arena (SKU AR-1): stock 0, mínimo 3, faltan 3"), "{cuerpo}");
}
