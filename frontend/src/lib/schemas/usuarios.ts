import { z } from 'zod';

export const RolSchema = z.enum(['ADMIN', 'GERENTE', 'OPERARIO', 'CAJERO']);
export type Rol = z.infer<typeof RolSchema>;

export const FamiliaRolSchema = z.enum(['GESTION', 'OPERACION']);
export type FamiliaRol = z.infer<typeof FamiliaRolSchema>;

/** Item de GET /usuarios y GET /usuarios/{id} (contracts/api.md, sin hash). */
export const UsuarioSchema = z.object({
	id: z.number().int(),
	nombre: z.string(),
	email: z.string(),
	rol: RolSchema,
	familia_rol: FamiliaRolSchema,
	activo: z.boolean(),
	password_temporal: z.boolean(),
	creado_en: z.string()
});
export type Usuario = z.infer<typeof UsuarioSchema>;

export const ListaUsuariosSchema = z.object({
	items: z.array(UsuarioSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaUsuarios = z.infer<typeof ListaUsuariosSchema>;

/**
 * Política de 001 para la contraseña **manual** (FR-008b): ausente ⇒ el
 * backend genera la automática; presente ⇒ ≥8, ≥1 mayúscula, ≥1 dígito.
 */
const PoliticaPasswordSchema = z
	.string()
	.min(8, 'Mínimo 8 caracteres')
	.regex(/[A-Z]/, 'Debe tener una mayúscula')
	.regex(/[0-9]/, 'Debe tener un dígito');

/** Alta: nombre/contraseña; `password` ausente = automática (FR-008b). */
export const CrearUsuarioSchema = z.object({
	nombre: z.string().trim().min(1, 'Ingresá el nombre').max(120, 'Máximo 120 caracteres'),
	email: z.email('Ingresá un email válido').max(255),
	rol: RolSchema,
	password: PoliticaPasswordSchema.optional()
});
export type CrearUsuarioInput = z.infer<typeof CrearUsuarioSchema>;

/** PUT parcial: nunca envía password (plan D8, exclusivo de la regeneración). */
export const ActualizarUsuarioSchema = CrearUsuarioSchema.omit({ password: true }).partial();
export type ActualizarUsuarioInput = z.infer<typeof ActualizarUsuarioSchema>;

/** 201 de POST /usuarios: usuario público + clave en claro para la modal. */
export const UsuarioCreadoSchema = z.object({
	usuario: UsuarioSchema,
	password_temporal: z.string().min(1)
});
export type UsuarioCreado = z.infer<typeof UsuarioCreadoSchema>;

/** 200 de POST /usuarios/{id}/password-temporal: la clave nueva para la modal. */
export const ClaveTemporalGeneradaSchema = z.object({
	password_temporal: z.string().min(1)
});
export type ClaveTemporalGenerada = z.infer<typeof ClaveTemporalGeneradaSchema>;
