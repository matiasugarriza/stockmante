import { z } from 'zod';
import { MotivoMovimientoSchema, TipoMovimientoSchema } from './movimientos';

/** Mensaje de la conversación en curso; vive solo en la memoria de la página (FR-077). */
export const MensajeHistorialSchema = z.object({
	rol: z.enum(['usuario', 'asistente']),
	texto: z.string()
});
export type MensajeHistorial = z.infer<typeof MensajeHistorialSchema>;

/** Propuesta del asistente: se ejecuta solo si el usuario la confirma (FR-067). */
export const AccionPropuestaSchema = z.object({
	tipo: TipoMovimientoSchema,
	deposito_id: z.number().int(),
	deposito_nombre: z.string(),
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	sku: z.string(),
	cantidad_delta: z.number().int(),
	motivo: MotivoMovimientoSchema,
	observacion: z.string().nullable()
});
export type AccionPropuesta = z.infer<typeof AccionPropuestaSchema>;

export const RespuestaAsistenteSchema = z.object({
	respuesta: z.string(),
	accion_propuesta: AccionPropuestaSchema.nullable()
});
export type RespuestaAsistente = z.infer<typeof RespuestaAsistenteSchema>;
