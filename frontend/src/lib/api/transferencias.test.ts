import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import { crearTransferencia, listarTransferencias, obtenerTransferencia } from './transferencias';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const transferencia = {
	id: 1,
	producto_id: 10,
	producto_nombre: 'Yerba 1 kg',
	sku: 'YER-001',
	deposito_origen: { id: 1, nombre: 'Principal' },
	deposito_destino: { id: 2, nombre: 'Sucursal' },
	cantidad: 8,
	stock_origen_resultante: 12,
	stock_destino_resultante: 8,
	usuario: { id: 3, nombre: 'Juan' },
	creado_en: '2026-10-29T12:00:00Z',
	movimiento_salida_id: 10,
	movimiento_entrada_id: 11,
	observacion: 'Abastecer sucursal'
};

describe('cliente de transferencias', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarTransferencias arma los filtros y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [transferencia], total: 1, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const pagina = await listarTransferencias({
			productoId: 10,
			depositoOrigenId: 1,
			depositoDestinoId: 2,
			desde: '2026-10-01',
			hasta: '2026-10-31'
		});

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/transferencias?');
		expect(url).toContain('producto_id=10');
		expect(url).toContain('deposito_origen_id=1');
		expect(url).toContain('deposito_destino_id=2');
		expect(url).toContain('desde=2026-10-01');
		expect(url).toContain('hasta=2026-10-31');
		expect(pagina.items).toHaveLength(1);
		expect(pagina.total).toBe(1);
	});

	it('crearTransferencia hace POST y devuelve el detalle', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(transferencia, 201));
		vi.stubGlobal('fetch', fetchMock);
		const alta = {
			producto_id: 10,
			deposito_origen_id: 1,
			deposito_destino_id: 2,
			cantidad: 8,
			ubicacion_origen_id: null,
			ubicacion_destino_id: null,
			observacion: 'Abastecer sucursal'
		};

		const creado = await crearTransferencia(alta);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/transferencias');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(alta);
		expect(creado.stock_origen_resultante).toBe(12);
		expect(creado.stock_destino_resultante).toBe(8);
	});

	it('crearTransferencia propaga el 409 stock_insuficiente', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'stock_insuficiente',
							mensaje: 'Stock insuficiente en Principal: 3 disponibles, 5 solicitadas.'
						}
					},
					409
				)
			)
		);

		await expect(
			crearTransferencia({
				producto_id: 10,
				deposito_origen_id: 1,
				deposito_destino_id: 2,
				cantidad: 5,
				ubicacion_origen_id: null,
				ubicacion_destino_id: null,
				observacion: null
			})
		).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'stock_insuficiente'
		} satisfies Partial<ApiError>);
	});

	it('obtenerTransferencia hace GET por id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(transferencia));
		vi.stubGlobal('fetch', fetchMock);

		const detalle = await obtenerTransferencia(1);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/transferencias/1');
		expect(detalle.id).toBe(1);
	});
});
