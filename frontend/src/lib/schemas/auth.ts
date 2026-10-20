import { z } from 'zod';

export const LoginSchema = z.object({
  email: z.email().max(255),
  password: z.string().min(1)
});
export type LoginInput = z.infer<typeof LoginSchema>;

export const CambioPasswordSchema = z.object({
  password_actual: z.string().min(1),
  password_nueva: z
    .string()
    .min(8, 'Mínimo 8 caracteres')
    .regex(/[A-Z]/, 'Debe tener al menos una mayúscula')
    .regex(/[0-9]/, 'Debe tener al menos un dígito')
});
export type CambioPasswordInput = z.infer<typeof CambioPasswordSchema>;

export const UsuarioSesionSchema = z.object({
  id: z.number().int(),
  nombre: z.string(),
  email: z.string(),
  rol: z.enum(['ADMIN', 'GERENTE', 'OPERARIO', 'CAJERO']),
  familia_rol: z.enum(['GESTION', 'OPERACION']),
  debe_cambiar_password: z.boolean()
});
export type UsuarioSesion = z.infer<typeof UsuarioSesionSchema>;

export const SesionResponseSchema = z.object({ usuario: UsuarioSesionSchema });
export type SesionResponse = z.infer<typeof SesionResponseSchema>;
