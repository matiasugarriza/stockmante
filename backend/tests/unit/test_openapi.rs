//! 000 — el documento OpenAPI refleja la superficie real del Módulo 1.
//!
//! No toca la base: valida el JSON generado por `ApiDoc::openapi()`.

use serde_json::Value;
use stockmante_backend::docs::ApiDoc;
use utoipa::OpenApi;

fn documento() -> Value {
    let json = ApiDoc::openapi().to_json().expect("serializar OpenAPI");
    serde_json::from_str(&json).expect("OpenAPI es JSON")
}

#[test]
fn declara_openapi_31() {
    let doc = documento();
    let version = doc["openapi"].as_str().expect("campo openapi");
    assert!(
        version.starts_with("3.1"),
        "se esperaba OpenAPI 3.1, llegó {version}"
    );
}

#[test]
fn lista_todas_las_rutas_del_modulo_1() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    let esperadas = [
        "/api/salud",
        "/api/auth/login",
        "/api/auth/logout",
        "/api/auth/sesion",
        "/api/auth/cambiar-password",
        "/api/usuarios",
        "/api/usuarios/{id}",
        "/api/usuarios/{id}/desactivar",
        "/api/usuarios/{id}/reactivar",
        "/api/usuarios/{id}/password-temporal",
        "/api/categorias",
        "/api/categorias/{id}",
        "/api/categorias/{id}/desactivar",
        "/api/categorias/{id}/reactivar",
        "/api/productos",
        "/api/productos/buscar",
        "/api/productos/ubicaciones",
        "/api/productos/{id}",
        "/api/productos/{id}/desactivar",
        "/api/productos/{id}/reactivar",
        "/api/productos/{id}/ubicaciones",
        "/api/depositos",
        "/api/depositos/{id}",
        "/api/depositos/{id}/ubicaciones",
        "/api/depositos/{id}/desactivar",
        "/api/depositos/{id}/reactivar",
        "/api/ubicaciones",
        "/api/ubicaciones/{id}",
        "/api/ubicaciones/{id}/desactivar",
        "/api/ubicaciones/{id}/reactivar",
        "/api/movimientos",
        "/api/movimientos/{id}",
        "/api/stock",
        "/api/stock/{producto_id}",
    ];
    for ruta in esperadas {
        assert!(paths.contains_key(ruta), "falta la ruta {ruta}");
    }
}

#[test]
fn documenta_los_schemas_de_negocio() {
    let doc = documento();
    let schemas = doc["components"]["schemas"]
        .as_object()
        .expect("components.schemas");
    for schema in [
        "Producto",
        "Categoria",
        "Usuario",
        "ErrorRespuesta",
        "Deposito",
        "Ubicacion",
        "ListaUbicaciones",
        "ListaUbicacionesPorProducto",
        "UbicacionesDeProducto",
        "MovimientoDetalle",
        "ListaMovimientos",
        "StockItem",
        "ListaStock",
        "StockProducto",
        "CrearMovimientoDto",
    ] {
        assert!(schemas.contains_key(schema), "falta el schema {schema}");
    }
}

#[test]
fn declara_el_esquema_de_seguridad_por_cookie() {
    let doc = documento();
    let esquema = &doc["components"]["securitySchemes"]["cookieAuth"];
    assert_eq!(esquema["type"], "apiKey");
    assert_eq!(esquema["in"], "cookie");
    assert_eq!(esquema["name"], "stockmante_sesion");
}

#[test]
fn las_rutas_publicas_no_exigen_cookie() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    for (ruta, metodo) in [
        ("/api/salud", "get"),
        ("/api/auth/login", "post"),
        ("/api/auth/logout", "post"),
    ] {
        let seguridad = &paths[ruta][metodo]["security"];
        // utoipa serializa "sin autenticación" como `[{}]`; `[]` también vale.
        let sin_seguridad = seguridad.as_array().is_some_and(|lista| {
            lista.is_empty()
                || lista
                    .iter()
                    .all(|requisito| requisito.as_object().is_some_and(|obj| obj.is_empty()))
        });
        assert!(
            sin_seguridad,
            "{ruta} debería declarar security sin requisitos; operación: {}",
            paths[ruta][metodo]
        );
    }
}

#[test]
fn las_rutas_protegidas_heredan_cookie_auth() {
    let doc = documento();
    let requisito = &doc["security"][0]["cookieAuth"];
    assert!(
        requisito.as_array().is_some(),
        "el documento debe exigir cookieAuth por defecto"
    );
}

/// 008 (T032): las rutas y los schemas de transferencias están documentados.
#[test]
fn documenta_transferencias() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    for ruta in ["/api/transferencias", "/api/transferencias/{id}"] {
        assert!(paths.contains_key(ruta), "falta la ruta {ruta}");
    }
    assert!(
        paths["/api/transferencias"]["get"].is_object()
            && paths["/api/transferencias"]["post"].is_object(),
        "GET y POST /api/transferencias"
    );
    assert!(paths["/api/transferencias/{id}"]["get"].is_object());
    let schemas = doc["components"]["schemas"].as_object().expect("schemas");
    for schema in ["TransferenciaDetalle", "ListaTransferencias", "CrearTransferenciaDto"] {
        assert!(schemas.contains_key(schema), "falta el schema {schema}");
    }
}

/// 009 (T029): las rutas y los schemas de conteos están documentados.
#[test]
fn documenta_conteos() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    for ruta in [
        "/api/conteos",
        "/api/conteos/{id}",
        "/api/conteos/{id}/lineas",
        "/api/conteos/{id}/cerrar",
        "/api/conteos/{id}/cancelar",
    ] {
        assert!(paths.contains_key(ruta), "falta la ruta {ruta}");
    }
    let schemas = doc["components"]["schemas"].as_object().expect("schemas");
    for schema in ["ConteoDetalle", "ListaConteos"] {
        assert!(schemas.contains_key(schema), "falta el schema {schema}");
    }
}

/// 011 (T024): las rutas y los schemas de alertas están documentados.
#[test]
fn documenta_alertas() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    assert!(paths["/api/alertas"]["get"].is_object(), "GET /api/alertas");
    assert!(paths["/api/alertas/enviar"]["post"].is_object(), "POST /api/alertas/enviar");
    assert!(paths["/api/depositos/activos"]["get"].is_object(), "GET /api/depositos/activos (010)");
    let schemas = doc["components"]["schemas"].as_object().expect("schemas");
    for schema in ["AlertaItem", "ListaAlertas", "RespuestaCiclo"] {
        assert!(schemas.contains_key(schema), "falta el schema {schema}");
    }
}

/// 013 (T014): ventas y clientes documentados, con los 403/409 nuevos.
#[test]
fn documenta_ventas_y_clientes() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    for ruta in [
        "/api/ventas",
        "/api/ventas/{id}",
        "/api/ventas/{id}/anular",
        "/api/clientes",
        "/api/clientes/{id}",
        "/api/clientes/{id}/activar",
        "/api/clientes/{id}/desactivar",
    ] {
        assert!(paths.contains_key(ruta), "falta la ruta {ruta}");
    }
    let anular = &paths["/api/ventas/{id}/anular"]["post"]["responses"];
    assert!(anular["403"].is_object() && anular["409"].is_object(), "{anular}");
    assert!(paths["/api/clientes"]["post"]["responses"]["409"].is_object());
    let schemas = doc["components"]["schemas"].as_object().expect("schemas");
    for schema in ["VentaDetalle", "ListaVentas", "Cliente", "ListaClientes"] {
        assert!(schemas.contains_key(schema), "falta el schema {schema}");
    }
}

/// 016 (T011): la importación y sus schemas están documentados.
#[test]
fn documenta_importacion() {
    let doc = documento();
    let paths = doc["paths"].as_object().expect("paths");
    assert!(paths["/api/importaciones/productos"]["post"].is_object());
    let schemas = doc["components"]["schemas"].as_object().expect("schemas");
    for schema in ["ResumenImportacion", "DetalleImportacion"] {
        assert!(schemas.contains_key(schema), "falta el schema {schema}");
    }
}
