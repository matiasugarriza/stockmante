import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarCategoria,
	crearCategoria,
	desactivarCategoria,
	listarCategorias,
	obtenerCategoria,
	reactivarCategoria
} from './categorias';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const categoria = {
	id: 4,
	nombre: 'Almacén',
	descripcion: 'Productos secos',
	activo: true
};

describe('cliente de categorías', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarCategorias arma el filtro y parsea la lista plana', async () => {
		const fetchMock = vi.fn(async () => jsonResponse([categoria]));
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarCategorias('all');

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/categorias?activo=all');
		expect(lista[0]?.nombre).toBe('Almacén');
	});

	it('listarCategorias pide solo activas por defecto', async () => {
		const fetchMock = vi.fn(async () => jsonResponse([]));
		vi.stubGlobal('fetch', fetchMock);

		await listarCategorias();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/categorias?activo=true');
	});

	it('obtenerCategoria hace GET al id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(categoria));
		vi.stubGlobal('fetch', fetchMock);

		const encontrada = await obtenerCategoria(4);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/categorias/4');
		expect(init.method).toBeUndefined();
		expect(encontrada.id).toBe(4);
	});

	it('crearCategoria hace POST y devuelve la categoría', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(categoria, 201));
		vi.stubGlobal('fetch', fetchMock);

		const creada = await crearCategoria({ nombre: 'Almacén', descripcion: '' });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/categorias');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ nombre: 'Almacén', descripcion: '' });
		expect(creada.activo).toBe(true);
	});

	it('crearCategoria propaga el 409 categoria_duplicada', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'categoria_duplicada',
							mensaje: 'Ya existe una categoría con ese nombre.'
						}
					},
					409
				)
			)
		);

		await expect(crearCategoria({ nombre: 'Almacén' })).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'categoria_duplicada'
		} satisfies Partial<ApiError>);
	});

	it('actualizarCategoria hace PUT parcial', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...categoria, descripcion: null }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizada = await actualizarCategoria(4, { descripcion: '' });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/categorias/4');
		expect(init.method).toBe('PUT');
		expect(actualizada.descripcion).toBeNull();
	});

	it.each([
		['desactivarCategoria', desactivarCategoria, '/api/categorias/4/desactivar'],
		['reactivarCategoria', reactivarCategoria, '/api/categorias/4/reactivar']
	])('%s hace POST y resuelve con 204', async (_nombre, accion, path) => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(accion(4)).resolves.toBeUndefined();

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(path);
		expect(init.method).toBe('POST');
	});
});
