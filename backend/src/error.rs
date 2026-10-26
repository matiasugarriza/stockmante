//! Contrato de errores del Módulo 1 (Principio V de la constitución + contracts/api.md).
//! - Errores de dominio → status 4xx específico + `{ error: { codigo, mensaje } }`
//! - Infraestructura/persistencia → 500 genérico, jamás exponer detalles internos.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Credenciales inválidas")]
    CredencialesInvalidas,
    #[error("Tu sesión expiró. Iniciá sesión nuevamente")]
    SesionExpirada,
    #[error("Necesitás iniciar sesión para continuar")]
    NoAutenticado,
    #[error("No tenés permiso para realizar esta acción")]
    SinPermisos,
    #[error("{0}")]
    RecursoNoEncontrado(String),
    #[error("Ya existe un usuario con ese email")]
    EmailDuplicado,
    #[error("Ya existe un producto con el SKU {0}. Elegí otro.")]
    SkuDuplicado(String),
    #[error("Ya existe un producto con ese código de barras")]
    CodigoBarrasDuplicado,
    #[error("Ya existe una categoría con ese nombre.")]
    CategoriaDuplicada,
    #[error("El depósito tiene ubicaciones o movimientos; desactívalo en lugar de borrarlo")]
    DepositoConDependencias,
    #[error("Ya existe un depósito con ese nombre")]
    DepositoNombreDuplicado,
    #[error("Ya existe un depósito con ese identificador")]
    DepositoIdentificadorDuplicado,
    #[error("La ubicación tiene hijos o productos asignados; desactívala en lugar de borrarla")]
    UbicacionConDependencias,
    #[error("Ya existe una ubicación con ese nombre en el mismo nivel")]
    UbicacionDuplicada,
    #[error("La ubicación padre debe existir en el mismo depósito y estar activa")]
    UbicacionHuerfana,
    #[error("No se puede desactivar al último usuario de gestión activo")]
    UltimoAdministrador,
    #[error("Debés cambiar tu contraseña temporal antes de continuar")]
    DebeCambiarPassword,
    #[error("{0}")]
    Validacion(String),
    #[error("Error interno. Intentá nuevamente o contactá al administrador")]
    Interno(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::CredencialesInvalidas
            | AppError::SesionExpirada
            | AppError::NoAutenticado => StatusCode::UNAUTHORIZED,
            AppError::SinPermisos | AppError::DebeCambiarPassword => StatusCode::FORBIDDEN,
            AppError::RecursoNoEncontrado(_) => StatusCode::NOT_FOUND,
            AppError::EmailDuplicado
            | AppError::SkuDuplicado(_)
            | AppError::CodigoBarrasDuplicado
            | AppError::CategoriaDuplicada
            | AppError::DepositoConDependencias
            | AppError::DepositoNombreDuplicado
            | AppError::DepositoIdentificadorDuplicado
            | AppError::UbicacionConDependencias
            | AppError::UbicacionDuplicada
            | AppError::UltimoAdministrador => StatusCode::CONFLICT,
            AppError::UbicacionHuerfana | AppError::Validacion(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            AppError::Interno(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn codigo(&self) -> &'static str {
        match self {
            AppError::CredencialesInvalidas => "credenciales_invalidas",
            AppError::SesionExpirada => "sesion_expirada",
            AppError::NoAutenticado => "no_autenticado",
            AppError::SinPermisos => "sin_permisos",
            AppError::RecursoNoEncontrado(_) => "recurso_no_encontrado",
            AppError::EmailDuplicado => "email_duplicado",
            AppError::SkuDuplicado(_) => "sku_duplicado",
            AppError::CodigoBarrasDuplicado => "codigo_barras_duplicado",
            AppError::CategoriaDuplicada => "categoria_duplicada",
            AppError::DepositoConDependencias => "deposito_con_dependencias",
            AppError::DepositoNombreDuplicado => "deposito_nombre_duplicado",
            AppError::DepositoIdentificadorDuplicado => "deposito_identificador_duplicado",
            AppError::UbicacionConDependencias => "ubicacion_con_dependencias",
            AppError::UbicacionDuplicada => "ubicacion_duplicada",
            AppError::UbicacionHuerfana => "ubicacion_huerfana",
            AppError::UltimoAdministrador => "ultimo_administrador",
            AppError::DebeCambiarPassword => "cambio_password_requerido",
            AppError::Validacion(_) => "validacion",
            AppError::Interno(_) => "interno",
        }
    }

    pub fn mensaje_publico(&self) -> String {
        self.to_string()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            // El detalle viaja solo al log; al cliente, mensaje genérico.
            tracing::error!(error = %self, "error interno");
        }
        let body = Json(json!({
            "error": {
                "codigo": self.codigo(),
                "mensaje": self.mensaje_publico(),
            }
        }));
        (status, body).into_response()
    }
}

// Infraestructura → 500 genérico (nunca filtrar sqlx::Error al cliente)
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "error de persistencia");
        AppError::Interno("falla de base de datos".into())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        tracing::error!(error = %e, "error argon2");
        AppError::Interno("error de hashing".into())
    }
}
