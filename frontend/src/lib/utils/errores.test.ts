import { describe, expect, it } from 'vitest';
import { z } from 'zod';
import { ApiError } from '../api/client';
import { mensajeError } from './errores';

describe('mensajeError', () => {
	it('explica el 401 y el 403 sin filtrar detalles internos', () => {
		expect(mensajeError(new ApiError(401, 'no_autenticado', 'no_autenticado'), 'fallback')).toBe(
			'Tu sesión expiró. Volvé a iniciar sesión.'
		);
		expect(mensajeError(new ApiError(403, 'sin_permisos', 'sin_permisos'), 'fallback')).toBe(
			'No tenés permisos de gestión para esta acción.'
		);
	});

	it('conserva el mensaje del backend para 409/422', () => {
		expect(
			mensajeError(new ApiError(409, 'proveedor_duplicado', 'Ya existe un proveedor.'), 'fallback')
		).toBe('Ya existe un proveedor.');
	});

	it('explica un fallo de parseo Zod', () => {
		const resultado = z.object({ id: z.number() }).safeParse({ id: 'x' });
		expect(resultado.success).toBe(false);
		if (!resultado.success) {
			expect(mensajeError(resultado.error, 'fallback')).toContain('formato esperado');
		}
	});

	it('usa el fallback para valores desconocidos', () => {
		expect(mensajeError('boom', 'No se pudo cargar')).toBe('No se pudo cargar');
	});
});