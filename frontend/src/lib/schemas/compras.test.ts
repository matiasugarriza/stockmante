import { describe, expect, it } from 'vitest';
import {
	CompraDetalleSchema,
	CompraSchema,
	CrearCompraSchema,
	ListaComprasSchema
} from './compras';

const compra = {
	id: 12,
	proveedor_id: 3,
	proveedor_nombre: 'Distribuidora Norte',
	deposito_id: 1,
	deposito_nombre: 'Depósito central',
	numero: 'OC-001',
	estado: 'BORRADOR',
	observacion: null,
	total_costo: 76.5,
	usuario_id: 2,
	usuario_nombre: 'Gerente',
	creado_en: '2026-11-06T12:00:00Z',
	confirmado_en: null,
	anulado_en: null
};

describe('schemas de compras', () => {
	it('parsea la cabecera con total numérico', () => {
		const parseada = CompraSchema.parse(compra);
		expect(parseada.estado).toBe('BORRADOR');
		expect(parseada.total_costo).toBe(76.5);
	});

	it('rechaza un total serializado como string (contrato serde-float)', () => {
		expect(() => CompraSchema.parse({ ...compra, total_costo: '76.50' })).toThrow();
	});

	it('parsea el detalle con líneas', () => {
		const detalle = CompraDetalleSchema.parse({
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
		});
		expect(detalle.lineas[0]?.subtotal).toBe(76.5);
	});

	it('parsea la lista paginada', () => {
		const lista = ListaComprasSchema.parse({
			items: [compra],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(lista.items[0]?.numero).toBe('OC-001');
	});

	it('exige al menos una línea y cantidad positiva', () => {
		const base = { proveedor_id: 3, deposito_id: 1 };

		expect(() => CrearCompraSchema.parse({ ...base, lineas: [] })).toThrow(/al menos una línea/);
		expect(() =>
			CrearCompraSchema.parse({
				...base,
				lineas: [{ producto_id: 9, cantidad: 0, costo_unitario: 10 }]
			})
		).toThrow(/mayor a cero/);
		expect(() =>
			CrearCompraSchema.parse({
				...base,
				lineas: [{ producto_id: 9, cantidad: 2, costo_unitario: -1 }]
			})
		).toThrow(/negativo/);
	});

	it('acepta una compra válida y recorta número/observación', () => {
		const datos = CrearCompraSchema.parse({
			proveedor_id: 3,
			deposito_id: 1,
			numero: '  OC-001  ',
			observacion: '  Recepción parcial  ',
			lineas: [{ producto_id: 9, cantidad: 2, costo_unitario: 10.25 }]
		});
		expect(datos.numero).toBe('OC-001');
		expect(datos.observacion).toBe('Recepción parcial');
	});
});