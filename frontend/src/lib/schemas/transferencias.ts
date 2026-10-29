import { z } from 'zod';

const DepositoBreveSchema = z.object({
	id: z.number().int(),
	nombre: z.string()
});

const UsuarioBreveSchema = z.object({
	id: z.number().int(),
	nombre: z.string()
});

/** Transferencia entre depósitos (008, contracts/api.md). */
export const TransferenciaSchema = z.object({
	id: z.number().int(),
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	sku: z.string(),
	deposito_origen: DepositoBreveSchema,
	deposito_destino: DepositoBreveSchema,
	cantidad: z.number().int(),
	stock_origen_resultante: z.number().int(),
	stock_destino_resultante: z.number().int(),
	usuario: UsuarioBreveSchema,
	creado_en: z.string(),
	movimiento_salida_id: z.number().int(),
	movimiento_entrada_id: z.number().int(),
	observacion: z.string().nullable()
});
export type Transferencia = z.infer<typeof TransferenciaSchema>;

export const ListaTransferenciasSchema = z.object({
	items: z.array(TransferenciaSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaTransferencias = z.infer<typeof ListaTransferenciasSchema>;

/** Alta de transferencia. */
export const CrearTransferenciaSchema = z.object({
	producto_id: z.number().int().positive('Elegí un producto'),
	deposito_origen_id: z.number().int().positive('Elegí un depósito de origen'),
	deposito_destino_id: z.number().int().positive('Elegí un depósito de destino'),
	cantidad: z.number().int().positive('La cantidad debe ser mayor que cero'),
	ubicacion_origen_id: z.number().int().positive().nullish(),
	ubicacion_destino_id: z.number().int().positive().nullish(),
	observacion: z
		.string()
		.trim()
		.max(500, 'Máximo 500 caracteres')
		.nullish()
		.transform((v) => (v ? v : null))
});
export type CrearTransferenciaInput = z.infer<typeof CrearTransferenciaSchema>;
