import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarDeposito,
	actualizarUbicacion,
	crearDeposito,
	crearUbicacion,
	desactivarDeposito,
	desactivarUbicacion,
	eliminarDeposito,
	eliminarUbicacion,
	listarDepositos,
	listarUbicaciones,
	reactivarDeposito,
	reemplazarUbicacionesDeProducto,
	ubicacionesDeProducto
} from './depositos';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const deposito = {
	id: 4,
	nombre: 'Depósito Central',
	identificador: 'DEP-CENTRAL',
	direccion: null,
	activo: true
};

const ubicacion = {
	id: 10,
	deposito_id: 4,
	padre_id: null,
	nombre: 'Pasillo 1',
	tipo: 'PASILLO',
	activo: true
};

describe('cliente de depósitos y ubicaciones', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarDepositos arma el filtro y parsea la lista', async () => {
		const fetchMock = vi.fn(async () => jsonResponse([deposito]));
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarDepositos('all');

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/depositos?activo=all');
		expect(lista).toHaveLength(1);
	});

	it('crearDeposito hace POST y devuelve el depósito', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(deposito, 201));
		vi.stubGlobal('fetch', fetchMock);
		const alta = { nombre: 'Depósito Central', identificador: 'DEP-CENTRAL' };

		const creado = await crearDeposito(alta);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/depositos');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(alta);
		expect(creado.identificador).toBe('DEP-CENTRAL');
	});

	it('crearDeposito propaga el 409 de nombre duplicado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'deposito_nombre_duplicado',
							mensaje: 'Ya existe un depósito con ese nombre'
						}
					},
					409
				)
			)
		);

		await expect(
			crearDeposito({ nombre: 'Repetido', identificador: 'DEP-2' })
		).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'deposito_nombre_duplicado'
		} satisfies Partial<ApiError>);
	});

	it('actualizarDeposito hace PUT parcial', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...deposito, nombre: 'Nuevo' }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizado = await actualizarDeposito(4, { nombre: 'Nuevo' });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/depositos/4');
		expect(init.method).toBe('PUT');
		expect(actualizado.nombre).toBe('Nuevo');
	});

	it('desactivar y reactivar usan POST sin cuerpo', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await desactivarDeposito(4);
		await reactivarDeposito(4);

		const llamadas = fetchMock.mock.calls as unknown as [string, RequestInit][];
		const urls = llamadas.map(([url]) => url);
		expect(urls[0]).toContain('/api/depositos/4/desactivar');
		expect(urls[1]).toContain('/api/depositos/4/reactivar');
	});

	it('eliminarDeposito hace DELETE', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await eliminarDeposito(4);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/depositos/4');
		expect(init.method).toBe('DELETE');
	});

	it('listarUbicaciones pide el depósito y devuelve la lista plana', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [ubicacion] }));
		vi.stubGlobal('fetch', fetchMock);

		const items = await listarUbicaciones(4);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/depositos/4/ubicaciones');
		expect(items).toHaveLength(1);
	});

	it('crearUbicacion envía depósito, padre y tipo', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...ubicacion, padre_id: 9 }, 201));
		vi.stubGlobal('fetch', fetchMock);
		const alta = {
			deposito_id: 4,
			padre_id: 9,
			nombre: 'Estantería A',
			tipo: 'ESTANTERIA' as const
		};

		const creada = await crearUbicacion(alta);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/ubicaciones');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(alta);
		expect(creada.padre_id).toBe(9);
	});

	it('actualizarUbicacion permite mover a la raíz con null', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(ubicacion));
		vi.stubGlobal('fetch', fetchMock);

		await actualizarUbicacion(10, { padre_id: null });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/ubicaciones/10');
		expect(JSON.parse(init.body as string)).toEqual({ padre_id: null });
	});

	it('desactivarUbicacion y eliminarUbicacion apuntan al id', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await desactivarUbicacion(10);
		await eliminarUbicacion(10);

		const llamadas = fetchMock.mock.calls as unknown as [string, RequestInit][];
		expect(llamadas[0][0]).toContain('/api/ubicaciones/10/desactivar');
		expect(llamadas[1][0]).toContain('/api/ubicaciones/10');
		expect(llamadas[1][1].method).toBe('DELETE');
	});

	it('ubicacionesDeProducto consulta el producto', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [ubicacion] }));
		vi.stubGlobal('fetch', fetchMock);

		const items = await ubicacionesDeProducto(7);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/7/ubicaciones');
		expect(items[0].id).toBe(10);
	});

	it('reemplazarUbicacionesDeProducto hace PUT del conjunto completo', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await reemplazarUbicacionesDeProducto(7, [10, 11]);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/7/ubicaciones');
		expect(init.method).toBe('PUT');
		expect(JSON.parse(init.body as string)).toEqual({ ubicacion_ids: [10, 11] });
	});
});
