//! Fechas de los reportes (014, R2): días locales UTC−03:00, rango inclusivo convertido a
//! límites UTC semiabiertos `[desde 00:00, hasta+1 00:00)`. Sin reloj: el servicio es
//! determinista.

use crate::error::AppError;
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, TimeZone, Utc};

fn offset_local() -> Option<FixedOffset> {
    FixedOffset::west_opt(3 * 3600)
}

/// `AAAA-MM-DD`, o RFC 3339 tomado en hora local. Vacío = sin fecha.
pub fn parsear_dia(valor: Option<&str>, campo: &str) -> Result<Option<NaiveDate>, AppError> {
    let Some(texto) = valor.map(str::trim).filter(|t| !t.is_empty()) else {
        return Ok(None);
    };
    if let Ok(dia) = NaiveDate::parse_from_str(texto, "%Y-%m-%d") {
        return Ok(Some(dia));
    }
    if let (Ok(instante), Some(offset)) = (DateTime::parse_from_rfc3339(texto), offset_local()) {
        return Ok(Some(instante.with_timezone(&offset).date_naive()));
    }
    Err(AppError::Validacion(format!(
        "El parámetro {campo} debe estar en formato ISO 8601 (AAAA-MM-DD)."
    )))
}

fn inicio_del_dia(dia: NaiveDate) -> Result<DateTime<Utc>, AppError> {
    let offset = offset_local().ok_or_else(|| AppError::Interno("offset local inválido".into()))?;
    let medianoche = dia
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::Validacion("Fecha fuera de rango.".into()))?;
    offset
        .from_local_datetime(&medianoche)
        .single()
        .map(|f| f.with_timezone(&Utc))
        .ok_or_else(|| AppError::Validacion("Fecha fuera de rango.".into()))
}

/// Límites UTC semiabiertos del rango local inclusivo; valida `desde <= hasta`.
pub fn limites_utc(
    desde: Option<NaiveDate>,
    hasta: Option<NaiveDate>,
) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>), AppError> {
    if let (Some(d), Some(h)) = (desde, hasta) {
        if d > h {
            return Err(AppError::Validacion(
                "La fecha desde no puede ser posterior a la fecha hasta.".into(),
            ));
        }
    }
    let inicio = desde.map(inicio_del_dia).transpose()?;
    let fin = hasta
        .map(|h| {
            h.checked_add_signed(Duration::days(1))
                .ok_or_else(|| AppError::Validacion("Fecha fuera de rango.".into()))
                .and_then(inicio_del_dia)
        })
        .transpose()?;
    Ok((inicio, fin))
}

/// Rotación y ventas exigen el período completo.
pub fn exigir_rango(
    desde: Option<NaiveDate>,
    hasta: Option<NaiveDate>,
) -> Result<(NaiveDate, NaiveDate), AppError> {
    match (desde, hasta) {
        (Some(d), Some(h)) => Ok((d, h)),
        _ => Err(AppError::Validacion(
            "Indicá las fechas desde y hasta del período (formato AAAA-MM-DD).".into(),
        )),
    }
}
