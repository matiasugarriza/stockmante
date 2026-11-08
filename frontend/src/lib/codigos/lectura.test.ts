import { afterEach, describe, expect, it, vi } from 'vitest';
import { leerProducto } from './lectura';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const producto = {
	id: 9,
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: '7791234567898',
	categoria: { id: 3, nombre: 'Almacén' },
	unidad_medida: 'UNIDAD',
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6,
	activo: true
};

describe('leerProducto (020)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('encontrado devuelve el resultado sin aviso', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => jsonResponse({ items: [producto] })));

		const { resultado, aviso } = await leerProducto('7791234567898', false);

		expect(resultado).toMatchObject({ estado: 'encontrado', producto: { id: 9 } });
		expect(aviso).toBeNull();
	});

	it('no encontrado devuelve aviso de advertencia con el valor', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => jsonResponse({ items: [] })));

		const { resultado, aviso } = await leerProducto('00000000', false);

		expect(resultado).toEqual({ estado: 'no_encontrado' });
		expect(aviso).toEqual({
			variante: 'advertencia',
			valor: '00000000',
			texto: 'No se encontró un producto activo con el código 00000000. Buscalo por nombre o SKU.'
		});
	});

	it('inactivo (gestión) avisa la baja', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => jsonResponse({ items: [{ ...producto, activo: false }] }))
		);

		const { aviso } = await leerProducto('7791234567898', true);

		expect(aviso?.texto).toContain('dado de baja');
	});

	it('error de red: sin resultado, aviso de error que conserva el valor (FR-155)', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new TypeError('Failed to fetch');
			})
		);

		const { resultado, aviso } = await leerProducto('7791234567898', false);

		expect(resultado).toBeNull();
		expect(aviso?.variante).toBe('error');
		expect(aviso?.valor).toBe('7791234567898');
		expect(aviso?.texto).toMatch(/^No se pudo buscar el código 7791234567898: .+\. Tocá Reintentar\.$/);
	});
});
