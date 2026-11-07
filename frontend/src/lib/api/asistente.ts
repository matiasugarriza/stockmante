/**
 * Cliente del asistente de inventario (017). La consulta nunca escribe: una propuesta
 * se ejecuta con `confirmarPropuesta`, que usa el alta normal de movimientos (FR-067).
 * Errores (401/429/503/504) llegan como `ApiError` con el mensaje real de la API.
 */
import { api } from './client';
import { crearMovimiento } from './movimientos';
import {
	RespuestaAsistenteSchema,
	type AccionPropuesta,
	type MensajeHistorial,
	type RespuestaAsistente
} from '../schemas/asistente';
import type { MovimientoDetalle } from '../schemas/movimientos';

/** FR-077: mensajes previos que se mandan como contexto. */
export const MAX_HISTORIAL = 10;

/** Devuelve un historial nuevo con `mensaje` al final y solo los últimos 10. */
export function agregarAlHistorial(
	historial: readonly MensajeHistorial[],
	mensaje: MensajeHistorial
): MensajeHistorial[] {
	return [...historial, mensaje].slice(-MAX_HISTORIAL);
}

export async function consultarAsistente(
	mensaje: string,
	historial: readonly MensajeHistorial[]
): Promise<RespuestaAsistente> {
	const body = await api<unknown>('/api/asistente/consulta', {
		method: 'POST',
		body: JSON.stringify({ mensaje, historial: historial.slice(-MAX_HISTORIAL) })
	});
	return RespuestaAsistenteSchema.parse(body);
}

/** 201 con el movimiento y el stock resultante; el autor sale de la sesión. */
export async function confirmarPropuesta(propuesta: AccionPropuesta): Promise<MovimientoDetalle> {
	return crearMovimiento({
		tipo: propuesta.tipo,
		deposito_id: propuesta.deposito_id,
		motivo: propuesta.motivo,
		observacion: propuesta.observacion,
		lineas: [{ producto_id: propuesta.producto_id, cantidad_delta: propuesta.cantidad_delta }]
	});
}
