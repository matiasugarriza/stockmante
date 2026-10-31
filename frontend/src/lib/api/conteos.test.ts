import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import { cancelarConteo, cerrarConteo, crearConteo, listarConteos, obtenerConteo } from './conteos';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const conteo = {
	id: 1,
	deposito_id: 1,
	deposito_nombre: 'Principal',
	estado: 'ABIERTO',
	usuario: { id: 2, nombre: 'Gerente' },
	creado_en: '2026-10-29T12:00:00Z',
	cerrado_en: null,
	cancelado_en: null,
	movimiento_ajuste_id: null,
	observacion: null,
	lineas: [
		{
			id: 1,
			producto_id: 10,
			producto_nombre: 'Yerba 1 kg',
			sku: 'YER-001',
			ubicacion_id: null,
			ubicacion_nombre: null,
			cantidad_sistema: 10,
			cantidad_contada: null,
			diferencia: null
		}
	]
};

describe('cliente de conteos', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarConteos arma los filtros y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [conteo], total: 1, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const pagina = await listarConteos({ depositoId: 1, estado: 'ABIERTO' });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/conteos?');
		expect(url).toContain('deposito_id=1');
		expect(url).toContain('estado=ABIERTO');
		expect(pagina.items).toHaveLength(1);
	});

	it('crearConteo hace POST y devuelve el conteo', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(conteo, 201));
		vi.stubGlobal('fetch', fetchMock);

		const creado = await crearConteo({ deposito_id: 1, observacion: null });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/conteos');
		expect(init.method).toBe('POST');
		expect(creado.id).toBe(1);
	});

	it('cerrarConteo hace POST a /cerrar', async () => {
		const cerrado = { ...conteo, estado: 'CERRADO', cerrado_en: '2026-10-29T13:00:00Z' };
		const fetchMock = vi.fn(async () => jsonResponse(cerrado));
		vi.stubGlobal('fetch', fetchMock);

		const res = await cerrarConteo(1);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/conteos/1/cerrar');
		expect(res.estado).toBe('CERRADO');
	});

	it('cancelarConteo hace POST a /cancelar', async () => {
		const cancelado = { ...conteo, estado: 'CANCELADO', cancelado_en: '2026-10-29T13:00:00Z' };
		const fetchMock = vi.fn(async () => jsonResponse(cancelado));
		vi.stubGlobal('fetch', fetchMock);

		const res = await cancelarConteo(1);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/conteos/1/cancelar');
		expect(res.estado).toBe('CANCELADO');
	});

	it('obtenerConteo hace GET por id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(conteo));
		vi.stubGlobal('fetch', fetchMock);

		const detalle = await obtenerConteo(1);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/conteos/1');
		expect(detalle.id).toBe(1);
	});
});
