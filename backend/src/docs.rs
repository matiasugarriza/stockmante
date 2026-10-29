//! Documentación viva de la API (000).
//!
//! `ApiDoc` se genera desde las anotaciones `#[utoipa::path]` de los handlers y
//! los `ToSchema` de los modelos. Se sirve en `/docs` (Swagger UI) y el JSON en
//! `/api-docs/openapi.json`; `app.rs` decide si se monta según `DOCS_ENABLED`.
//!
//! El contrato de negocio (mensajes, códigos, reglas) sigue viviendo en
//! `specs/*/contracts/api.md`; acá solo se refleja lo implementado.

use crate::app::AppEstado;
use axum::Router;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// Sobre de error del Módulo 1 (`{ error: { codigo, mensaje } }`).
#[derive(Debug, ToSchema)]
pub struct ErrorRespuesta {
    pub error: ErrorDetalle,
}

#[derive(Debug, ToSchema)]
pub struct ErrorDetalle {
    /// Código estable para el frontend (`validacion`, `sin_permisos`, …).
    #[schema(example = "validacion")]
    pub codigo: String,
    /// Mensaje accionable en español.
    #[schema(example = "El SKU debe tener entre 3 y 30 caracteres.")]
    pub mensaje: String,
}

/// Agrega `cookieAuth` al documento: la sesión viaja en la cookie HttpOnly.
struct AgregarSeguridadCookie;

impl Modify for AgregarSeguridadCookie {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let componentes = openapi.components.get_or_insert_with(Default::default);
        componentes.add_security_scheme(
            "cookieAuth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("stockmante_sesion"))),
        );
    }
}

/// Documento OpenAPI 3.1 de todo el backend.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Stockmante API",
        version = "0.1.0",
        description = "API del Módulo 1: autenticación, usuarios, catálogo, búsqueda, depósitos, ubicaciones y auditoría. Los errores usan el sobre `{ error: { codigo, mensaje } }`.",
        contact(name = "Stockmante", url = "http://localhost:5173")
    ),
    paths(
        crate::handlers::salud,
        crate::handlers::auth::login,
        crate::handlers::auth::logout,
        crate::handlers::auth::sesion,
        crate::handlers::auth::cambiar_password,
        crate::handlers::auditoria::listar,
        crate::handlers::usuarios::listar,
        crate::handlers::usuarios::crear,
        crate::handlers::usuarios::obtener,
        crate::handlers::usuarios::actualizar,
        crate::handlers::usuarios::desactivar,
        crate::handlers::usuarios::reactivar,
        crate::handlers::usuarios::password_temporal,
        crate::handlers::categorias::listar,
        crate::handlers::categorias::crear,
        crate::handlers::categorias::obtener,
        crate::handlers::categorias::actualizar,
        crate::handlers::categorias::desactivar,
        crate::handlers::categorias::reactivar,
        crate::handlers::conteos::crear,
        crate::handlers::conteos::listar,
        crate::handlers::conteos::obtener,
        crate::handlers::conteos::cargar_lineas,
        crate::handlers::conteos::cerrar,
        crate::handlers::conteos::cancelar,
        crate::handlers::productos::listar,
        crate::handlers::productos::buscar,
        crate::handlers::productos::crear,
        crate::handlers::productos::obtener,
        crate::handlers::productos::actualizar,
        crate::handlers::productos::desactivar,
        crate::handlers::productos::reactivar,
        crate::handlers::productos::ubicaciones,
        crate::handlers::productos::reemplazar_ubicaciones,
        crate::handlers::depositos::listar,
        crate::handlers::depositos::crear,
        crate::handlers::depositos::obtener,
        crate::handlers::depositos::actualizar,
        crate::handlers::depositos::eliminar,
        crate::handlers::depositos::listar_ubicaciones,
        crate::handlers::depositos::desactivar,
        crate::handlers::depositos::reactivar,
        crate::handlers::ubicaciones::crear,
        crate::handlers::ubicaciones::obtener,
        crate::handlers::ubicaciones::actualizar,
        crate::handlers::ubicaciones::eliminar,
        crate::handlers::ubicaciones::desactivar,
        crate::handlers::ubicaciones::reactivar,
        crate::handlers::movimientos::crear,
        crate::handlers::movimientos::listar,
        crate::handlers::movimientos::obtener,
        crate::handlers::transferencias::crear,
        crate::handlers::transferencias::listar,
        crate::handlers::transferencias::obtener,
        crate::handlers::stock::listar,
        crate::handlers::stock::detalle
    ),
    components(schemas(
        ErrorRespuesta,
        ErrorDetalle,
        crate::models::usuario::Usuario,
        crate::models::producto::Categoria,
        crate::models::producto::CategoriaBreve,
        crate::models::producto::Producto,
        crate::models::producto::ListaProductos,
        crate::models::producto::ListaBusqueda,
        crate::models::deposito::Deposito,
        crate::models::deposito::Ubicacion,
        crate::models::deposito::ListaUbicaciones,
        crate::models::stock::DepositoBreve,
        crate::models::stock::UsuarioBreve,
        crate::models::stock::LineaMovimiento,
        crate::models::stock::MovimientoDetalle,
        crate::models::stock::ListaMovimientos,
        crate::models::stock::ConteoDetalle,
        crate::models::stock::ListaConteos,
        crate::models::stock::TransferenciaDetalle,
        crate::models::stock::ListaTransferencias,
        crate::models::stock::StockItem,
        crate::models::stock::ListaStock,
        crate::models::stock::DesgloseStock,
        crate::models::stock::StockProducto,
        crate::services::usuarios::ListaUsuarios,
        crate::handlers::auth::LoginDto,
        crate::handlers::auth::CambioPasswordDto,
        crate::handlers::auth::UsuarioSesionDto,
        crate::handlers::auth::SesionDto,
        crate::handlers::auditoria::ListaAuditoria,
        crate::handlers::auditoria::AuditoriaItem,
        crate::handlers::usuarios::CrearUsuarioDto,
        crate::handlers::usuarios::ActualizarUsuarioDto,
        crate::handlers::categorias::CrearCategoriaDto,
        crate::handlers::categorias::ActualizarCategoriaDto,
        crate::handlers::productos::CrearProductoDto,
        crate::handlers::productos::ActualizarProductoDto,
        crate::handlers::productos::AsignarUbicacionesDto,
        crate::handlers::depositos::CrearDepositoDto,
        crate::handlers::depositos::ActualizarDepositoDto,
        crate::handlers::ubicaciones::CrearUbicacionDto,
        crate::handlers::ubicaciones::ActualizarUbicacionDto,
        crate::handlers::movimientos::CrearMovimientoDto,
        crate::handlers::movimientos::LineaMovimientoDto
    )),
    modifiers(&AgregarSeguridadCookie),
    security(("cookieAuth" = [])),
    tags(
        (name = "auth", description = "Login, logout, sesión y contraseña"),
        (name = "usuarios", description = "Administración de usuarios (GESTION)"),
        (name = "categorias", description = "Categorías del catálogo"),
        (name = "productos", description = "Catálogo y búsqueda de caja"),
        (name = "depositos", description = "Depósitos y sus ubicaciones (GESTION)"),
        (name = "ubicaciones", description = "Ubicaciones jerárquicas (GESTION)"),
        (name = "movimientos", description = "Entradas, salidas, mermas y ajustes"),
        (name = "stock", description = "Consulta de stock y desglose por depósito")
    )
)]
pub struct ApiDoc;

/// Router de Swagger UI. Solo se monta cuando `DOCS_ENABLED` está encendido.
pub fn router() -> Router<AppEstado> {
    Router::new().merge(
        SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()),
    )
}
