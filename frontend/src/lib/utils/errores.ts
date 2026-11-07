import { ZodError } from 'zod';
import { ApiError } from '../api/client';

/**
 * Traduce un fallo de la capa API a un mensaje accionable.
 * 401/403 y los errores de parseo se explican; el resto conserva el mensaje
 * del backend (409 duplicado, 422 validación, 404, etc.).
 */
export function mensajeError(error: unknown, fallback: string): string {
	if (error instanceof ZodError) {
		return 'La respuesta del servidor no tiene el formato esperado.';
	}
	if (error instanceof ApiError) {
		if (error.status === 401) {
			return 'Tu sesión expiró. Volvé a iniciar sesión.';
		}
		if (error.status === 403) {
			return 'No tenés permisos de gestión para esta acción.';
		}
		return error.message;
	}
	if (error instanceof Error) {
		return error.message;
	}
	return fallback;
}