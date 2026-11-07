//! DTOs de alertas de stock mínimo (011).

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

/// Producto activo por debajo de su mínimo (`stock_minimo > 0`).
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AlertaItem {
    pub producto_id: i32,
    pub nombre: String,
    pub sku: String,
    pub stock_actual: i64,
    pub stock_minimo: i32,
    /// `stock_minimo − stock_actual` (siempre > 0 en una alerta).
    pub faltante: i64,
}

/// Página de `GET /api/alertas`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaAlertas {
    pub items: Vec<AlertaItem>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

/// Resultado de un ciclo de alertas (`POST /api/alertas/enviar` y scheduler).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RespuestaCiclo {
    /// Día local (UTC−03:00) usado como clave de deduplicación.
    #[schema(value_type = String, example = "2026-11-07")]
    pub fecha: NaiveDate,
    /// Productos distintos bajo mínimo detectados.
    pub productos: i64,
    /// Usuarios GESTION activos evaluados.
    pub destinatarios: i64,
    /// Filas marcadas `enviada` en esta corrida.
    pub enviados: i64,
    /// Filas marcadas `fallida` en esta corrida.
    pub fallidos: i64,
}
