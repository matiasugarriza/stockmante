import { describe, expect, it } from 'vitest';
import {
	CrearMovimientoSchema,
	ListaMovimientosSchema,
	ListaStockSchema,
	MovimientoDetalleSchema,
	StockProductoSchema,
	TipoMovimientoSchema
} from './movimientos';

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
	stock_deposito: null
};

describe('TipoMovimientoSchema', () => {
	it('acepta los cuatro tipos', () => {
		for (const tipo of ['ENTRADA', 'SALIDA', 'MERMA', 'AJUSTE']) {
			expect(TipoMovimientoSchema.safeParse(tipo).success).toBe(true);
		}
	});

	it('rechaza tipos fuera del enum', () => {
		expect(TipoMovimientoSchema.safeParse('TRANSFERENCIA').success).toBe(false);
	});
});

describe('MovimientoDetalleSchema', () => {
	it('parsea un movimiento con sus líneas', () => {
		expect(MovimientoDetalleSchema.safeParse(movimiento).success).toBe(true);
	});

	it('acepta observación nula y ubicación nula', () => {
		const r = MovimientoDetalleSchema.safeParse({
			...movimiento,
			observacion: null,
			lineas: [{ ...movimiento.lineas[0], ubicacion_id: null, ubicacion_nombre: null }]
		});
		expect(r.success).toBe(true);
	});
});

describe('ListaMovimientosSchema', () => {
	it('parsea la página', () => {
		const r = ListaMovimientosSchema.safeParse({
			items: [movimiento],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(r.success).toBe(true);
	});
});

describe('CrearMovimientoSchema', () => {
	const base = {
		tipo: 'ENTRADA',
		deposito_id: 1,
		motivo: 'COMPRA',
		observacion: null,
		lineas: [{ producto_id: 10, ubicacion_id: null, cantidad_delta: 5 }]
	};

	it('parsea un alta válida', () => {
		expect(CrearMovimientoSchema.safeParse(base).success).toBe(true);
	});

	it('exige al menos una línea', () => {
		expect(CrearMovimientoSchema.safeParse({ ...base, lineas: [] }).success).toBe(false);
	});

	it('rechaza delta cero', () => {
		const r = CrearMovimientoSchema.safeParse({
			...base,
			lineas: [{ producto_id: 10, ubicacion_id: null, cantidad_delta: 0 }]
		});
		expect(r.success).toBe(false);
	});

	it('rechaza producto no positivo', () => {
		const r = CrearMovimientoSchema.safeParse({
			...base,
			lineas: [{ producto_id: 0, ubicacion_id: null, cantidad_delta: 1 }]
		});
		expect(r.success).toBe(false);
	});

	it('recorta la observación vacía a null', () => {
		const r = CrearMovimientoSchema.safeParse({ ...base, observacion: '   ' });
		expect(r.success).toBe(true);
		if (r.success) expect(r.data.observacion).toBeNull();
	});

	it('rechaza observación mayor a 500', () => {
		expect(
			CrearMovimientoSchema.safeParse({ ...base, observacion: 'x'.repeat(501) }).success
		).toBe(false);
	});
});

describe('ListaStockSchema y StockProductoSchema', () => {
	it('parsea un item de stock con stock_deposito nulo', () => {
		const r = ListaStockSchema.safeParse({
			items: [stockItem],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(r.success).toBe(true);
	});

	it('parsea el desglose por depósito', () => {
		const r = StockProductoSchema.safeParse({
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
		});
		expect(r.success).toBe(true);
	});
});
