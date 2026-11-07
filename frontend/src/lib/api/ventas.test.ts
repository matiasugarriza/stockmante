import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import { anularVenta, crearVenta, listarVentas, obtenerVenta } from './ventas';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const venta = {
	id: 31,
	numero: 'V-000031',
	deposito_id: 1,
	deposito_nombre: 'Principal',
	cliente_id: null,
	cliente_nombre: null,
	medio_pago: 'EFECTIVO',
	subtotal: 15000,
	descuento: 1000,
	total: 14000,
	estado: 'CONFIRMADA',
	observacion: null,
	usuario_id: 4,
	usuario_nombre: 'Caja 1',
	creado_en: '2026-11-07T12:30:00Z',
	anulado_en: null
};

const detalle = {
	venta,
	lineas: [
		{
			id: 91,
			producto_id: 12,
			producto_nombre: 'Cemento 50 kg',
			sku: 'CEM-50',
			cantidad: 3,
			precio_unitario: 3000,
			subtotal: 9000,
			stock_resultante: 7
		}
	]
};

describe('cliente de ventas (013)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('crearVenta envía el cuerpo sin precio y devuelve el stock resultante', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(detalle, 201));
		vi.stubGlobal('fetch', fetchMock);

		const r = await crearVenta({
			deposito_id: 1,
			medio_pago: 'EFECTIVO',
			descuento: 1000,
			lineas: [{ producto_id: 12, cantidad: 3 }]
		});

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/ventas');
		expect(init.method).toBe('POST');
		const cuerpo = JSON.parse(String(init.body));
		expect(cuerpo.lineas[0]).toEqual({ producto_id: 12, cantidad: 3 });
		expect(cuerpo.cliente_id).toBeNull();
		expect(r.lineas[0].stock_resultante).toBe(7);
	});

	it('crearVenta rechaza localmente una cantidad no positiva', async () => {
		const fetchMock = vi.fn();
		vi.stubGlobal('fetch', fetchMock);
		await expect(
			crearVenta({ deposito_id: 1, medio_pago: 'EFECTIVO', lineas: [{ producto_id: 1, cantidad: 0 }] })
		).rejects.toThrow();
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('listarVentas arma los filtros', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [venta], total: 1, pagina: 1, por_pagina: 20 }));
		vi.stubGlobal('fetch', fetchMock);

		await listarVentas({ depositoId: 1, clienteId: 7, estado: 'ANULADA', desde: '2026-11-01', hasta: '2026-11-07' });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		const q = new URL(url, 'http://x').searchParams;
		expect(q.get('deposito_id')).toBe('1');
		expect(q.get('cliente_id')).toBe('7');
		expect(q.get('estado')).toBe('ANULADA');
		expect(q.get('desde')).toBe('2026-11-01');
		expect(q.get('hasta')).toBe('2026-11-07');
	});

	it('obtenerVenta y anularVenta usan sus rutas', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(detalle));
		vi.stubGlobal('fetch', fetchMock);

		await obtenerVenta(31);
		await anularVenta(31);

		const [urlDetalle] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		const [urlAnular, initAnular] = fetchMock.mock.calls[1] as unknown as [string, RequestInit];
		expect(urlDetalle).toContain('/api/ventas/31');
		expect(urlAnular).toContain('/api/ventas/31/anular');
		expect(initAnular.method).toBe('POST');
	});

	it('propaga el 409 stock_insuficiente con su mensaje', async () => {
		const mensaje = 'Stock insuficiente en Principal: 2 disponibles, 3 solicitadas. Reducí la cantidad o registrá antes una entrada en ese depósito.';
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => jsonResponse({ error: { codigo: 'stock_insuficiente', mensaje } }, 409))
		);
		const error = await crearVenta({
			deposito_id: 1,
			medio_pago: 'EFECTIVO',
			lineas: [{ producto_id: 12, cantidad: 3 }]
		}).catch((e: unknown) => e);
		expect(error).toBeInstanceOf(ApiError);
		expect((error as ApiError).codigo).toBe('stock_insuficiente');
		expect((error as ApiError).message).toBe(mensaje);
	});
});
