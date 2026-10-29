pub mod auth;
pub mod categorias;
pub mod depositos;
pub mod movimientos;
pub mod productos;
pub mod stock;
pub mod ubicaciones;
pub mod usuarios;

use crate::error::AppError;
use axum::Json;
use serde::{Deserialize, Deserializer};
use serde_json::json;

/// GET /api/salud → 200 `{ "ok": true }` (pública, sin sesión).
#[utoipa::path(
    get,
    path = "/api/salud",
    tag = "salud",
    summary = "Sonda de salud",
    responses((status = 200, description = "Servicio arriba")),
    security(())
)]
pub async fn salud() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}

/// `?activo=true|false|all` compartido por usuarios y catálogo.
///
/// `por_defecto` fija qué significa "sin filtro": usuarios lista todas
/// (`None`), catálogo lista solo activas (`Some(true)`). Un valor desconocido
/// es 422 en ambos casos.
pub(crate) fn parsear_activo(
    valor: Option<&str>,
    por_defecto: Option<bool>,
) -> Result<Option<bool>, AppError> {
    match valor.map(|v| v.trim().to_lowercase()).as_deref() {
        None | Some("") => Ok(por_defecto),
        Some("all") => Ok(None),
        Some("true") => Ok(Some(true)),
        Some("false") => Ok(Some(false)),
        Some(_) => Err(AppError::Validacion(
            "El filtro activo debe ser true, false o all".into(),
        )),
    }
}

/// Serde colapsa `{}` y `{"campo": null}` en `None` para `Option<T>`. Este
/// `deserialize_with` devuelve `Some(None)` cuando el campo está presente con
/// `null` y deja `None` solo cuando el campo está ausente: así el `PUT`
/// distingue "limpiar" de "conservar".
pub(crate) fn deserializar_presente<'de, D, T>(
    deserializador: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializador).map(Some)
}
