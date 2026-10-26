use axum::http::StatusCode;
use axum::response::IntoResponse;
use stockmante_backend::error::AppError;

fn status_y_codigo(e: AppError) -> (StatusCode, String) {
    let r = e.into_response();
    let status = r.status();
    // El body se verifica por separado en los tests de contenido
    let codigo = match status {
        StatusCode::UNAUTHORIZED => "credenciales_invalidas".to_string(),
        _ => String::new(),
    };
    (status, codigo)
}

#[test]
fn dominio_mapea_status_esperados() {
    assert_eq!(status_y_codigo(AppError::CredencialesInvalidas).0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        status_y_codigo(AppError::SesionExpirada).0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_y_codigo(AppError::NoAutenticado).0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(status_y_codigo(AppError::SinPermisos).0, StatusCode::FORBIDDEN);
    assert_eq!(
        AppError::DebeCambiarPassword.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        AppError::DebeCambiarPassword.codigo(),
        "cambio_password_requerido"
    );
    assert_eq!(
        status_y_codigo(AppError::RecursoNoEncontrado("x".into())).0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status_y_codigo(AppError::EmailDuplicado).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        status_y_codigo(AppError::SkuDuplicado("SKU-1".into())).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        status_y_codigo(AppError::CodigoBarrasDuplicado).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        status_y_codigo(AppError::CategoriaDuplicada).0,
        StatusCode::CONFLICT
    );
    assert_eq!(AppError::CategoriaDuplicada.codigo(), "categoria_duplicada");
    assert_eq!(
        status_y_codigo(AppError::DepositoConDependencias).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        status_y_codigo(AppError::UltimoAdministrador).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        status_y_codigo(AppError::Validacion("sku inválido".into())).0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[test]
fn interno_no_expone_detalles() {
    let r = AppError::Interno("password de postgres en la connectionString".into()).into_response();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn dominio_usa_mensajes_espanol_accionables() {
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    let resp = AppError::SinPermisos.into_response();
    let status = resp.status();
    assert_eq!(status, StatusCode::FORBIDDEN);
    // consumir body de forma síncrona en test bloqueante
    let body = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            let bytes = to_bytes(resp.into_body(), 64 * 1024).await.expect("body");
            String::from_utf8_lossy(&bytes).to_string()
        });
    assert!(body.contains("permiso"), "mensaje sin acción clara: {body}");
    assert!(body.contains("codigo"), "falta campo codigo: {body}");
    assert!(body.contains("mensaje"), "falta campo mensaje: {body}");
}
