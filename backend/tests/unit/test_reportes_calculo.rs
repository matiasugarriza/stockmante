//! 014 (T004) — reglas puras de reportes: fechas locales, saldos, rotación y CSV.

use chrono::{NaiveDate, TimeZone, Utc};
use stockmante_backend::services::reportes::calculo::{acumular_saldos, calcular_rotacion};
use stockmante_backend::services::reportes::csv::a_csv;
use stockmante_backend::services::reportes::rango::{exigir_rango, limites_utc, parsear_dia};

fn dia(a: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(a, m, d).unwrap_or_default()
}

fn mensaje<T: std::fmt::Debug>(r: Result<T, stockmante_backend::error::AppError>) -> Option<String> {
    r.err().map(|e| e.to_string())
}

#[test]
fn parsear_dia_acepta_fecha_y_rfc3339_en_hora_local() {
    assert_eq!(parsear_dia(Some("2026-10-31"), "desde").ok(), Some(Some(dia(2026, 10, 31))));
    // 02:00 UTC del 1/11 es el 31/10 a las 23:00 en -03:00.
    assert_eq!(
        parsear_dia(Some("2026-11-01T02:00:00Z"), "hasta").ok(),
        Some(Some(dia(2026, 10, 31)))
    );
    assert_eq!(parsear_dia(Some("  "), "desde").ok(), Some(None));
    assert_eq!(parsear_dia(None, "desde").ok(), Some(None));
    assert_eq!(
        mensaje(parsear_dia(Some("31/10/2026"), "hasta")).as_deref(),
        Some("El parámetro hasta debe estar en formato ISO 8601 (AAAA-MM-DD).")
    );
}

#[test]
fn limites_utc_son_semiabiertos_en_menos_tres() {
    let (inicio, fin) = limites_utc(Some(dia(2026, 10, 1)), Some(dia(2026, 10, 31))).unwrap_or_default();
    assert_eq!(inicio, Utc.with_ymd_and_hms(2026, 10, 1, 3, 0, 0).single());
    assert_eq!(fin, Utc.with_ymd_and_hms(2026, 11, 1, 3, 0, 0).single());
    assert_eq!(limites_utc(None, None).ok(), Some((None, None)));
    assert_eq!(
        mensaje(limites_utc(Some(dia(2026, 11, 2)), Some(dia(2026, 11, 1)))).as_deref(),
        Some("La fecha desde no puede ser posterior a la fecha hasta.")
    );
}

#[test]
fn exigir_rango_pide_ambas_fechas() {
    assert_eq!(
        exigir_rango(Some(dia(2026, 10, 1)), Some(dia(2026, 10, 31))).ok(),
        Some((dia(2026, 10, 1), dia(2026, 10, 31)))
    );
    for (d, h) in [(None, Some(dia(2026, 10, 31))), (Some(dia(2026, 10, 1)), None), (None, None)] {
        assert_eq!(
            mensaje(exigir_rango(d, h)).as_deref(),
            Some("Indicá las fechas desde y hasta del período (formato AAAA-MM-DD).")
        );
    }
}

#[test]
fn acumular_saldos_es_el_saldo_corrido() {
    assert_eq!(acumular_saldos(5, &[10, -3, -2, 4]), vec![15, 12, 10, 14]);
    assert!(acumular_saldos(5, &[]).is_empty());
}

#[test]
fn calcular_rotacion_con_promedio_cero_es_cero() {
    let (promedio, indice) = calcular_rotacion(4, 20, 26);
    assert!((promedio - 23.0).abs() < 1e-9);
    assert!((indice - 0.1739).abs() < 1e-4, "{indice}");
    assert_eq!(calcular_rotacion(0, 0, 0), (0.0, 0.0));
    assert_eq!(calcular_rotacion(3, 0, 0), (0.0, 0.0));
}

#[test]
fn a_csv_escapa_y_usa_bom_y_crlf() {
    let csv = a_csv(
        &["producto", "nota"],
        vec![
            vec!["Yerba, 1 kg".to_string(), "dijo \"hola\"".to_string()],
            vec!["Arena".to_string(), "línea\nnueva".to_string()],
        ]
        .into_iter(),
    );
    assert_eq!(
        csv,
        "\u{FEFF}producto,nota\r\n\"Yerba, 1 kg\",\"dijo \"\"hola\"\"\"\r\nArena,\"línea\nnueva\"\r\n"
    );
}
