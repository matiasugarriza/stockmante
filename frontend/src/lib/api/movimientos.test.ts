import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import { crearMovimiento, listarMovimientos, obtenerMovimiento } from './movimientos';
import { listarStock, obtenerStockProducto } from './stock';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const movimiento = {
	id: 1,
	tipo: 'ENTRADA',
	deposito: { id: 1, nombre: 'Principal' },
	motivo: 'COMPRA',
	observacion: null,
	usuario: { id: 2, nombre: 'Operario' },
	creado_en: '2026-10-27T12:00:00Z',
	lineas: [
		{
			producto_id: 10,
			producto_nombre: 'Yerba 1 kg',
			sku: 'YER-001',
			ubicacion_id: null,
			ubicacion_nombre: null,
			cantidad_delta: 10,
			stock_resultante: 10
		}
	]
};

const stockItem = {
	producto_id: 10,
	nombre: 'Yerba 1 kg',
	sku: 'YER-001',
	categoria_id: 1,
	categoria_nombre: 'Almacén',
	unidad_medida: 'UNIDAD',
	activo: true,
	stock_actual: 10,
	stock_minimo: 5,
	bajo_minimo: false,
	stock_deposito: 3
};

describe('cliente de movimientos', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarMovimientos arma los filtros y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [movimiento], total: 1, pagina: 2, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const pagina = await listarMovimientos({
			tipo: 'SALIDA',
			depositoId: 1,
			productoId: 10,
			pagina: 2
		});

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/movimientos?');
		expect(url).toContain('tipo=SALIDA');
		expect(url).toContain('deposito_id=1');
		expect(url).toContain('producto_id=10');
		expect(url).toContain('pagina=2');
		expect(pagina.items).toHaveLength(1);
		expect(pagina.total).toBe(1);
	});

	it('crearMovimiento hace POST y devuelve el detalle', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(movimiento, 201));
		vi.stubGlobal('fetch', fetchMock);
		const alta = {
			tipo: 'ENTRADA' as const,
			deposito_id: 1,
			motivo: 'COMPRA' as const,
			observacion: null,
			lineas: [{ producto_id: 10, ubicacion_id: null, cantidad_delta: 10 }]
		};

		const creado = await crearMovimiento(alta);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/movimientos');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(alta);
		expect(creado.lineas[0].stock_resultante).toBe(10);
	});

	it('crearMovimiento propaga el 409 stock_insuficiente', async () => {
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
			crearMovimiento({
				tipo: 'SALIDA',
				deposito_id: 1,
				motivo: 'VENTA',
				observacion: null,
				lineas: [{ producto_id: 10, ubicacion_id: null, cantidad_delta: -5 }]
			})
		).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'stock_insuficiente'
		} satisfies Partial<ApiError>);
	});

	it('obtenerMovimiento hace GET por id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(movimiento));
		vi.stubGlobal('fetch', fetchMock);

		const detalle = await obtenerMovimiento(1);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/movimientos/1');
		expect(detalle.id).toBe(1);
	});
});

describe('cliente de stock', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarStock arma los filtros', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [stockItem], total: 1, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarStock({ q: 'yerba', depositoId: 1, bajoMinimo: true });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/stock?');
		expect(url).toContain('q=yerba');
		expect(url).toContain('deposito_id=1');
		expect(url).toContain('bajo_minimo=true');
		expect(lista.items[0].stock_deposito).toBe(3);
	});

	it('listarStock sin filtros no manda query', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [], total: 0, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		await listarStock();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/stock');
		expect(url).not.toContain('?');
	});

	it('obtenerStockProducto trae el desglose', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({
				producto_id: 10,
				nombre: 'Yerba 1 kg',
				sku: 'YER-001',
				unidad_medida: 'UNIDAD',
				activo: true,
				stock_actual: 10,
				stock_minimo: 5,
				bajo_minimo: false,
				desglose: [
					{
						deposito_id: 1,
						deposito_nombre: 'Principal',
						ubicacion_id: null,
						ubicacion_nombre: null,
						cantidad: 10
					}
				]
			})
		);
		vi.stubGlobal('fetch', fetchMock);

		const detalle = await obtenerStockProducto(10);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/stock/10');
		expect(detalle.desglose).toHaveLength(1);
	});
});
