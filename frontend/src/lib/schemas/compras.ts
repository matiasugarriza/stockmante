import { z } from 'zod';

export const EstadoCompraSchema = z.enum(['BORRADOR', 'CONFIRMADA', 'ANULADA']);
export type EstadoCompra = z.infer<typeof EstadoCompraSchema>;

/** Línea de compra tal como la devuelve el detalle (012). */
export const CompraLineaSchema = z.object({
	id: z.number().int(),
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	sku: z.string().nullable(),
	cantidad: z.number().int(),
	costo_unitario: z.number(),
	subtotal: z.number()
});
export type CompraLinea = z.infer<typeof CompraLineaSchema>;

/** Cabecera de compra (012). `total_costo` llega como número por serde-float. */
export const CompraSchema = z.object({
	id: z.number().int(),
	proveedor_id: z.number().int(),
	proveedor_nombre: z.string(),
	deposito_id: z.number().int(),
	deposito_nombre: z.string(),
	numero: z.string().nullable(),
	estado: EstadoCompraSchema,
	observacion: z.string().nullable(),
	total_costo: z.number(),
	usuario_id: z.number().int(),
	usuario_nombre: z.string(),
	creado_en: z.string(),
	confirmado_en: z.string().nullable(),
	anulado_en: z.string().nullable()
});
export type Compra = z.infer<typeof CompraSchema>;

export const CompraDetalleSchema = z.object({
	compra: CompraSchema,
	lineas: z.array(CompraLineaSchema)
});
export type CompraDetalle = z.infer<typeof CompraDetalleSchema>;

export const ListaComprasSchema = z.object({
	items: z.array(CompraSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaCompras = z.infer<typeof ListaComprasSchema>;

const MontoSchema = z
	.number()
	.nonnegative('El costo no puede ser negativo.')
	.lt(10_000_000_000, 'El costo es demasiado alto.');

export const CompraLineaInputSchema = z.object({
	producto_id: z.number().int().positive('Elegí un producto.'),
	cantidad: z
		.number()
		.int('La cantidad debe ser un entero.')
		.positive('La cantidad debe ser mayor a cero.'),
	costo_unitario: MontoSchema
});
export type CompraLineaInput = z.input<typeof CompraLineaInputSchema>;

/** Alta/edición de compra en borrador: una o más líneas. */
export const CrearCompraSchema = z.object({
	proveedor_id: z.number().int().positive('Elegí un proveedor.'),
	deposito_id: z.number().int().positive('Elegí un depósito.'),
	numero: z.string().trim().max(40, 'Máximo 40 caracteres.').optional().default(''),
	observacion: z.string().trim().max(500, 'Máximo 500 caracteres.').optional().default(''),
	lineas: z.array(CompraLineaInputSchema).min(1, 'Agregá al menos una línea.')
});
export type CrearCompraInput = z.input<typeof CrearCompraSchema>;

export const ActualizarCompraSchema = CrearCompraSchema;
export type ActualizarCompraInput = z.input<typeof ActualizarCompraSchema>;