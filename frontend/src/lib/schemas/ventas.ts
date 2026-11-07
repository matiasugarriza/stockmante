import { z } from 'zod';

export const MedioPagoSchema = z.enum(['EFECTIVO', 'TARJETA', 'TRANSFERENCIA']);
export type MedioPago = z.infer<typeof MedioPagoSchema>;

export const EstadoVentaSchema = z.enum(['CONFIRMADA', 'ANULADA']);
export type EstadoVenta = z.infer<typeof EstadoVentaSchema>;

/** Línea de venta con el saldo que dejó en el depósito (013, FR-051). */
export const VentaLineaSchema = z.object({
	id: z.number().int(),
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	sku: z.string().nullable(),
	cantidad: z.number().int(),
	precio_unitario: z.number(),
	subtotal: z.number(),
	stock_resultante: z.number().int().nullable()
});
export type VentaLinea = z.infer<typeof VentaLineaSchema>;

export const VentaSchema = z.object({
	id: z.number().int(),
	numero: z.string(),
	deposito_id: z.number().int(),
	deposito_nombre: z.string(),
	cliente_id: z.number().int().nullable(),
	cliente_nombre: z.string().nullable(),
	medio_pago: MedioPagoSchema,
	subtotal: z.number(),
	descuento: z.number(),
	total: z.number(),
	estado: EstadoVentaSchema,
	observacion: z.string().nullable(),
	usuario_id: z.number().int(),
	usuario_nombre: z.string(),
	creado_en: z.string(),
	anulado_en: z.string().nullable()
});
export type Venta = z.infer<typeof VentaSchema>;

export const VentaDetalleSchema = z.object({
	venta: VentaSchema,
	lineas: z.array(VentaLineaSchema)
});
export type VentaDetalle = z.infer<typeof VentaDetalleSchema>;

export const ListaVentasSchema = z.object({
	items: z.array(VentaSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaVentas = z.infer<typeof ListaVentasSchema>;

/** Venta a registrar: sin `precio_unitario` (el precio sale del catálogo, FR-050). */
export const NuevaVentaSchema = z.object({
	deposito_id: z.number().int().positive('Elegí un depósito.'),
	cliente_id: z
		.number()
		.int()
		.positive()
		.nullish()
		.transform((v) => v ?? null),
	medio_pago: MedioPagoSchema,
	descuento: z.number().nonnegative('El descuento no puede ser negativo.').optional().default(0),
	observacion: z
		.string()
		.trim()
		.max(500, 'Máximo 500 caracteres.')
		.nullish()
		.transform((v) => (v ? v : null)),
	lineas: z
		.array(
			z.object({
				producto_id: z.number().int().positive(),
				cantidad: z
					.number()
					.int('La cantidad debe ser entera.')
					.positive('La cantidad debe ser mayor a cero.')
			})
		)
		.min(1, 'Agregá al menos un producto.')
});
export type NuevaVentaInput = z.input<typeof NuevaVentaSchema>;
