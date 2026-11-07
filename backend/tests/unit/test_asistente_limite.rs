//! 017 — reglas puras del asistente: límite por usuario (FR-068) e historial (FR-077).

use chrono::{Duration, TimeZone, Utc};
use stockmante_backend::services::asistente::{
    recortar_historial, LimiteConsultas, MensajeHistorial, RolMensaje, MAX_CONSULTAS_POR_MINUTO,
    MAX_HISTORIAL,
};

fn limite() -> LimiteConsultas {
    LimiteConsultas::nuevo(MAX_CONSULTAS_POR_MINUTO, Duration::minutes(1))
}

#[test]
fn admite_veinte_consultas_y_rechaza_la_veintiuna() {
    let limite = limite();
    let t0 = Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).single().expect("fecha");
    for i in 0..20 {
        assert!(limite.admitir(1, t0 + Duration::seconds(i)), "consulta {} admitida", i + 1);
    }
    assert!(!limite.admitir(1, t0 + Duration::seconds(30)), "la 21.ª se rechaza");
}

#[test]
fn el_limite_es_por_usuario() {
    let limite = limite();
    let t0 = Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).single().expect("fecha");
    for _ in 0..20 {
        limite.admitir(1, t0);
    }
    assert!(!limite.admitir(1, t0));
    assert!(limite.admitir(2, t0), "otro usuario no se ve afectado");
}

#[test]
fn la_ventana_se_libera_al_pasar_un_minuto() {
    let limite = limite();
    let t0 = Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).single().expect("fecha");
    for _ in 0..20 {
        limite.admitir(1, t0);
    }
    assert!(!limite.admitir(1, t0 + Duration::seconds(59)));
    assert!(limite.admitir(1, t0 + Duration::seconds(60)), "a los 60 s vuelve a admitir");
}

#[test]
fn una_consulta_rechazada_no_consume_cupo() {
    let limite = limite();
    let t0 = Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).single().expect("fecha");
    for _ in 0..20 {
        limite.admitir(1, t0);
    }
    for _ in 0..5 {
        assert!(!limite.admitir(1, t0 + Duration::seconds(30)));
    }
    assert!(limite.admitir(1, t0 + Duration::seconds(60)));
}

fn mensajes(n: usize) -> Vec<MensajeHistorial> {
    (0..n)
        .map(|i| MensajeHistorial {
            rol: if i % 2 == 0 { RolMensaje::Usuario } else { RolMensaje::Asistente },
            texto: format!("m{i}"),
        })
        .collect()
}

#[test]
fn recorta_el_historial_a_los_diez_mas_recientes() {
    let recortado = recortar_historial(mensajes(12));
    assert_eq!(recortado.len(), MAX_HISTORIAL);
    assert_eq!(recortado[0].texto, "m2");
    assert_eq!(recortado[9].texto, "m11");
}

#[test]
fn historial_corto_queda_intacto() {
    let recortado = recortar_historial(mensajes(3));
    assert_eq!(recortado.len(), 3);
    assert_eq!(recortado[0].texto, "m0");
}
