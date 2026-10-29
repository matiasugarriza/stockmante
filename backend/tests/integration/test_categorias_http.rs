//! 003/T011 — integración HTTP de /api/categorias contra el router real
//! (contracts/api.md, CU-CAT-01…06 y la mitad de CU-RET-01).

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::{header, StatusCode};
use common::{
    app_de_test, cuerpo_json, nombre_categoria_unico, sesion_gestion, sesion_operacion, solicitud,
    solicitud_anonima,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn crear_categoria(
    app: &axum::Router,
    cookie: &str,
    nombre: &str,
    descripcion: Option<&str>,
) -> (StatusCode, Value) {
    let mut body = json!({ "nombre": nombre });
    if let Some(texto) = descripcion {
        body["descripcion"] = json!(texto);
    }
    let res = app
        .clone()
        .oneshot(solicitud("POST", "/api/categorias", cookie, Some(body)))
        .await
        .expect("alta de categoría");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

fn id_de(body: &Value) -> i32 {
    body["id"].as_i64().expect("id de la categoría") as i32
}

#[tokio::test]
async fn alta_detalle_y_edicion_parcial() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let nombre = nombre_categoria_unico("Almacén");

    let (status, body) = crear_categoria(&app, &cookie, &format!("  {nombre}  "), Some("  Secos  ")).await;
    assert_eq!(status, StatusCode::CREATED, "alta válida: {body}");
    assert_eq!(body["nombre"], nombre);
    assert_eq!(body["descripcion"], "Secos");
    assert_eq!(body["activo"], true);
    let id = id_de(&body);

    // Unicidad sin distinguir mayúsculas ni espacios sobrantes.
    let (status, body) = crear_categoria(&app, &cookie, &format!("  {}  ", nombre.to_uppercase()), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["codigo"], "categoria_duplicada");

    // `descripcion` vacía se normaliza a null; ausente también.
    let otra = nombre_categoria_unico("Bebidas");
    let (status, body) = crear_categoria(&app, &cookie, &otra, Some("")).await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["descripcion"].is_null());

    // Detalle.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/categorias/{id}"),
            &cookie,
            None,
        ))
        .await
        .expect("detalle");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["id"], id);
    assert_eq!(body["nombre"], nombre);

    // Edición parcial: `descripcion: ""` limpia y no toca el nombre.
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/categorias/{id}"),
            &cookie,
            Some(json!({ "descripcion": "" })),
        ))
        .await
        .expect("edición de descripción");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert!(body["descripcion"].is_null());
    assert_eq!(body["nombre"], nombre);

    // Editar solo el nombre conserva la descripción nula.
    let nuevo_nombre = nombre_categoria_unico("Almacén editado");
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/categorias/{id}"),
            &cookie,
            Some(json!({ "nombre": nuevo_nombre })),
        ))
        .await
        .expect("edición de nombre");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["nombre"], nuevo_nombre);
    assert!(body["descripcion"].is_null());

    // Duplicar el nombre de otra categoría al editar es 409.
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/categorias/{id}"),
            &cookie,
            Some(json!({ "nombre": otra.to_uppercase() })),
        ))
        .await
        .expect("edición duplicada");
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "categoria_duplicada");

    // Cuerpo vacío y `activo` en el PUT son 422.
    for cuerpo in [json!({}), json!({ "activo": false })] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "PUT",
                &format!("/api/categorias/{id}"),
                &cookie,
                Some(cuerpo.clone()),
            ))
            .await
            .expect("PUT inválido");
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{cuerpo}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "validacion");
    }

    // La fila quedó intacta tras los 422.
    let (nombre_db, activo_db): (String, bool) =
        sqlx::query_as("SELECT nombre, activo FROM categorias WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("fila de categoría");
    assert_eq!(nombre_db, nuevo_nombre);
    assert!(activo_db);

    // 404 y path/query inválidos.
    for (metodo, uri) in [
        ("GET", "/api/categorias/999999999".to_string()),
        ("PUT", "/api/categorias/999999999".to_string()),
    ] {
        let cuerpo = (metodo == "PUT").then(|| json!({ "nombre": "x" }));
        let res = app
            .clone()
            .oneshot(solicitud(metodo, &uri, &cookie, cuerpo))
            .await
            .expect("recurso inexistente");
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{metodo} {uri}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "recurso_no_encontrado");
    }

    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/categorias/abc", &cookie, None))
        .await
        .expect("path inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn listado_por_estado_y_orden_por_nombre() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;

    let alfa = format!("ALFA-{}", uuid::Uuid::new_v4().simple());
    let zeta = format!("ZETA-{}", uuid::Uuid::new_v4().simple());
    let inactiva = nombre_categoria_unico("INACTIVA");
    for nombre in [&zeta, &alfa] {
        let (status, _) = crear_categoria(&app, &cookie, nombre, None).await;
        assert_eq!(status, StatusCode::CREATED, "{nombre}");
    }
    let (status, creada) = crear_categoria(&app, &cookie, &inactiva, None).await;
    assert_eq!(status, StatusCode::CREATED);
    let id_inactiva = id_de(&creada);

    // Desactivar la tercera.
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/categorias/{id_inactiva}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Default: solo activas, ordenadas por nombre. OPERACION puede leer.
    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/categorias", &operario, None))
        .await
        .expect("listado por defecto");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    let items = body.as_array().expect("lista plana de categorías");
    assert!(items.iter().all(|c| c["activo"] == true));
    assert!(
        !items.iter().any(|c| c["id"].as_i64() == Some(id_inactiva as i64)),
        "la inactiva no debe aparecer por defecto"
    );
    let posicion = |nombre: &str| items.iter().position(|c| c["nombre"] == nombre);
    let pos_alfa = posicion(&alfa).expect("alfa activa");
    let pos_zeta = posicion(&zeta).expect("zeta activa");
    assert!(pos_alfa < pos_zeta, "orden ascendente por nombre");

    // `?activo=all` incluye la inactiva con su estado.
    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/categorias?activo=all", &cookie, None))
        .await
        .expect("listado completo");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    let items = body.as_array().expect("lista plana de categorías");
    let inactiva_item = items
        .iter()
        .find(|c| c["id"].as_i64() == Some(id_inactiva as i64))
        .expect("la inactiva aparece con activo=all");
    assert_eq!(inactiva_item["activo"], false);

    // `?activo=false` solo inactivas.
    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/categorias?activo=false", &cookie, None))
        .await
        .expect("listado inactivas");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    let items = body.as_array().expect("lista plana de categorías");
    assert!(items.iter().all(|c| c["activo"] == false));
    assert!(items
        .iter()
        .any(|c| c["id"].as_i64() == Some(id_inactiva as i64)));

    // Filtro inválido.
    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/categorias?activo=todos", &cookie, None))
        .await
        .expect("filtro inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "validacion");
}

#[tokio::test]
async fn baja_logica_idempotente_y_sin_delete() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;
    let nombre = nombre_categoria_unico("Baja lógica");

    let (status, body) = crear_categoria(&app, &cookie, &nombre, Some("descartable")).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = id_de(&body);

    // Desactivar dos veces: idempotente.
    for _ in 0..2 {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/categorias/{id}/desactivar"),
                &cookie,
                None,
            ))
            .await
            .expect("desactivar");
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        assert!(cuerpo_json(res).await.is_null());
    }

    // El detalle sigue respondiendo 200 y muestrea el estado.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/categorias/{id}"),
            &cookie,
            None,
        ))
        .await
        .expect("detalle inactiva");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["activo"], false);
    assert_eq!(body["descripcion"], "descartable");

    // Reactivar dos veces: idempotente.
    for _ in 0..2 {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/categorias/{id}/reactivar"),
                &cookie,
                None,
            ))
            .await
            .expect("reactivar");
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }
    let (activo,): (bool,) = sqlx::query_as("SELECT activo FROM categorias WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("estado de la categoría");
    assert!(activo);

    // DELETE no está registrado: 405 con Allow y sin cuerpo; la fila no cambia.
    let res = app
        .clone()
        .oneshot(solicitud(
            "DELETE",
            &format!("/api/categorias/{id}"),
            &cookie,
            None,
        ))
        .await
        .expect("DELETE");
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    let allow = res
        .headers()
        .get(header::ALLOW)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .replace(' ', "");
    assert!(allow.contains("GET"), "Allow: {allow}");
    assert!(allow.contains("PUT"), "Allow: {allow}");
    assert!(!allow.contains("DELETE"), "Allow: {allow}");
    assert!(cuerpo_json(res).await.is_null());

    let (nombre_db, activo_db): (String, bool) =
        sqlx::query_as("SELECT nombre, activo FROM categorias WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("fila tras DELETE");
    assert_eq!(nombre_db, nombre);
    assert!(activo_db);

    // 404 y 403 en las acciones de estado.
    for accion in ["desactivar", "reactivar"] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/categorias/999999999/{accion}"),
                &cookie,
                None,
            ))
            .await
            .expect("estado inexistente");
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{accion}");

        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/categorias/{id}/{accion}"),
                &operario,
                None,
            ))
            .await
            .expect("estado sin permisos");
        assert_eq!(res.status(), StatusCode::FORBIDDEN, "{accion}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "sin_permisos");
    }
}

#[tokio::test]
async fn permisos_y_limites_de_validacion() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;

    // Sin sesión: 401.
    let res = app
        .clone()
        .oneshot(solicitud_anonima("GET", "/api/categorias"))
        .await
        .expect("sin sesión");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = cuerpo_json(res).await;
    assert_eq!(body["error"]["codigo"], "no_autenticado");

    let res = app
        .clone()
        .oneshot(solicitud_anonima("GET", "/api/categorias/1"))
        .await
        .expect("sin sesión detalle");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // OPERACION puede leer pero no escribir.
    let (status, _) = crear_categoria(&app, &operario, &nombre_categoria_unico("Operario"), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, body) = crear_categoria(&app, &cookie, &nombre_categoria_unico("Base"), None).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = id_de(&body);

    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/categorias/{id}"),
            &operario,
            Some(json!({ "nombre": nombre_categoria_unico("Ops") })),
        ))
        .await
        .expect("PUT sin permisos");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // Límites de validación → 422 validacion.
    for cuerpo in [
        json!({ "nombre": "" }),
        json!({ "nombre": "   " }),
        json!({ "nombre": "a".repeat(81) }),
        json!({ "nombre": "ok", "descripcion": "d".repeat(256) }),
        json!({ "descripcion": "sin nombre" }),
    ] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                "/api/categorias",
                &cookie,
                Some(cuerpo.clone()),
            ))
            .await
            .expect("alta inválida");
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{cuerpo}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "validacion");
    }
}
