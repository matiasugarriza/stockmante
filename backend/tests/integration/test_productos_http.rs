//! 003/T017 — integración HTTP de /api/productos contra el router real
//! (contracts/api.md, CU-PROD-01…06).

#[path = "../unit/common/mod.rs"]
mod common;

use axum::http::{header, StatusCode};
use common::{
    app_de_test, codigo_barras_unico, cuerpo_json, nombre_categoria_unico, sesion_gestion,
    sesion_operacion, sku_unico, solicitud, solicitud_anonima,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn crear_categoria(app: &axum::Router, cookie: &str, nombre: &str) -> i32 {
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            "/api/categorias",
            cookie,
            Some(json!({ "nombre": nombre })),
        ))
        .await
        .expect("alta de categoría");
    assert_eq!(res.status(), StatusCode::CREATED, "categoría {nombre}");
    cuerpo_json(res).await["id"]
        .as_i64()
        .expect("id de categoría") as i32
}

fn cuerpo_producto(nombre: &str, sku: &str, categoria_id: i32) -> Value {
    json!({
        "nombre": nombre,
        "sku": sku,
        "categoria_id": categoria_id,
        "unidad_medida": "UNIDAD",
        "costo": 1200.50,
        "precio_venta": 1800,
        "stock_minimo": 6,
    })
}

async fn crear_producto(app: &axum::Router, cookie: &str, cuerpo: Value) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(solicitud("POST", "/api/productos", cookie, Some(cuerpo)))
        .await
        .expect("alta de producto");
    let status = res.status();
    (status, cuerpo_json(res).await)
}

fn id_de(body: &Value) -> i32 {
    body["id"].as_i64().expect("id del producto") as i32
}

#[tokio::test]
async fn alta_normaliza_y_valida_unicidad_y_limites() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;
    let categoria = crear_categoria(&app, &cookie, &nombre_categoria_unico("Almacén")).await;

    // Normalización de SKU, `codigo_barras` ausente y montos numéricos.
    let sku = sku_unico("yerba");
    let (status, body) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Yerba 1 kg", &format!("  {}  ", sku.to_lowercase()), categoria),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta válida: {body}");
    assert_eq!(body["sku"], sku);
    assert_eq!(body["nombre"], "Yerba 1 kg");
    assert!(body["codigo_barras"].is_null());
    assert_eq!(body["categoria"]["id"], categoria);
    assert_eq!(body["unidad_medida"], "UNIDAD");
    assert!(body["costo"].is_number(), "costo debe ser número JSON");
    assert_eq!(body["costo"].as_f64(), Some(1200.5));
    assert_eq!(body["precio_venta"].as_f64(), Some(1800.0));
    assert_eq!(body["stock_minimo"], 6);
    assert_eq!(body["activo"], true);
    let primero = id_de(&body);

    // SKU repetido (mismo valor normalizado) → 409.
    let (status, body) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Otra yerba", &sku.to_lowercase(), categoria),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["codigo"], "sku_duplicado");

    // El SKU sigue siendo único aunque el producto esté dado de baja.
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/productos/{primero}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let (status, body) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Otra yerba", &sku, categoria),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["codigo"], "sku_duplicado");

    // Varios productos sin código de barras conviven (NULLS DISTINCT).
    let (status, _) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Sin código 1", &sku_unico("sin"), categoria),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Código de barras único.
    let codigo = codigo_barras_unico();
    let mut con_codigo = cuerpo_producto("Con código", &sku_unico("cod"), categoria);
    con_codigo["codigo_barras"] = json!(codigo);
    let (status, body) = crear_producto(&app, &cookie, con_codigo.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["codigo_barras"], codigo);
    con_codigo["sku"] = json!(sku_unico("cod2"));
    let (status, body) = crear_producto(&app, &cookie, con_codigo).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["codigo"], "codigo_barras_duplicado");

    // Categoría inexistente o inactiva → 422.
    let (status, body) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Huérfano", &sku_unico("huerfano"), 999_999_999),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["codigo"], "validacion");

    let inactiva_nombre = nombre_categoria_unico("Inactiva");
    let inactiva = crear_categoria(&app, &cookie, &inactiva_nombre).await;
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/categorias/{inactiva}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar categoría");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let (status, body) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Sin categoría activa", &sku_unico("inact"), inactiva),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["codigo"], "validacion");

    // Montos, unidad, nombre y stock.
    let mut casos: Vec<(Value, &str)> = Vec::new();
    let mut negativo = cuerpo_producto("Monto", &sku_unico("m1"), categoria);
    negativo["costo"] = json!(-1);
    casos.push((negativo, "costo negativo"));
    let mut decimales = cuerpo_producto("Monto", &sku_unico("m2"), categoria);
    decimales["costo"] = json!(10.999);
    casos.push((decimales, "tres decimales"));
    let mut tope = cuerpo_producto("Monto", &sku_unico("m3"), categoria);
    tope["precio_venta"] = json!(10_000_000_000i64);
    casos.push((tope, "tope de monto"));
    let mut unidad = cuerpo_producto("Unidad", &sku_unico("u1"), categoria);
    unidad["unidad_medida"] = json!("LITRO");
    casos.push((unidad, "unidad fuera de lista"));
    let mut sin_nombre = cuerpo_producto("   ", &sku_unico("n1"), categoria);
    sin_nombre["nombre"] = json!("   ");
    casos.push((sin_nombre, "nombre vacío"));
    let mut nombre_largo = cuerpo_producto("Nombre", &sku_unico("n2"), categoria);
    nombre_largo["nombre"] = json!("a".repeat(161));
    casos.push((nombre_largo, "nombre de 161"));
    let mut stock = cuerpo_producto("Stock", &sku_unico("s1"), categoria);
    stock["stock_minimo"] = json!(-3);
    casos.push((stock, "stock negativo"));
    let mut stock_decimal = cuerpo_producto("Stock", &sku_unico("s2"), categoria);
    stock_decimal["stock_minimo"] = json!(2.5);
    casos.push((stock_decimal, "stock no entero"));

    for (cuerpo, etiqueta) in casos {
        let (status, body) = crear_producto(&app, &cookie, cuerpo).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{etiqueta}");
        assert_eq!(body["error"]["codigo"], "validacion", "{etiqueta}");
    }

    // Escritura de OPERACION → 403; sin sesión → 401.
    let (status, body) = crear_producto(
        &app,
        &operario,
        cuerpo_producto("Sin permiso", &sku_unico("p1"), categoria),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["codigo"], "sin_permisos");

    let res = app
        .clone()
        .oneshot(solicitud_anonima("GET", "/api/productos"))
        .await
        .expect("sin sesión");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn listado_filtra_pagina_y_ordena() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;
    let categoria_a = crear_categoria(&app, &cookie, &nombre_categoria_unico("Filtro A")).await;
    let categoria_b = crear_categoria(&app, &cookie, &nombre_categoria_unico("Filtro B")).await;

    let sufijo = uuid::Uuid::new_v4().simple().to_string();
    let sku_aaa = sku_unico("aaa");
    let nombre_aaa = format!("AAA {sufijo}");
    let nombre_zzz = format!("ZZZ {sufijo}");
    let nombre_mmm = format!("MMM {sufijo}");
    let mut ids = Vec::new();
    for (nombre, sku, categoria) in [
        (nombre_zzz.as_str(), sku_unico("zzz"), categoria_b),
        (nombre_aaa.as_str(), sku_aaa.clone(), categoria_a),
        (nombre_mmm.as_str(), sku_unico("mmm"), categoria_a),
    ] {
        let (status, body) =
            crear_producto(&app, &cookie, cuerpo_producto(nombre, &sku, categoria)).await;
        assert_eq!(status, StatusCode::CREATED, "{nombre}");
        ids.push(id_de(&body));
    }
    let id_mmm = ids[2];
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/productos/{id_mmm}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Default: solo activos, con total, orden por nombre.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?por_pagina=100&q={sufijo}"),
            &cookie,
            None,
        ))
        .await
        .expect("listado por defecto");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["pagina"], 1);
    assert_eq!(body["por_pagina"], 100);
    assert_eq!(body["total"], 2, "el inactivo no cuenta por defecto: {body}");
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|p| p["activo"] == true));
    assert_eq!(items[0]["nombre"], nombre_aaa);
    assert_eq!(items[1]["nombre"], nombre_zzz);

    // `q` busca en nombre y en SKU, sin distinguir mayúsculas.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?q={}", sku_aaa.to_lowercase()),
            &cookie,
            None,
        ))
        .await
        .expect("filtro por sku");
    let body = cuerpo_json(res).await;
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["sku"], sku_aaa);

    // `categoria_id` combina por AND.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?categoria_id={categoria_a}&q={sufijo}"),
            &cookie,
            None,
        ))
        .await
        .expect("filtro por categoría");
    let body = cuerpo_json(res).await;
    assert_eq!(body["total"], 1, "{body}");
    assert_eq!(body["items"][0]["nombre"], nombre_aaa);

    // Categoría inexistente: lista vacía, no 404.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?categoria_id=999999999&q={sufijo}"),
            &cookie,
            None,
        ))
        .await
        .expect("categoría inexistente");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["total"], 0);
    assert_eq!(body["items"].as_array().map(Vec::len), Some(0));

    // `?activo=all` incluye el inactivo con su id original.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?activo=all&q={sufijo}&por_pagina=100"),
            &cookie,
            None,
        ))
        .await
        .expect("listado completo");
    let body = cuerpo_json(res).await;
    assert_eq!(body["total"], 3);
    let items = body["items"].as_array().expect("items");
    assert!(items
        .iter()
        .any(|p| p["id"].as_i64() == Some(id_mmm as i64) && p["activo"] == false));

    // `?activo=false` solo inactivos.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?activo=false&q={sufijo}&por_pagina=100"),
            &cookie,
            None,
        ))
        .await
        .expect("solo inactivos");
    let body = cuerpo_json(res).await;
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["id"].as_i64(), Some(id_mmm as i64));

    // Paginación: una fila por página, sin perder el total.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?q={sufijo}&por_pagina=1&pagina=1"),
            &cookie,
            None,
        ))
        .await
        .expect("página 1");
    let body = cuerpo_json(res).await;
    assert_eq!(body["total"], 2);
    assert_eq!(body["items"].as_array().map(Vec::len), Some(1));
    let primero = body["items"][0]["id"].as_i64();
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos?q={sufijo}&por_pagina=1&pagina=2"),
            &cookie,
            None,
        ))
        .await
        .expect("página 2");
    let body = cuerpo_json(res).await;
    assert_eq!(body["items"].as_array().map(Vec::len), Some(1));
    assert_ne!(body["items"][0]["id"].as_i64(), primero);

    // Rangos inválidos → 422.
    for query in [
        "pagina=0",
        "por_pagina=0",
        "por_pagina=101",
        "activo=todos",
        "categoria_id=0",
        &format!("q={}", "x".repeat(161)),
    ] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "GET",
                &format!("/api/productos?{query}"),
                &cookie,
                None,
            ))
            .await
            .expect("query inválida");
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{query}");
        let body = cuerpo_json(res).await;
        assert_eq!(body["error"]["codigo"], "validacion", "{query}");
    }

    // El listado de gestión es solo GESTION.
    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/productos", &operario, None))
        .await
        .expect("listado sin permisos");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn detalle_y_edicion_parcial_atomica() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let operario = sesion_operacion(&pool).await;
    let categoria = crear_categoria(&app, &cookie, &nombre_categoria_unico("Edición")).await;

    let codigo = codigo_barras_unico();
    let mut cuerpo = cuerpo_producto("Producto editable", &sku_unico("edit"), categoria);
    cuerpo["codigo_barras"] = json!(codigo);
    let (status, body) = crear_producto(&app, &cookie, cuerpo).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = id_de(&body);
    let sku_original = body["sku"].as_str().expect("sku").to_string();

    // Detalle: sesión OPERACION puede leer.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos/{id}"),
            &operario,
            None,
        ))
        .await
        .expect("detalle");
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(cuerpo_json(res).await["codigo_barras"], codigo);

    // Editar solo el precio conserva el resto.
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "precio_venta": 2000.75 })),
        ))
        .await
        .expect("editar precio");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["precio_venta"].as_f64(), Some(2000.75));
    assert_eq!(body["sku"], sku_original);
    assert_eq!(body["codigo_barras"], codigo);
    assert_eq!(body["nombre"], "Producto editable");

    // `codigo_barras: ""` limpia; omitirlo lo conserva.
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "codigo_barras": "" })),
        ))
        .await
        .expect("limpiar código");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert!(body["codigo_barras"].is_null());

    let otro_codigo = codigo_barras_unico();
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "codigo_barras": otro_codigo })),
        ))
        .await
        .expect("asignar código");
    assert_eq!(res.status(), StatusCode::OK);
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "nombre": "Producto renombrado" })),
        ))
        .await
        .expect("editar nombre");
    let body = cuerpo_json(res).await;
    assert_eq!(body["codigo_barras"], otro_codigo);

    // Cuerpo vacío y `activo` son 422; la fila no cambia (atomicidad).
    for cuerpo in [json!({}), json!({ "activo": false })] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "PUT",
                &format!("/api/productos/{id}"),
                &cookie,
                Some(cuerpo.clone()),
            ))
            .await
            .expect("PUT inválido");
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{cuerpo}");
    }
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "precio_venta": 3000, "costo": -5 })),
        ))
        .await
        .expect("PUT parcialmente inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let (precio, codigo_db): (rust_decimal::Decimal, Option<String>) =
        sqlx::query_as("SELECT precio_venta, codigo_barras FROM productos WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("fila del producto");
    assert_eq!(precio.to_string(), "2000.75");
    assert_eq!(codigo_db.as_deref(), Some(otro_codigo.as_str()));

    // SKU duplicado al editar → 409.
    let (status, otro) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Otro producto", &sku_unico("otro"), categoria),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let sku_otro = otro["sku"].as_str().expect("sku").to_string();
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "sku": sku_otro })),
        ))
        .await
        .expect("sku duplicado");
    assert_eq!(res.status(), StatusCode::CONFLICT);
    assert_eq!(cuerpo_json(res).await["error"]["codigo"], "sku_duplicado");

    // Editar un producto inactivo responde 200.
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/productos/{id}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "stock_minimo": 9 })),
        ))
        .await
        .expect("editar inactivo");
    assert_eq!(res.status(), StatusCode::OK);
    let body = cuerpo_json(res).await;
    assert_eq!(body["stock_minimo"], 9);
    assert_eq!(body["activo"], false);

    // Un PUT hacia una categoría inactiva es 422.
    let inactiva = crear_categoria(&app, &cookie, &nombre_categoria_unico("Cat inactiva")).await;
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/categorias/{inactiva}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar categoría");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &cookie,
            Some(json!({ "categoria_id": inactiva })),
        ))
        .await
        .expect("categoría inactiva");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 404, path inválido y 403 en escritura.
    for (metodo, uri, cuerpo) in [
        ("GET", "/api/productos/999999999".to_string(), None),
        (
            "PUT",
            "/api/productos/999999999".to_string(),
            Some(json!({ "nombre": "x" })),
        ),
    ] {
        let res = app
            .clone()
            .oneshot(solicitud(metodo, &uri, &cookie, cuerpo))
            .await
            .expect("inexistente");
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{metodo} {uri}");
    }
    let res = app
        .clone()
        .oneshot(solicitud("GET", "/api/productos/abc", &cookie, None))
        .await
        .expect("path inválido");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let res = app
        .clone()
        .oneshot(solicitud(
            "PUT",
            &format!("/api/productos/{id}"),
            &operario,
            Some(json!({ "nombre": "x" })),
        ))
        .await
        .expect("PUT sin permisos");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn baja_y_reactivacion_idempotentes_sin_cascada() {
    let (app, pool, _correo) = app_de_test().await;
    let cookie = sesion_gestion(&pool).await;
    let categoria = crear_categoria(&app, &cookie, &nombre_categoria_unico("Baja")).await;

    let (status, body) = crear_producto(
        &app,
        &cookie,
        cuerpo_producto("Producto con baja", &sku_unico("baja"), categoria),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = id_de(&body);

    for _ in 0..2 {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/productos/{id}/desactivar"),
                &cookie,
                None,
            ))
            .await
            .expect("desactivar");
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        assert!(cuerpo_json(res).await.is_null());
    }

    // Desactivar el producto no toca la categoría (sin cascada).
    let (activo_categoria,): (bool,) = sqlx::query_as("SELECT activo FROM categorias WHERE id = $1")
        .bind(categoria)
        .fetch_one(&pool)
        .await
        .expect("categoría");
    assert!(activo_categoria);

    // El detalle sigue disponible con `activo: false`.
    let res = app
        .clone()
        .oneshot(solicitud(
            "GET",
            &format!("/api/productos/{id}"),
            &cookie,
            None,
        ))
        .await
        .expect("detalle inactivo");
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(cuerpo_json(res).await["activo"], false);

    // Reactivar es idempotente, incluso con la categoría ya inactiva.
    let res = app
        .clone()
        .oneshot(solicitud(
            "POST",
            &format!("/api/categorias/{categoria}/desactivar"),
            &cookie,
            None,
        ))
        .await
        .expect("desactivar categoría");
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    for _ in 0..2 {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/productos/{id}/reactivar"),
                &cookie,
                None,
            ))
            .await
            .expect("reactivar");
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }
    let (activo,): (bool,) = sqlx::query_as("SELECT activo FROM productos WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("producto");
    assert!(activo);

    // 404 en acciones de estado inexistentes.
    for accion in ["desactivar", "reactivar"] {
        let res = app
            .clone()
            .oneshot(solicitud(
                "POST",
                &format!("/api/productos/999999999/{accion}"),
                &cookie,
                None,
            ))
            .await
            .expect("estado inexistente");
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{accion}");
    }

    // DELETE no está registrado: 405 con Allow y la fila intacta.
    let res = app
        .clone()
        .oneshot(solicitud(
            "DELETE",
            &format!("/api/productos/{id}"),
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

    let (nombre, sku, activo_db): (String, String, bool) =
        sqlx::query_as("SELECT nombre, sku, activo FROM productos WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("fila tras DELETE");
    assert_eq!(nombre, "Producto con baja");
    assert_eq!(sku, body["sku"].as_str().expect("sku"));
    assert!(activo_db);
}
