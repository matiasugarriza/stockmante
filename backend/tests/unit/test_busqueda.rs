//! 004 — parsing del campo de búsqueda de caja.
//!
//! La validación de longitudes y límites se cubre por HTTP en
//! `tests/integration/test_busqueda_http.rs`; acá se aísla lo puro.

use stockmante_backend::services::productos::CampoBusqueda;

#[test]
fn campo_ausente_o_vacio_equivale_a_todos() {
    assert_eq!(CampoBusqueda::parsear(None).expect("ausente"), CampoBusqueda::Todos);
    assert_eq!(CampoBusqueda::parsear(Some("")).expect("vacío"), CampoBusqueda::Todos);
    assert_eq!(
        CampoBusqueda::parsear(Some("  ")).expect("espacios"),
        CampoBusqueda::Todos
    );
    assert_eq!(
        CampoBusqueda::parsear(Some("todos")).expect("todos"),
        CampoBusqueda::Todos
    );
}

#[test]
fn reconoce_los_cuatro_campos_y_todos() {
    for (valor, esperado) in [
        ("sku", CampoBusqueda::Sku),
        ("codigo", CampoBusqueda::Codigo),
        ("nombre", CampoBusqueda::Nombre),
        ("categoria", CampoBusqueda::Categoria),
        ("todos", CampoBusqueda::Todos),
    ] {
        assert_eq!(
            CampoBusqueda::parsear(Some(valor)).expect(valor),
            esperado,
            "campo {valor}"
        );
    }
}

#[test]
fn normaliza_mayusculas_y_espacios_del_campo() {
    assert_eq!(
        CampoBusqueda::parsear(Some("  SKU  ")).expect("mayúsculas"),
        CampoBusqueda::Sku
    );
    assert_eq!(
        CampoBusqueda::parsear(Some("Categoria")).expect("capitalizado"),
        CampoBusqueda::Categoria
    );
}

#[test]
fn campo_desconocido_es_422_con_mensaje_accionable() {
    let error = CampoBusqueda::parsear(Some("precio")).expect_err("campo inválido");
    assert_eq!(error.status(), axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error.codigo(), "validacion");
    assert_eq!(
        error.mensaje_publico(),
        "El campo de búsqueda debe ser sku, codigo, nombre, categoria o todos."
    );
}
