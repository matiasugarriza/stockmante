//! Servicios de dominio.

pub mod auditoria;
pub mod auth;
pub mod categorias;
pub mod depositos;
pub mod movimientos;
pub mod productos;
pub mod stock;
pub mod usuarios;

use crate::error::AppError;

/// Paginación compartida por los listados nuevos (007 en adelante).
pub(crate) const POR_PAGINA_DEFAULT: i64 = 20;
pub(crate) const POR_PAGINA_MAX: i64 = 100;

pub(crate) fn normalizar_paginacion(
    pagina: Option<i64>,
    por_pagina: Option<i64>,
) -> Result<(i64, i64, i64), AppError> {
    let pagina = pagina.unwrap_or(1);
    let por_pagina = por_pagina.unwrap_or(POR_PAGINA_DEFAULT);
    if pagina < 1 {
        return Err(AppError::Validacion(
            "La página debe ser mayor o igual a 1".into(),
        ));
    }
    if !(1..=POR_PAGINA_MAX).contains(&por_pagina) {
        return Err(AppError::Validacion(format!(
            "por_pagina debe estar entre 1 y {POR_PAGINA_MAX}"
        )));
    }
    let offset = pagina
        .checked_sub(1)
        .and_then(|previas| previas.checked_mul(por_pagina))
        .ok_or_else(|| AppError::Validacion("La página es demasiado grande".into()))?;
    Ok((pagina, por_pagina, offset))
}
