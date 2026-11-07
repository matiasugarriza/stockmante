import { afterEach, describe, expect, it, vi } from 'vitest';
import { buscarInventario } from './inventario';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const item = {
	producto_id: 9,
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	categoria_id: 3,
	categoria_nombre: 'Almacén',
	unidad_medida: 'UNIDAD',
	activo: true,
	stock_actual: 7,
	stock_minimo: 6,
	bajo_minimo: false,
	stock_deposito: 7,
	faltante: 0,
	costo: null,
	valorizado: null
};

const producto = {
	id: 9,
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: '7791234567890',
	categoria: { id: 3, nombre: 'Almacén' },
	unidad_medida: 'UNIDAD',
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6,
	activo: true
};

const paginaVacia = { items: [], total: 0, pagina: 1, por_pagina: 20 };
const paginaConItem = { items: [item], total: 1, pagina: 1, por_pagina: 20 };

describe('buscarInventario', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('busca por nombre o SKU con una sola llamada a /api/stock', async () => {
		const fetchMock = vi.fn(async (_url: string) => jsonResponse(paginaConItem));
		vi.stubGlobal('fetch', fetchMock);

		const items = await buscarInventario('  yerba  ');

		expect(items).toHaveLength(1);
		expect(String(fetchMock.mock.calls[0]?.[0])).toContain('/api/stock?q=yerba');
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('no consulta con menos de dos caracteres', async () => {
		const fetchMock = vi.fn();
		vi.stubGlobal('fetch', fetchMock);

		await expect(buscarInventario(' y ')).resolves.toEqual([]);
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('si no hay resultados y parece código, resuelve por código de barras', async () => {
		const respuestas = [
			jsonResponse(paginaVacia),
			jsonResponse({ items: [producto] }),
			jsonResponse(paginaConItem)
		];
		const fetchMock = vi.fn(async (_url: string) => respuestas.shift() as Response);
		vi.stubGlobal('fetch', fetchMock);

		const items = await buscarInventario('7791234567890');

		expect(items).toHaveLength(1);
		expect(fetchMock).toHaveBeenCalledTimes(3);
		expect(String(fetchMock.mock.calls[1]?.[0])).toContain('/api/productos/buscar?');
		expect(String(fetchMock.mock.calls[1]?.[0])).toContain('campo=codigo');
		expect(String(fetchMock.mock.calls[2]?.[0])).toContain('producto_ids=9');
	});

	it('no busca por código si el texto no tiene forma de código', async () => {
		const fetchMock = vi.fn(async (_url: string) => jsonResponse(paginaVacia));
		vi.stubGlobal('fetch', fetchMock);

		await expect(buscarInventario('abcd')).resolves.toEqual([]);
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('propaga el error de la API', async () => {
		const fetchMock = vi.fn(async (_url: string) =>
			jsonResponse({ error: { codigo: 'interno', mensaje: 'Falló el servidor' } }, 500)
		);
		vi.stubGlobal('fetch', fetchMock);

		await expect(buscarInventario('yerba')).rejects.toMatchObject({ status: 500 });
	});
});
