import { z } from 'zod';

/** Item de GET /categorias (contracts/api.md). */
export const CategoriaSchema = z.object({
	id: z.number().int(),
	nombre: z.string(),
	descripcion: z.string().nullable(),
	activo: z.boolean()
});
export type Categoria = z.infer<typeof CategoriaSchema>;

export const ListaCategoriasSchema = z.array(CategoriaSchema);

/** Alta: el backend recorta y exige 1–80; descripción vacía → null. */
export const CrearCategoriaSchema = z.object({
	nombre: z.string().trim().min(1, 'Ingresá el nombre').max(80, 'Máximo 80 caracteres'),
	descripcion: z.string().trim().max(255, 'Máximo 255 caracteres').nullish()
});
export type CrearCategoriaInput = z.infer<typeof CrearCategoriaSchema>;

/** Edición parcial: al menos un campo y nunca `activo` (se cambia por acciones). */
export const ActualizarCategoriaSchema = CrearCategoriaSchema.partial().refine(
	(datos) => Object.keys(datos).length > 0,
	{ message: 'Indicá al menos un campo para actualizar.' }
);
export type ActualizarCategoriaInput = z.infer<typeof ActualizarCategoriaSchema>;
