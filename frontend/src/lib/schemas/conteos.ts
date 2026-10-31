import { z } from 'zod';

const UsuarioBreveSchema = z.object({
	id: z.number().int(),
	nombre: z.string()
});

export const ConteoLineaSchema = z.object({
	id: z.number().int(),
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	sku: z.string(),
	ubicacion_id: z.number().int().nullable(),
	ubicacion_nombre: z.string().nullable(),
	cantidad_sistema: z.number().int(),
	cantidad_contada: z.number().int().nullable(),
	diferencia: z.number().int().nullable()
});
export type ConteoLinea = z.infer<typeof ConteoLineaSchema>;

export const ConteoSchema = z.object({
	id: z.number().int(),
	deposito_id: z.number().int(),
	deposito_nombre: z.string(),
	estado: z.enum(['ABIERTO', 'CERRADO', 'CANCELADO']),
	usuario: UsuarioBreveSchema,
	creado_en: z.string(),
	cerrado_en: z.string().nullable(),
	cancelado_en: z.string().nullable(),
	movimiento_ajuste_id: z.number().int().nullable(),
	observacion: z.string().nullable(),
	lineas: z.array(ConteoLineaSchema)
});
export type Conteo = z.infer<typeof ConteoSchema>;

export const ListaConteosSchema = z.object({
	items: z.array(ConteoSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaConteos = z.infer<typeof ListaConteosSchema>;

export const CrearConteoSchema = z.object({
	deposito_id: z.number().int().positive('Elegí un depósito'),
	observacion: z
		.string()
		.trim()
		.max(500, 'Máximo 500 caracteres')
		.nullish()
		.transform((v) => (v ? v : null))
});
export type CrearConteoInput = z.infer<typeof CrearConteoSchema>;

export const CargarLineasSchema = z.object({
	lineas: z.array(
		z.object({
			linea_id: z.number().int().positive(),
			cantidad_contada: z.number().int().min(0, 'La cantidad no puede ser negativa')
		})
	)
});
export type CargarLineasInput = z.infer<typeof CargarLineasSchema>;