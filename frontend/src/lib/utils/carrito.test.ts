import { describe, expect, it } from 'vitest';
import {
	agregarAlCarrito,
	cambiarCantidad,
	errorDescuento,
	lineasParaVenta,
	quitarDelCarrito,
	totalesCarrito,
	type ItemCarrito
} from './carrito';

const yerba = { producto_id: 1, nombre: 'Yerba', precio: 1500 };
const azucar = { producto_id: 2, nombre: 'Azúcar', precio: 900 };

describe('carrito del POS (013)', () => {
	it('agregar el mismo producto fusiona la línea (13.9)', () => {
		let c: ItemCarrito[] = [];
		c = agregarAlCarrito(c, yerba);
		c = agregarAlCarrito(c, azucar);
		c = agregarAlCarrito(c, yerba, 2);
		expect(c).toEqual([
			{ ...yerba, cantidad: 3 },
			{ ...azucar, cantidad: 1 }
		]);
	});

	it('cambiar la cantidad a 0 o menos quita la línea; quitar elimina por producto', () => {
		let c = agregarAlCarrito([], yerba, 2);
		c = agregarAlCarrito(c, azucar);
		expect(cambiarCantidad(c, 1, 5)[0].cantidad).toBe(5);
		expect(cambiarCantidad(c, 1, 0).map((i) => i.producto_id)).toEqual([2]);
		expect(quitarDelCarrito(c, 2).map((i) => i.producto_id)).toEqual([1]);
	});

	it('totaliza subtotal, descuento y total', () => {
		const c = agregarAlCarrito(agregarAlCarrito([], yerba, 2), azucar);
		expect(totalesCarrito(c, 400)).toEqual({ subtotal: 3900, descuento: 400, total: 3500 });
		expect(totalesCarrito([], 0)).toEqual({ subtotal: 0, descuento: 0, total: 0 });
	});

	it('valida el descuento contra el subtotal', () => {
		expect(errorDescuento(-1, 100)).toBe('El descuento no puede ser negativo.');
		expect(errorDescuento(101, 100)).toBe('El descuento no puede superar el subtotal.');
		expect(errorDescuento(Number.NaN, 100)).toBe('Ingresá un descuento numérico.');
		expect(errorDescuento(100, 100)).toBeNull();
	});

	it('arma las líneas de la venta sin precio', () => {
		const c = agregarAlCarrito(agregarAlCarrito([], yerba, 2), azucar);
		expect(lineasParaVenta(c)).toEqual([
			{ producto_id: 1, cantidad: 2 },
			{ producto_id: 2, cantidad: 1 }
		]);
	});
});
