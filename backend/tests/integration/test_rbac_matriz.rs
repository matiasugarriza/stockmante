//! 006 — matriz RBAC transversal y auditoría básica (FR-019, SC-007, SC-008).
//!
//! Cubre todos los endpoints de escritura y los listados restringidos:
//! OPERACION → 403 `sin_permisos`; GESTION → nunca 403. También verifica que
//! cada acción de gestión completada deja autor y timestamp, y que un 403 no
//! registra una acción de negocio.

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    app_de_test, cuerpo_json, email_unico, nombre_categoria_unico, sesion_con_familia,
    sku_unico, solicitud,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const ID_INEXISTENTE: i32 = 999_999;

async fn pedir(
    app: &Router,
    cookie: &str,
    metodo: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud(metodo, uri, cookie, body))
        .await
        .expect("request de prueba");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

/// Cuerpo vacío para las escrituras: alcanza para probar RBAC sin crear datos.
fn cuerpo(metodo: &str) -> Option<Value> {
    match metodo {
        "POST" | "PUT" => Some(json!({})),
        _ => None,
    }
}

#[tokio::test]
async fn matriz_rbac_de_escritura_y_listados_restringidos() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie_operacion = common::sesion_operacion(&pool).await;
    let cookie_gestion = common::sesion_gestion(&pool).await;

    // (método, ruta) de toda la superficie restringida de 001–005.
    let rutas: &[(&str, String)] = &[
        ("GET", "/api/usuarios".into()),
        ("GET", format!("/api/usuarios/{ID_INEXISTENTE}")),
        ("POST", "/api/usuarios".into()),
        ("PUT", format!("/api/usuarios/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/usuarios/{ID_INEXISTENTE}/desactivar"),
        ),
        (
            "POST",
            format!("/api/usuarios/{ID_INEXISTENTE}/reactivar"),
        ),
        (
            "POST",
            format!("/api/usuarios/{ID_INEXISTENTE}/password-temporal"),
        ),
        ("POST", "/api/categorias".into()),
        ("PUT", format!("/api/categorias/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/categorias/{ID_INEXISTENTE}/desactivar"),
        ),
        (
            "POST",
            format!("/api/categorias/{ID_INEXISTENTE}/reactivar"),
        ),
        ("GET", "/api/productos".into()),
        (
            "GET",
            format!("/api/productos/ubicaciones?producto_ids={ID_INEXISTENTE}"),
        ),
        ("POST", "/api/productos".into()),
        ("PUT", format!("/api/productos/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/productos/{ID_INEXISTENTE}/desactivar"),
        ),
        (
            "POST",
            format!("/api/productos/{ID_INEXISTENTE}/reactivar"),
        ),
        (
            "PUT",
            format!("/api/productos/{ID_INEXISTENTE}/ubicaciones"),
        ),
        (
            "GET",
            format!("/api/productos/{ID_INEXISTENTE}/ubicaciones"),
        ),
        ("GET", "/api/depositos".into()),
        ("POST", "/api/depositos".into()),
        ("PUT", format!("/api/depositos/{ID_INEXISTENTE}")),
        ("DELETE", format!("/api/depositos/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/depositos/{ID_INEXISTENTE}/desactivar"),
        ),
        (
            "POST",
            format!("/api/depositos/{ID_INEXISTENTE}/reactivar"),
        ),
        (
            "GET",
            format!("/api/depositos/{ID_INEXISTENTE}/ubicaciones"),
        ),
        ("GET", format!("/api/ubicaciones/{ID_INEXISTENTE}")),
        ("POST", "/api/ubicaciones".into()),
        ("PUT", format!("/api/ubicaciones/{ID_INEXISTENTE}")),
        ("DELETE", format!("/api/ubicaciones/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/ubicaciones/{ID_INEXISTENTE}/desactivar"),
        ),
        (
            "POST",
            format!("/api/ubicaciones/{ID_INEXISTENTE}/reactivar"),
        ),
        // 012 — proveedores y compras.
        ("GET", "/api/proveedores".into()),
        ("POST", "/api/proveedores".into()),
        ("GET", format!("/api/proveedores/{ID_INEXISTENTE}")),
        ("PUT", format!("/api/proveedores/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/proveedores/{ID_INEXISTENTE}/activar"),
        ),
        (
            "POST",
            format!("/api/proveedores/{ID_INEXISTENTE}/desactivar"),
        ),
        ("GET", "/api/compras".into()),
        ("POST", "/api/compras".into()),
        ("GET", format!("/api/compras/{ID_INEXISTENTE}")),
        ("PUT", format!("/api/compras/{ID_INEXISTENTE}")),
        ("DELETE", format!("/api/compras/{ID_INEXISTENTE}")),
        (
            "POST",
            format!("/api/compras/{ID_INEXISTENTE}/confirmar"),
        ),
        ("POST", format!("/api/compras/{ID_INEXISTENTE}/anular")),
        // 013 (T010) — anular ventas y escribir clientes es GESTION.
        ("POST", format!("/api/ventas/{ID_INEXISTENTE}/anular")),
        ("POST", "/api/clientes".into()),
        ("PUT", format!("/api/clientes/{ID_INEXISTENTE}")),
        ("POST", format!("/api/clientes/{ID_INEXISTENTE}/activar")),
        ("POST", format!("/api/clientes/{ID_INEXISTENTE}/desactivar")),
        // 014 (T006) — los cuatro reportes, en JSON y CSV, son GESTION (SC-031).
        ("GET", format!("/api/reportes/kardex?producto_id={ID_INEXISTENTE}")),
        ("GET", format!("/api/reportes/kardex?producto_id={ID_INEXISTENTE}&formato=csv")),
        ("GET", "/api/reportes/stock-valorizado".into()),
        ("GET", "/api/reportes/stock-valorizado?formato=csv".into()),
        ("GET", "/api/reportes/rotacion?desde=2026-10-01&hasta=2026-10-31".into()),
        ("GET", "/api/reportes/rotacion?desde=2026-10-01&hasta=2026-10-31&formato=csv".into()),
        ("GET", "/api/reportes/ventas?desde=2026-10-01&hasta=2026-10-31".into()),
        ("GET", "/api/reportes/ventas?desde=2026-10-01&hasta=2026-10-31&formato=csv".into()),
    ];

    for (metodo, ruta) in rutas {
        let (status, body) = pedir(
            &app,
            &cookie_operacion,
            metodo,
            ruta,
            cuerpo(metodo),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "OPERACION debe recibir 403 en {metodo} {ruta}: {body}"
        );
        assert_eq!(
            body["error"]["codigo"], "sin_permisos",
            "{metodo} {ruta}: {body}"
        );
    }

    // GESTION pasa el control de acceso: nunca 401/403, aunque el recurso no
    // exista o el cuerpo esté incompleto (404/422 son respuestas válidas).
    for (metodo, ruta) in rutas {
        let (status, body) = pedir(&app, &cookie_gestion, metodo, ruta, cuerpo(metodo)).await;
        assert_ne!(
            status,
            StatusCode::FORBIDDEN,
            "GESTION no debe recibir 403 en {metodo} {ruta}: {body}"
        );
        assert_ne!(
            status,
            StatusCode::UNAUTHORIZED,
            "GESTION no debe recibir 401 en {metodo} {ruta}: {body}"
        );
    }
}

async fn id_de(pool: &PgPool, email: &str) -> i32 {
    sqlx::query_scalar("SELECT id FROM usuarios WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("id del usuario de sesión")
}

async fn acciones(pool: &PgPool, tabla: &str, registro_id: i32) -> Vec<(String, Option<i32>)> {
    sqlx::query_as::<_, (String, Option<i32>)>(
        r#"SELECT accion, usuario_id
           FROM auditoria
           WHERE tabla_ref = $1 AND registro_id = $2
           ORDER BY id"#,
    )
    .bind(tabla)
    .bind(registro_id)
    .fetch_all(pool)
    .await
    .expect("filas de auditoría")
}

/// Alta de categoría por HTTP con una sesión GESTION.
async fn crear_categoria(app: &Router, cookie: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/categorias",
        Some(json!({ "nombre": nombre_categoria_unico("RBAC") })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "categoría: {body}");
    body["id"].as_i64().expect("id de categoría") as i32
}

#[tokio::test]
async fn audita_altas_cambios_y_bajas_de_productos() {
    let (app, pool, _correo) = app_de_test().await;
    let (email_gestor, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let actor = id_de(&pool, &email_gestor).await;
    let categoria_id = crear_categoria(&app, &cookie).await;

    let (status, producto) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/productos",
        Some(json!({
            "nombre": "Producto auditado",
            "sku": sku_unico("audit"),
            "categoria_id": categoria_id,
            "unidad_medida": "UNIDAD",
            "costo": 100,
            "precio_venta": 150,
            "stock_minimo": 0,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{producto}");
    let id = producto["id"].as_i64().expect("id de producto") as i32;

    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/productos/{id}"),
        Some(json!({ "precio_venta": 175 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, _) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/productos/{id}/desactivar"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = pedir(
        &app,
        &cookie,
        "POST",
        &format!("/api/productos/{id}/reactivar"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let filas = acciones(&pool, "productos", id).await;
    let acciones_esperadas = ["CREAR", "ACTUALIZAR", "DESACTIVAR", "REACTIVAR"];
    assert_eq!(filas.len(), acciones_esperadas.len(), "{filas:?}");
    for (fila, esperada) in filas.iter().zip(acciones_esperadas) {
        assert_eq!(fila.0, esperada);
        assert_eq!(fila.1, Some(actor), "el autor debe ser el gestor de la sesión");
    }

    let con_timestamp: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM auditoria
           WHERE tabla_ref = 'productos' AND registro_id = $1 AND cuando IS NOT NULL"#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("timestamps de auditoría");
    assert_eq!(con_timestamp, 4);
}

#[tokio::test]
async fn audita_altas_cambios_y_bajas_de_usuarios() {
    let (app, pool, _correo) = app_de_test().await;
    let (email_gestor, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let actor = id_de(&pool, &email_gestor).await;
    let email_objetivo = email_unico("auditado");

    let (status, creado) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/usuarios",
        Some(json!({
            "nombre": "Usuario auditado",
            "email": email_objetivo,
            "rol": "OPERARIO",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{creado}");
    let id = creado["usuario"]["id"].as_i64().expect("id de usuario") as i32;

    let (status, body) = pedir(
        &app,
        &cookie,
        "PUT",
        &format!("/api/usuarios/{id}"),
        Some(json!({ "nombre": "Usuario auditado II" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    for accion in ["desactivar", "reactivar", "password-temporal"] {
        let (status, body) = pedir(
            &app,
            &cookie,
            "POST",
            &format!("/api/usuarios/{id}/{accion}"),
            None,
        )
        .await;
        // `password-temporal` pasó de 204 a 200: devuelve la clave para la
        // modal (contrato 2026-11, FR-008b); el resto sigue en 204.
        let esperado = if accion == "password-temporal" {
            StatusCode::OK
        } else {
            StatusCode::NO_CONTENT
        };
        assert_eq!(status, esperado, "{accion}: {body}");
    }

    let filas = acciones(&pool, "usuarios", id).await;
    let acciones_esperadas = [
        "CREAR",
        "ACTUALIZAR",
        "DESACTIVAR",
        "REACTIVAR",
        "PASSWORD_TEMPORAL",
    ];
    assert_eq!(filas.len(), acciones_esperadas.len(), "{filas:?}");
    for (fila, esperada) in filas.iter().zip(acciones_esperadas) {
        assert_eq!(fila.0, esperada);
        assert_eq!(fila.1, Some(actor));
    }

    // Limpieza: la auditoría referencia `registro_id` sin FK y conserva la fila.
    sqlx::query("DELETE FROM usuarios WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("limpieza del usuario auditado");
}

#[tokio::test]
async fn un_403_no_registra_auditoria() {
    let (app, pool, _correo) = app_de_test().await;
    let (email_operacion, cookie) = sesion_con_familia(&pool, "CAJERO", "OPERACION").await;
    let actor = id_de(&pool, &email_operacion).await;

    for (metodo, uri, body) in [
        (
            "POST",
            "/api/productos",
            Some(json!({
                "nombre": "No permitido",
                "sku": sku_unico("rbac"),
                "categoria_id": 1,
                "unidad_medida": "UNIDAD",
                "costo": 1,
                "precio_venta": 2,
            })),
        ),
        (
            "POST",
            "/api/usuarios",
            Some(json!({
                "nombre": "No permitido",
                "email": email_unico("rbac"),
                "rol": "OPERARIO",
            })),
        ),
        ("POST", "/api/categorias", Some(json!({ "nombre": "No permitida" }))),
    ] {
        let (status, respuesta) = pedir(&app, &cookie, metodo, uri, body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{metodo} {uri}: {respuesta}");
    }

    let auditadas: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auditoria WHERE usuario_id = $1")
            .bind(actor)
            .fetch_one(&pool)
            .await
            .expect("conteo de auditoría del operario");
    assert_eq!(auditadas, 0, "un 403 no debe dejar acciones auditadas");
}

/// T020: la lectura de caja sigue disponible para OPERACION (US6 escenario 2).
#[tokio::test]
async fn operacion_conserva_las_lecturas_de_caja() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie_gestion = common::sesion_gestion(&pool).await;
    let cookie_operacion = common::sesion_operacion(&pool).await;
    let categoria_id = crear_categoria(&app, &cookie_gestion).await;
    let (status, producto) = pedir(
        &app,
        &cookie_gestion,
        "POST",
        "/api/productos",
        Some(json!({
            "nombre": "Producto lectura caja",
            "sku": sku_unico("caja"),
            "categoria_id": categoria_id,
            "unidad_medida": "UNIDAD",
            "costo": 10,
            "precio_venta": 20,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{producto}");
    let id = producto["id"].as_i64().expect("id de producto");

    for ruta in [
        format!("/api/productos/{id}"),
        "/api/productos/buscar?q=lectura".to_string(),
        "/api/categorias".to_string(),
        format!("/api/categorias/{categoria_id}"),
    ] {
        let (status, body) = pedir(&app, &cookie_operacion, "GET", &ruta, None).await;
        assert_eq!(status, StatusCode::OK, "OPERACION debe leer {ruta}: {body}");
    }
}

/// T021: degradar el rol con la sesión viva corta el acceso en el siguiente request.
#[tokio::test]
async fn degradar_el_rol_corta_el_acceso_en_el_siguiente_request() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let (status, body) = pedir(&app, &cookie, "GET", "/api/usuarios", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    sqlx::query(
        "UPDATE usuarios SET rol = 'OPERARIO', familia_rol = 'OPERACION' WHERE email = $1",
    )
    .bind(&email)
    .execute(&pool)
    .await
    .expect("degradar el rol");

    let (status, body) = pedir(&app, &cookie, "GET", "/api/usuarios", None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["codigo"], "sin_permisos");
}

async fn crear_deposito(app: &Router, cookie: &str) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/depositos",
        Some(json!({
            "nombre": sku_unico("Deposito"),
            "identificador": sku_unico("DEP"),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "depósito: {body}");
    body["id"].as_i64().expect("id de depósito") as i32
}

async fn crear_ubicacion(app: &Router, cookie: &str, deposito_id: i32, padre: Option<i32>) -> i32 {
    let (status, body) = pedir(
        app,
        cookie,
        "POST",
        "/api/ubicaciones",
        Some(json!({
            "deposito_id": deposito_id,
            "padre_id": padre,
            "nombre": sku_unico("U"),
            "tipo": "PASILLO",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "ubicación: {body}");
    body["id"].as_i64().expect("id de ubicación") as i32
}

async fn accion_ok(app: &Router, cookie: &str, metodo: &str, uri: &str, body: Option<Value>) {
    let (status, respuesta) = pedir(app, cookie, metodo, uri, body).await;
    assert!(
        status.is_success(),
        "{metodo} {uri} debía ser 2xx y fue {status}: {respuesta}"
    );
}

fn assert_acciones(filas: &[(String, Option<i32>)], esperadas: &[&str], actor: i32) {
    let obtenidas: Vec<&str> = filas.iter().map(|f| f.0.as_str()).collect();
    assert_eq!(obtenidas, esperadas, "{filas:?}");
    assert!(
        filas.iter().all(|f| f.1 == Some(actor)),
        "el autor debe ser el gestor de la sesión: {filas:?}"
    );
}

/// T012/T017: categorías auditadas con autor y timestamp.
#[tokio::test]
async fn audita_altas_cambios_y_bajas_de_categorias() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let actor = id_de(&pool, &email).await;
    let id = crear_categoria(&app, &cookie).await;
    accion_ok(
        &app,
        &cookie,
        "PUT",
        &format!("/api/categorias/{id}"),
        Some(json!({ "descripcion": "editada" })),
    )
    .await;
    accion_ok(&app, &cookie, "POST", &format!("/api/categorias/{id}/desactivar"), None).await;
    accion_ok(&app, &cookie, "POST", &format!("/api/categorias/{id}/reactivar"), None).await;

    let filas = acciones(&pool, "categorias", id).await;
    assert_acciones(&filas, &["CREAR", "ACTUALIZAR", "DESACTIVAR", "REACTIVAR"], actor);
    let sin_timestamp: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM auditoria \
         WHERE tabla_ref = 'categorias' AND registro_id = $1 AND cuando IS NULL",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("timestamps");
    assert_eq!(sin_timestamp, 0);
}

/// T014/T015/T017: depósitos y ubicaciones, incluido el borrado físico permitido.
#[tokio::test]
async fn audita_depositos_y_ubicaciones() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let actor = id_de(&pool, &email).await;

    let deposito = crear_deposito(&app, &cookie).await;
    let dep_uri = format!("/api/depositos/{deposito}");
    accion_ok(&app, &cookie, "PUT", &dep_uri, Some(json!({ "direccion": "Calle 1" }))).await;
    accion_ok(&app, &cookie, "POST", &format!("{dep_uri}/desactivar"), None).await;
    accion_ok(&app, &cookie, "POST", &format!("{dep_uri}/reactivar"), None).await;

    let ubicacion = crear_ubicacion(&app, &cookie, deposito, None).await;
    let ubi_uri = format!("/api/ubicaciones/{ubicacion}");
    accion_ok(&app, &cookie, "PUT", &ubi_uri, Some(json!({ "tipo": "ESTANTERIA" }))).await;
    accion_ok(&app, &cookie, "POST", &format!("{ubi_uri}/desactivar"), None).await;
    accion_ok(&app, &cookie, "POST", &format!("{ubi_uri}/reactivar"), None).await;
    accion_ok(&app, &cookie, "DELETE", &ubi_uri, None).await;
    accion_ok(&app, &cookie, "DELETE", &dep_uri, None).await;

    let esperadas = ["CREAR", "ACTUALIZAR", "DESACTIVAR", "REACTIVAR", "ELIMINAR"];
    assert_acciones(&acciones(&pool, "ubicaciones", ubicacion).await, &esperadas, actor);
    assert_acciones(&acciones(&pool, "depositos", deposito).await, &esperadas, actor);
}

/// T016/T017: la asignación se audita sobre el producto, también al limpiar.
#[tokio::test]
async fn audita_la_asignacion_de_ubicaciones_incluso_vacia() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let actor = id_de(&pool, &email).await;
    let categoria_id = crear_categoria(&app, &cookie).await;
    let (status, producto) = pedir(
        &app,
        &cookie,
        "POST",
        "/api/productos",
        Some(json!({
            "nombre": "Producto con ubicaciones",
            "sku": sku_unico("asig"),
            "categoria_id": categoria_id,
            "unidad_medida": "UNIDAD",
            "costo": 1,
            "precio_venta": 2,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{producto}");
    let id = producto["id"].as_i64().expect("id") as i32;
    let deposito = crear_deposito(&app, &cookie).await;
    let ubicacion = crear_ubicacion(&app, &cookie, deposito, None).await;

    let uri = format!("/api/productos/{id}/ubicaciones");
    accion_ok(&app, &cookie, "PUT", &uri, Some(json!({ "ubicacion_ids": [ubicacion] }))).await;
    accion_ok(&app, &cookie, "PUT", &uri, Some(json!({ "ubicacion_ids": [] }))).await;

    let filas = acciones(&pool, "productos", id).await;
    assert_acciones(
        &filas,
        &["CREAR", "ASIGNAR_UBICACIONES", "ASIGNAR_UBICACIONES"],
        actor,
    );
}

/// T022 (SC-007): borrar con dependencias es 409 y no audita; la baja lógica es 204.
#[tokio::test]
async fn borrar_con_dependencias_es_409_y_desactivar_es_la_alternativa() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "ADMIN", "GESTION").await;
    let actor = id_de(&pool, &email).await;
    let deposito = crear_deposito(&app, &cookie).await;
    let padre = crear_ubicacion(&app, &cookie, deposito, None).await;
    let _hija = crear_ubicacion(&app, &cookie, deposito, Some(padre)).await;
    let dep_uri = format!("/api/depositos/{deposito}");
    let padre_uri = format!("/api/ubicaciones/{padre}");

    let (status, body) = pedir(&app, &cookie, "DELETE", &dep_uri, None).await;
    assert_eq!(status, StatusCode::CONFLICT, "depósito con ubicaciones: {body}");
    let (status, body) = pedir(&app, &cookie, "DELETE", &padre_uri, None).await;
    assert_eq!(status, StatusCode::CONFLICT, "ubicación con hijos: {body}");

    let (status, _) = pedir(&app, &cookie, "POST", &format!("{padre_uri}/desactivar"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = pedir(&app, &cookie, "POST", &format!("{dep_uri}/desactivar"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let eliminadas: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM auditoria WHERE usuario_id = $1 AND accion = 'ELIMINAR'",
    )
    .bind(actor)
    .fetch_one(&pool)
    .await
    .expect("conteo de ELIMINAR");
    assert_eq!(eliminadas, 0, "un 409 no debe auditar");
}

/// T017: un 403 sobre las nuevas superficies no deja filas.
#[tokio::test]
async fn un_403_en_depositos_ubicaciones_y_asignaciones_no_audita() {
    let (app, pool, _correo) = app_de_test().await;
    let (email, cookie) = sesion_con_familia(&pool, "CAJERO", "OPERACION").await;
    let actor = id_de(&pool, &email).await;
    let casos: Vec<(&str, &str, Option<Value>)> = vec![
        ("POST", "/api/depositos", Some(json!({ "nombre": "x", "identificador": "x" }))),
        (
            "POST",
            "/api/ubicaciones",
            Some(json!({ "deposito_id": 1, "nombre": "x", "tipo": "PASILLO" })),
        ),
        ("DELETE", "/api/depositos/1", None),
        ("DELETE", "/api/ubicaciones/1", None),
        ("PUT", "/api/productos/1/ubicaciones", Some(json!({ "ubicacion_ids": [] }))),
        ("PUT", "/api/categorias/1", Some(json!({ "nombre": "x" }))),
    ];
    for (metodo, uri, body) in casos {
        let (status, respuesta) = pedir(&app, &cookie, metodo, uri, body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{metodo} {uri}: {respuesta}");
    }
    let auditadas: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auditoria WHERE usuario_id = $1")
            .bind(actor)
            .fetch_one(&pool)
            .await
            .expect("conteo");
    assert_eq!(auditadas, 0);
}
