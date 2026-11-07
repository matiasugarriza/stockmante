import { describe, expect, it } from 'vitest';
import { avisoResultadoCompra } from './compras';

const movimiento = (tipo: 'ENTRADA' | 'SALIDA', lineas: Array<[string, number]>) => ({
	tipo,
	deposito: { id: 1, nombre: 'Depósito Central' },
	lineas: lineas.map(([producto_nombre, stock_resultante]) => ({ producto_nombre, stock_resultante }))
});

describe('avisoResultadoCompra (012, Principio III)', () => {
	it('arma el aviso de confirmación con el stock resultante', () => {
		expect(
			avisoResultadoCompra({ compra: { id: 7 }, movimiento: movimiento('ENTRADA', [['Cemento 50 kg', 24]]) })
		).toBe('Compra #7 confirmada en Depósito Central: Cemento 50 kg → 24 u.');
	});

	it('arma el aviso de anulación y separa varias líneas', () => {
		expect(
			avisoResultadoCompra({
				compra: { id: 8 },
				movimiento: movimiento('SALIDA', [
					['Cemento 50 kg', 14],
					['Arena', 0]
				])
			})
		).toBe('Compra #8 anulada en Depósito Central: Cemento 50 kg → 14 u., Arena → 0 u.');
	});
});
