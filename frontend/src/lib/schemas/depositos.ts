import { z } from 'zod';

/** Depósito (005, contracts/api.md). */
export const DepositoSchema = z.object({
	id: z.number().int(),
	nombre: z.string(),
	identificador: z.string(),
	direccion: z.string().nullable(),
	activo: z.boolean()
});
export type Deposito = z.infer<typeof DepositoSchema>;

export const ListaDepositosSchema = z.array(DepositoSchema);

export const TipoUbicacionSchema = z.enum(['PASILLO', 'ESTANTERIA', 'NIVEL', 'POSICION']);
export type TipoUbicacion = z.infer<typeof TipoUbicacionSchema>;

/** Nodo jerárquico dentro de un depósito. */
export const UbicacionSchema = z.object({
	id: z.number().int(),
	deposito_id: z.number().int(),
	padre_id: z.number().int().nullable(),
	nombre: z.string(),
	tipo: TipoUbicacionSchema,
	activo: z.boolean()
});
export type Ubicacion = z.infer<typeof UbicacionSchema>;

export const ListaUbicacionesSchema = z.object({
	items: z.array(UbicacionSchema)
});
export type ListaUbicaciones = z.infer<typeof ListaUbicacionesSchema>;

export const CrearDepositoSchema = z.object({
	nombre: z.string().trim().min(1, 'El nombre es obligatorio').max(120),
	identificador: z.string().trim().min(1, 'El identificador es obligatorio').max(40),
	direccion: z.string().max(500).optional()
});
export type CrearDepositoInput = z.infer<typeof CrearDepositoSchema>;

export const ActualizarDepositoSchema = z.object({
	nombre: z.string().trim().min(1).max(120).optional(),
	identificador: z.string().trim().min(1).max(40).optional(),
	direccion: z.string().max(500).nullable().optional()
});
export type ActualizarDepositoInput = z.infer<typeof ActualizarDepositoSchema>;

export const CrearUbicacionSchema = z.object({
	deposito_id: z.number().int().positive(),
	padre_id: z.number().int().positive().nullable().optional(),
	nombre: z.string().trim().min(1, 'El nombre es obligatorio').max(80),
	tipo: TipoUbicacionSchema
});
export type CrearUbicacionInput = z.infer<typeof CrearUbicacionSchema>;

/**
 * PUT parcial: un campo ausente se conserva. `padre_id: null` mueve la
 * ubicación a la raíz; `deposito_id` solo cambia si no tiene dependencias.
 */
export const ActualizarUbicacionSchema = z.object({
	nombre: z.string().trim().min(1).max(80).optional(),
	tipo: TipoUbicacionSchema.optional(),
	padre_id: z.number().int().positive().nullable().optional(),
	deposito_id: z.number().int().positive().optional()
});
export type ActualizarUbicacionInput = z.infer<typeof ActualizarUbicacionSchema>;
