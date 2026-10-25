//! T028 — mapa rol → familia (data-model.md §usuarios + migrations/0001).

use axum::http::StatusCode;
use stockmante_backend::error::AppError;
use stockmante_backend::services::usuarios::familia_de_rol;

#[test]
fn roles_de_gestion_mapean_a_gestion() {
    assert_eq!(familia_de_rol("ADMIN").expect("ADMIN es válido"), "GESTION");
    assert_eq!(
        familia_de_rol("GERENTE").expect("GERENTE es válido"),
        "GESTION"
    );
}

#[test]
fn roles_de_operacion_mapean_a_operacion() {
    assert_eq!(
        familia_de_rol("OPERARIO").expect("OPERARIO es válido"),
        "OPERACION"
    );
    assert_eq!(
        familia_de_rol("CAJERO").expect("CAJERO es válido"),
        "OPERACION"
    );
}

#[test]
fn rol_fuera_del_catalogo_es_validacion() {
    let err = familia_de_rol("SUPERVISOR").expect_err("rol fuera del catálogo");
    assert!(
        matches!(err, AppError::Validacion(_)),
        "esperaba Validacion, vino {err:?}"
    );
    assert_eq!(err.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(err.codigo(), "validacion");
}
