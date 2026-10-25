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

/** Alta: el backend genera la contraseña temporal y la envía por correo. */
export const CrearUsuarioSchema = z.object({
	nombre: z.string().trim().min(1, 'Ingresá el nombre').max(120, 'Máximo 120 caracteres'),
	email: z.email('Ingresá un email válido').max(255),
	rol: RolSchema
});
export type CrearUsuarioInput = z.infer<typeof CrearUsuarioSchema>;

export const ActualizarUsuarioSchema = CrearUsuarioSchema.partial();
export type ActualizarUsuarioInput = z.infer<typeof ActualizarUsuarioSchema>;
