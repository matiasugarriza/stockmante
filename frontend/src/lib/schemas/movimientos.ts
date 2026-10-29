import { z } from 'zod';

/** Tipos de movimiento (FR-024). */
export const TipoMovimientoSchema = z.enum(['ENTRADA', 'SALIDA', 'MERMA', 'AJUSTE']);
export type TipoMovimiento = z.infer<typeof TipoMovimientoSchema>;

/** Motivos admitidos (FR-024). */
export const MotivoMovimientoSchema = z.enum([
	'COMPRA',
	'VENTA',
	'MERMA',
	'AJUSTE',
	'DEVOLUCION',
	'TRANSFERENCIA',
	'INVENTARIO',
	'IMPORTACION',
	'OTRO'
]);
export type MotivoMovimiento = z.infer<typeof MotivoMovimientoSchema>;

const DepositoBreveSchema = z.object({
	id: z.number().int(),
	nombre: z.string()
});

const UsuarioBreveSchema = z.object({
	id: z.number().int(),
	nombre: z.string()
});

/** Línea devuelta por la API, con el saldo resultante en el depósito. */
export const LineaMovimientoSchema = z.object({
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	sku: z.string(),
	ubicacion_id: z.number().int().nullable(),
	ubicacion_nombre: z.string().nullable(),
	cantidad_delta: z.number().int(),
	stock_resultante: z.number().int()
});
export type LineaMovimiento = z.infer<typeof LineaMovimientoSchema>;

/** Movimiento completo (cabecera + líneas). */
export const MovimientoDetalleSchema = z.object({
	id: z.number().int(),
	tipo: TipoMovimientoSchema,
	deposito: DepositoBreveSchema,
	motivo: MotivoMovimientoSchema,
	observacion: z.string().nullable(),
	usuario: UsuarioBreveSchema,
	creado_en: z.string(),
	lineas: z.array(LineaMovimientoSchema)
});
export type MovimientoDetalle = z.infer<typeof MovimientoDetalleSchema>;

export const ListaMovimientosSchema = z.object({
	items: z.array(MovimientoDetalleSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaMovimientos = z.infer<typeof ListaMovimientosSchema>;

/** Línea del alta: el signo de `cantidad_delta` lo impone el tipo. */
export const LineaNuevaSchema = z.object({
	producto_id: z.number().int().positive('Elegí un producto'),
	ubicacion_id: z.number().int().positive().nullish(),
	cantidad_delta: z.number().int().refine((v) => v !== 0, 'La cantidad no puede ser cero')
});
export type LineaNuevaInput = z.infer<typeof LineaNuevaSchema>;

/** Alta de movimiento; las reglas de signo por tipo se validan al enviar. */
export const CrearMovimientoSchema = z.object({
	tipo: TipoMovimientoSchema,
	deposito_id: z.number().int().positive('Elegí un depósito'),
	motivo: MotivoMovimientoSchema,
	observacion: z
		.string()
		.trim()
		.max(500, 'Máximo 500 caracteres')
		.nullish()
		.transform((v) => (v ? v : null)),
	lineas: z.array(LineaNuevaSchema).min(1, 'Agregá al menos una línea').max(200)
});
export type CrearMovimientoInput = z.infer<typeof CrearMovimientoSchema>;

/** Item del listado de stock (FR-029). */
export const StockItemSchema = z.object({
	producto_id: z.number().int(),
	nombre: z.string(),
	sku: z.string(),
	categoria_id: z.number().int(),
	categoria_nombre: z.string(),
	unidad_medida: z.string(),
	activo: z.boolean(),
	stock_actual: z.number().int(),
	stock_minimo: z.number().int(),
	bajo_minimo: z.boolean(),
	stock_deposito: z.number().int().nullable()
});
export type StockItem = z.infer<typeof StockItemSchema>;

export const ListaStockSchema = z.object({
	items: z.array(StockItemSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaStock = z.infer<typeof ListaStockSchema>;

export const DesgloseStockSchema = z.object({
	deposito_id: z.number().int(),
	deposito_nombre: z.string(),
	ubicacion_id: z.number().int().nullable(),
	ubicacion_nombre: z.string().nullable(),
	cantidad: z.number().int()
});
export type DesgloseStock = z.infer<typeof DesgloseStockSchema>;

export const StockProductoSchema = z.object({
	producto_id: z.number().int(),
	nombre: z.string(),
	sku: z.string(),
	unidad_medida: z.string(),
	activo: z.boolean(),
	stock_actual: z.number().int(),
	stock_minimo: z.number().int(),
	bajo_minimo: z.boolean(),
	desglose: z.array(DesgloseStockSchema)
});
export type StockProducto = z.infer<typeof StockProductoSchema>;
