import { z } from 'zod';

/** Cliente (013, FR-049). */
export const ClienteSchema = z.object({
	id: z.number().int(),
	nombre: z.string(),
	documento: z.string().nullable(),
	email: z.string().nullable(),
	telefono: z.string().nullable(),
	direccion: z.string().nullable(),
	activo: z.boolean()
});
export type Cliente = z.infer<typeof ClienteSchema>;

export const ListaClientesSchema = z.object({
	items: z.array(ClienteSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaClientes = z.infer<typeof ListaClientesSchema>;

const opcional = (max: number) =>
	z
		.string()
		.trim()
		.max(max, `Máximo ${max} caracteres.`)
		.nullish()
		.transform((v) => (v ? v : null));

/** Alta/edición de cliente: los opcionales vacíos viajan como `null`. */
export const NuevoClienteSchema = z.object({
	nombre: z.string().trim().min(1, 'El nombre es obligatorio.').max(120, 'Máximo 120 caracteres.'),
	documento: opcional(20),
	email: opcional(200),
	telefono: opcional(60),
	direccion: opcional(500)
});
export type NuevoClienteInput = z.input<typeof NuevoClienteSchema>;
