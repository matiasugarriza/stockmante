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
