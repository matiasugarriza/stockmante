import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarProducto,
	crearProducto,
	desactivarProducto,
	listarProductos,
	obtenerProducto,
	reactivarProducto
} from './productos';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const producto = {
	id: 9,
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: null,
	categoria: { id: 3, nombre: 'Almacén' },
	unidad_medida: 'UNIDAD',
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6,
	activo: true
};

const alta = {
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: '',
	categoria_id: 3,
	unidad_medida: 'UNIDAD' as const,
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6
};

describe('cliente de productos', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarProductos arma el query completo y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [producto], total: 1, pagina: 2, por_pagina: 50 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarProductos({
			q: 'yerba',
			categoriaId: 3,
			activo: 'all',
			pagina: 2,
			porPagina: 50
		});

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(
			'/api/productos?q=yerba&categoria_id=3&activo=all&pagina=2&por_pagina=50'
		);
		expect(lista.total).toBe(1);
	});

	it('listarProductos omite `activo` para usar el default del backend', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [], total: 0, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		await listarProductos();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos');
		expect(url).not.toContain('activo=');
	});

	it('crearProducto hace POST y devuelve el producto', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(producto, 201));
		vi.stubGlobal('fetch', fetchMock);

		const creado = await crearProducto(alta);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(alta);
		expect(creado.categoria.id).toBe(3);
	});

	it('crearProducto propaga el 409 sku_duplicado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'sku_duplicado',
							mensaje: 'Ya existe un producto con el SKU YERBA-1K. Elegí otro.'
						}
					},
					409
				)
			)
		);

		await expect(crearProducto(alta)).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'sku_duplicado'
		} satisfies Partial<ApiError>);
	});

	it('obtenerProducto hace GET al id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(producto));
		vi.stubGlobal('fetch', fetchMock);

		const encontrado = await obtenerProducto(9);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/9');
		expect(encontrado.sku).toBe('YERBA-1K');
	});

	it('actualizarProducto hace PUT parcial', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...producto, precio_venta: 2000.75 }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizado = await actualizarProducto(9, { precio_venta: 2000.75 });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/9');
		expect(init.method).toBe('PUT');
		expect(actualizado.precio_venta).toBe(2000.75);
	});

	it.each([
		['desactivarProducto', desactivarProducto, '/api/productos/9/desactivar'],
		['reactivarProducto', reactivarProducto, '/api/productos/9/reactivar']
	])('%s hace POST y resuelve con 204', async (_nombre, accion, path) => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(accion(9)).resolves.toBeUndefined();

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(path);
		expect(init.method).toBe('POST');
	});
});
