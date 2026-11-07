import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import {
	activarCliente,
	actualizarCliente,
	crearCliente,
	desactivarCliente,
	listarClientes,
	obtenerCliente
} from './clientes';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const cliente = {
	id: 7,
	nombre: 'Ana Pérez',
	documento: '30111222',
	email: null,
	telefono: null,
	direccion: null,
	activo: true
};

describe('cliente de clientes (013)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarClientes pasa el filtro activo y la paginación', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [cliente], total: 1, pagina: 1, por_pagina: 20 }));
		vi.stubGlobal('fetch', fetchMock);

		const r = await listarClientes({ activo: false, pagina: 2 });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('activo=false');
		expect(url).toContain('pagina=2');
		expect(r.items[0].nombre).toBe('Ana Pérez');
	});

	it('crear y actualizar normalizan los opcionales vacíos a null', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(cliente, 201));
		vi.stubGlobal('fetch', fetchMock);

		await crearCliente({ nombre: ' Ana Pérez ', documento: '', email: '  ' });
		await actualizarCliente(7, { nombre: 'Ana', documento: '30111222' });

		const [, alta] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(JSON.parse(String(alta.body))).toEqual({
			nombre: 'Ana Pérez',
			documento: null,
			email: null,
			telefono: null,
			direccion: null
		});
		const [url, edicion] = fetchMock.mock.calls[1] as unknown as [string, RequestInit];
		expect(url).toContain('/api/clientes/7');
		expect(edicion.method).toBe('PUT');
	});

	it('obtener, activar y desactivar usan sus rutas', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(cliente));
		vi.stubGlobal('fetch', fetchMock);

		await obtenerCliente(7);
		await activarCliente(7);
		await desactivarCliente(7);

		const urls = fetchMock.mock.calls.map((c) => String((c as unknown as [string])[0]));
		expect(urls[0]).toContain('/api/clientes/7');
		expect(urls[1]).toContain('/api/clientes/7/activar');
		expect(urls[2]).toContain('/api/clientes/7/desactivar');
	});

	it('propaga el 409 documento_duplicado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{ error: { codigo: 'documento_duplicado', mensaje: 'Ya existe un cliente con ese documento.' } },
					409
				)
			)
		);
		const error = await crearCliente({ nombre: 'Otro', documento: '30111222' }).catch((e: unknown) => e);
		expect(error).toBeInstanceOf(ApiError);
		expect((error as ApiError).codigo).toBe('documento_duplicado');
	});
});
