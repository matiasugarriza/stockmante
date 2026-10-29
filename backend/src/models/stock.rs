//! Structs de respuesta de movimientos y stock (007).
//!
//! Las filas planas (`sqlx::FromRow`) alimentan structs anidados que sí son el contrato
//! público de la API: SQLx no mapea structs anidados directamente.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

/// Referencia mínima al depósito dentro de un movimiento.

/// Referencia mínima al depósito dentro de un movimiento.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DepositoBreve {
    pub id: i32,
    pub nombre: String,
}

/// Referencia mínima al autor dentro de un movimiento.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UsuarioBreve {
    pub id: i32,
    pub nombre: String,
}

/// Línea de un movimiento con el saldo que dejó en el depósito.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LineaMovimiento {
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    pub ubicacion_id: Option<i32>,
    pub ubicacion_nombre: Option<String>,
    pub cantidad_delta: i32,
    pub stock_resultante: i32,
}

/// Movimiento completo con sus líneas.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MovimientoDetalle {
    pub id: i32,
    pub tipo: String,
    pub deposito: DepositoBreve,
    pub motivo: String,
    pub observacion: Option<String>,
    pub usuario: UsuarioBreve,
    pub creado_en: DateTime<Utc>,
    pub lineas: Vec<LineaMovimiento>,
}

/// Página del historial de movimientos.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaMovimientos {
    pub items: Vec<MovimientoDetalle>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

/// Detalle de una transferencia entre depósitos (008).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TransferenciaDetalle {
    pub id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    pub deposito_origen: DepositoBreve,
    pub deposito_destino: DepositoBreve,
    pub cantidad: i32,
    pub stock_origen_resultante: i32,
    pub stock_destino_resultante: i32,
    pub usuario: UsuarioBreve,
    pub creado_en: DateTime<Utc>,
    pub movimiento_salida_id: i32,
    pub movimiento_entrada_id: i32,
    pub observacion: Option<String>,
}

/// Página del historial de transferencias.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaTransferencias {
    pub items: Vec<TransferenciaDetalle>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

/// Fila plana del SELECT de la cabecera de la transferencia.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TransferenciaFila {
    pub id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    pub deposito_origen_id: i32,
    pub deposito_origen_nombre: String,
    pub deposito_destino_id: i32,
    pub deposito_destino_nombre: String,
    pub cantidad: i32,
    pub stock_origen_resultante: i32,
    pub stock_destino_resultante: i32,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub creado_en: DateTime<Utc>,
    pub movimiento_salida_id: i32,
    pub movimiento_entrada_id: i32,
    pub observacion: Option<String>,
}

/// Fila plana del `SELECT` de la cabecera del movimiento.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MovimientoCabecera {
    pub id: i32,
    pub tipo: String,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub motivo: String,
    pub observacion: Option<String>,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub creado_en: DateTime<Utc>,
}

/// Fila plana de una línea con producto y ubicación ya unidos.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LineaFila {
    pub movimiento_id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    pub ubicacion_id: Option<i32>,
    pub ubicacion_nombre: Option<String>,
    pub cantidad_delta: i32,
    pub stock_resultante: i32,
}

impl From<LineaFila> for LineaMovimiento {
    fn from(fila: LineaFila) -> Self {
        LineaMovimiento {
            producto_id: fila.producto_id,
            producto_nombre: fila.producto_nombre,
            sku: fila.sku,
            ubicacion_id: fila.ubicacion_id,
            ubicacion_nombre: fila.ubicacion_nombre,
            cantidad_delta: fila.cantidad_delta,
            stock_resultante: fila.stock_resultante,
        }
    }
}

/// Detalle de un conteo de inventario (009).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConteoLinea {
    pub id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    pub ubicacion_id: Option<i32>,
    pub ubicacion_nombre: Option<String>,
    pub cantidad_sistema: i32,
    pub cantidad_contada: Option<i32>,
    pub diferencia: Option<i32>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConteoDetalle {
    pub id: i32,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub estado: String,
    pub usuario: UsuarioBreve,
    pub creado_en: DateTime<Utc>,
    pub cerrado_en: Option<DateTime<Utc>>,
    pub cancelado_en: Option<DateTime<Utc>>,
    pub movimiento_ajuste_id: Option<i32>,
    pub observacion: Option<String>,
    pub lineas: Vec<ConteoLinea>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaConteos {
    pub items: Vec<ConteoDetalle>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

/// Fila plana del SELECT de la cabecera del conteo.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConteoFila {
    pub id: i32,
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub estado: String,
    pub usuario_id: i32,
    pub usuario_nombre: String,
    pub creado_en: DateTime<Utc>,
    pub cerrado_en: Option<DateTime<Utc>>,
    pub cancelado_en: Option<DateTime<Utc>>,
    pub movimiento_ajuste_id: Option<i32>,
    pub observacion: Option<String>,
}

/// Fila plana de una línea de conteo.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConteoLineaFila {
    pub id: i32,
    pub conteo_id: i32,
    pub producto_id: i32,
    pub producto_nombre: String,
    pub sku: String,
    pub ubicacion_id: Option<i32>,
    pub ubicacion_nombre: Option<String>,
    pub cantidad_sistema: i32,
    pub cantidad_contada: Option<i32>,
    pub diferencia: Option<i32>,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StockFila {
    pub producto_id: i32,
    pub nombre: String,
    pub sku: String,
    pub categoria_id: i32,
    pub categoria_nombre: String,
    pub unidad_medida: String,
    pub activo: bool,
    pub stock_actual: i64,
    pub stock_minimo: i32,
    pub stock_deposito: Option<i64>,
}

/// Item del listado de stock.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StockItem {
    pub producto_id: i32,
    pub nombre: String,
    pub sku: String,
    pub categoria_id: i32,
    pub categoria_nombre: String,
    pub unidad_medida: String,
    pub activo: bool,
    pub stock_actual: i64,
    pub stock_minimo: i32,
    pub bajo_minimo: bool,
    /// Solo presente cuando se filtra por depósito.
    pub stock_deposito: Option<i64>,
}

impl From<StockFila> for StockItem {
    fn from(fila: StockFila) -> Self {
        let bajo_minimo = fila.stock_minimo > 0 && fila.stock_actual < i64::from(fila.stock_minimo);
        StockItem {
            producto_id: fila.producto_id,
            nombre: fila.nombre,
            sku: fila.sku,
            categoria_id: fila.categoria_id,
            categoria_nombre: fila.categoria_nombre,
            unidad_medida: fila.unidad_medida,
            activo: fila.activo,
            stock_actual: fila.stock_actual,
            stock_minimo: fila.stock_minimo,
            bajo_minimo,
            stock_deposito: fila.stock_deposito,
        }
    }
}

/// Página del listado de stock.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListaStock {
    pub items: Vec<StockItem>,
    pub total: i64,
    pub pagina: i64,
    pub por_pagina: i64,
}

/// Una fila del desglose por depósito y ubicación.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct DesgloseStock {
    pub deposito_id: i32,
    pub deposito_nombre: String,
    pub ubicacion_id: Option<i32>,
    pub ubicacion_nombre: Option<String>,
    pub cantidad: i64,
}

/// Detalle de stock de un producto.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StockProducto {
    pub producto_id: i32,
    pub nombre: String,
    pub sku: String,
    pub unidad_medida: String,
    pub activo: bool,
    pub stock_actual: i64,
    pub stock_minimo: i32,
    pub bajo_minimo: bool,
    pub desglose: Vec<DesgloseStock>,
}
