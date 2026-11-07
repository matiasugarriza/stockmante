import { z } from 'zod';

/** Proveedor (012). Espejo del `Proveedor` serializado por el backend. */
export const ProveedorSchema = z.object({
	id: z.number().int(),
	nombre: z.string(),
	cuit: z.string().nullable(),
	email: z.string().nullable(),
	telefono: z.string().nullable(),
	direccion: z.string().nullable(),
	activo: z.boolean()
});
export type Proveedor = z.infer<typeof ProveedorSchema>;

export const ListaProveedoresSchema = z.object({
	items: z.array(ProveedorSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaProveedores = z.infer<typeof ListaProveedoresSchema>;

const CuitSchema = z
	.string()
	.trim()
	.refine((valor) => valor === '' || /^\d{11}$/.test(valor), 'El CUIT debe tener 11 dígitos.')
	.optional()
	.default('');

const TextoOpcional = (max: number) =>
	z
		.string()
		.trim()
		.max(max, `Máximo ${max} caracteres.`)
		.optional()
		.default('');

/** Alta: el backend acepta cuit/email/teléfono/dirección ausentes o null. */
export const CrearProveedorSchema = z.object({
	nombre: z.string().trim().min(1, 'El nombre es obligatorio.').max(120, 'Máximo 120 caracteres.'),
	cuit: CuitSchema,
	email: TextoOpcional(200),
	telefono: TextoOpcional(60),
	direccion: TextoOpcional(500)
});
export type CrearProveedorInput = z.input<typeof CrearProveedorSchema>;

/** El PUT del backend recibe el DTO completo, igual que el alta. */
export const ActualizarProveedorSchema = CrearProveedorSchema;
export type ActualizarProveedorInput = z.input<typeof ActualizarProveedorSchema>;