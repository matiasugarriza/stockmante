import { describe, expect, it } from 'vitest';
import { elegirDeposito, indexarPorProducto, textoValorizado } from './stock';

describe('utilidades de stock (010)', () => {
	it('indexarPorProducto arma el mapa producto → stock en el depósito', () => {
		const mapa = indexarPorProducto([
			{ producto_id: 1, stock_deposito: 4 },
			{ producto_id: 2, stock_deposito: null },
			{ producto_id: 3, stock_deposito: 0 }
		]);
		expect(mapa.get(1)).toBe(4);
		expect(mapa.get(2)).toBe(0);
		expect(mapa.get(3)).toBe(0);
		expect(mapa.has(9)).toBe(false);
	});

	it('elegirDeposito conserva el guardado si sigue activo', () => {
		const activos = [{ id: 5 }, { id: 7 }];
		expect(elegirDeposito(7, activos)).toBe(7);
	});

	it('elegirDeposito descarta un id inactivo o ausente y toma el primero', () => {
		const activos = [{ id: 5 }, { id: 7 }];
		expect(elegirDeposito(99, activos)).toBe(5);
		expect(elegirDeposito(null, activos)).toBe(5);
		expect(elegirDeposito(5, [])).toBeNull();
	});

	it('textoValorizado muestra "—" sin valorizado y formatea el importe', () => {
		expect(textoValorizado({ valorizado: null })).toBe('—');
		expect(textoValorizado({ valorizado: 451.5 })).toBe('$ 451,50');
		expect(textoValorizado({ valorizado: 1234567.8 })).toBe('$ 1.234.567,80');
	});
});
