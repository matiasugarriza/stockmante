import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarCompra,
	anularCompra,
	confirmarCompra,
	crearCompra,
	eliminarCompra,
	listarCompras,
	obtenerCompra
} from './compras';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const compra = {
	id: 12,
	proveedor_id: 3,
	proveedor_nombre: 'Distribuidora Norte',
	deposito_id: 1,
	deposito_nombre: 'Depósito central',
	numero: 'OC-001',
	estado: 'BORRADOR' as const,
	observacion: null,
	total_costo: 76.5,
	usuario_id: 2,
	usuario_nombre: 'Gerente',
	creado_en: '2026-11-06T12:00:00Z',
	confirmado_en: null,
	anulado_en: null
};

const detalle = {
	compra,
	lineas: [
		{
			id: 1,
			producto_id: 9,
			producto_nombre: 'Yerba',
			sku: 'YER-001',
			cantidad: 3,
			costo_unitario: 25.5,
			subtotal: 76.5
		}
	]
};

describe('cliente de compras', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarCompras arma filtros y paginación', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [compra], total: 1, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		await listarCompras({ proveedorId: 3, estado: 'BORRADOR', pagina: 1, porPagina: 20 });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(
			'/api/compras?proveedor_id=3&estado=BORRADOR&pagina=1&por_pagina=20'
		);
	});

	it('obtenerCompra parsea el detalle con líneas', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(detalle));
		vi.stubGlobal('fetch', fetchMock);

		const encontrada = await obtenerCompra(12);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/compras/12');
		expect(encontrada.lineas[0]?.cantidad).toBe(3);
	});

	it('crearCompra envía null en numero/observacion vacíos', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(detalle, 201));
		vi.stubGlobal('fetch', fetchMock);

		await crearCompra({
			proveedor_id: 3,
			deposito_id: 1,
			numero: '',
			observacion: '',
			lineas: [{ producto_id: 9, cantidad: 3, costo_unitario: 25.5 }]
		});

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/compras');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({
			proveedor_id: 3,
			deposito_id: 1,
			numero: null,
			observacion: null,
			lineas: [{ producto_id: 9, cantidad: 3, costo_unitario: 25.5 }]
		});
	});

	it('actualizarCompra hace PUT al id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(detalle));
		vi.stubGlobal('fetch', fetchMock);

		await actualizarCompra(12, {
			proveedor_id: 3,
			deposito_id: 1,
			lineas: [{ producto_id: 9, cantidad: 4, costo_unitario: 25.5 }]
		});

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/compras/12');
		expect(init.method).toBe('PUT');
	});

	it.each([
		['confirmarCompra', confirmarCompra, '/api/compras/12/confirmar'],
		['anularCompra', anularCompra, '/api/compras/12/anular']
	])('%s hace POST y parsea el detalle', async (_nombre, accion, path) => {
		const fetchMock = vi.fn(async () => jsonResponse(detalle));
		vi.stubGlobal('fetch', fetchMock);

		const resultado = await accion(12);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(path);
		expect(init.method).toBe('POST');
		expect(resultado.compra.id).toBe(12);
	});

	it('eliminarCompra resuelve con 204', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(eliminarCompra(12)).resolves.toBeUndefined();

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/compras/12');
		expect(init.method).toBe('DELETE');
	});

	it('confirmarCompra propaga el 409 compra_ya_confirmada', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'compra_ya_confirmada',
							mensaje: 'La compra ya fue confirmada; no se puede confirmar dos veces.'
						}
					},
					409
				)
			)
		);

		await expect(confirmarCompra(12)).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'compra_ya_confirmada'
		} satisfies Partial<ApiError>);
	});
});