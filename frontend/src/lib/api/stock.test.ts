import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import { listarStock, obtenerStockProducto } from './stock';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const pagina = { items: [], total: 0, pagina: 1, por_pagina: 50 };

describe('cliente de stock (010)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('serializa categoria_id, activo y producto_ids separados por coma', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(pagina));
		vi.stubGlobal('fetch', fetchMock);

		await listarStock({ categoriaId: 3, activo: 'all', productoIds: [4, 9, 12], depositoId: 2 });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		const query = new URL(url, 'http://x').searchParams;
		expect(query.get('categoria_id')).toBe('3');
		expect(query.get('activo')).toBe('all');
		expect(query.get('producto_ids')).toBe('4,9,12');
		expect(query.get('deposito_id')).toBe('2');
	});

	it('omite producto_ids cuando la lista está vacía', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(pagina));
		vi.stubGlobal('fetch', fetchMock);

		await listarStock({ productoIds: [] });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).not.toContain('producto_ids');
	});

	it('obtenerStockProducto propaga el 404', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{ error: { codigo: 'recurso_no_encontrado', mensaje: 'Producto no encontrado.' } },
					404
				)
			)
		);

		const error = await obtenerStockProducto(999).catch((e: unknown) => e);

		expect(error).toBeInstanceOf(ApiError);
		expect((error as ApiError).status).toBe(404);
		expect((error as ApiError).message).toBe('Producto no encontrado.');
	});
});
