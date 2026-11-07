import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarProveedor,
	crearProveedor,
	desactivarProveedor,
	listarProveedores,
	obtenerProveedor,
	reactivarProveedor
} from './proveedores';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const proveedor = {
	id: 7,
	nombre: 'Distribuidora Norte',
	cuit: '20123456789',
	email: null,
	telefono: null,
	direccion: null,
	activo: true
};

describe('cliente de proveedores', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarProveedores arma filtros y paginación', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [proveedor], total: 1, pagina: 2, por_pagina: 5 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarProveedores({ activo: 'all', pagina: 2, porPagina: 5 });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/proveedores?activo=all&pagina=2&por_pagina=5');
		expect(lista.items[0]?.nombre).toBe('Distribuidora Norte');
	});

	it('listarProveedores sin filtros no agrega query', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [], total: 0, pagina: 1, por_pagina: 20 }));
		vi.stubGlobal('fetch', fetchMock);

		await listarProveedores();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/proveedores$/);
	});

	it('obtenerProveedor hace GET al id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(proveedor));
		vi.stubGlobal('fetch', fetchMock);

		const encontrado = await obtenerProveedor(7);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/proveedores/7');
		expect(encontrado.id).toBe(7);
	});

	it('crearProveedor envía null en los campos vacíos', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(proveedor, 201));
		vi.stubGlobal('fetch', fetchMock);

		await crearProveedor({ nombre: 'Distribuidora Norte', cuit: '', email: '', telefono: '', direccion: '' });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/proveedores');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({
			nombre: 'Distribuidora Norte',
			cuit: null,
			email: null,
			telefono: null,
			direccion: null
		});
	});

	it('actualizarProveedor hace PUT al id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...proveedor, nombre: 'Norte II' }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizado = await actualizarProveedor(7, { nombre: 'Norte II' });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/proveedores/7');
		expect(init.method).toBe('PUT');
		expect(actualizado.nombre).toBe('Norte II');
	});

	it('crearProveedor propaga el 409 proveedor_duplicado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'proveedor_duplicado',
							mensaje: 'Ya existe un proveedor con ese nombre.'
						}
					},
					409
				)
			)
		);

		await expect(crearProveedor({ nombre: 'Distribuidora Norte' })).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'proveedor_duplicado'
		} satisfies Partial<ApiError>);
	});

	it.each([
		['desactivarProveedor', desactivarProveedor, '/api/proveedores/7/desactivar'],
		['reactivarProveedor', reactivarProveedor, '/api/proveedores/7/activar']
	])('%s hace POST y devuelve el proveedor actualizado', async (_nombre, accion, path) => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...proveedor, activo: false }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizado = await accion(7);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(path);
		expect(init.method).toBe('POST');
		expect(actualizado.activo).toBe(false);
	});
});