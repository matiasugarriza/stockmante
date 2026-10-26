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
    let id = creado["id"].as_i64().expect("id de usuario") as i32;

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
        assert_eq!(status, StatusCode::NO_CONTENT, "{accion}: {body}");
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
