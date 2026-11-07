import { describe, expect, it } from 'vitest';
import {
	CrearMovimientoSchema,
	ListaMovimientosSchema,
	ListaStockSchema,
	MovimientoDetalleSchema,
	StockProductoSchema,
	MOTIVOS_POR_TIPO,
	TipoMovimientoSchema,
	motivoPorDefecto
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
	stock_deposito: null,
	faltante: 0,
	costo: null,
	valorizado: null
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
			faltante: 0,
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

describe('StockItemSchema con faltante y valorizado (010)', () => {
	it('acepta costo y valorizado numéricos (GESTION) o null (OPERACION)', () => {
		const lista = (item: object) =>
			ListaStockSchema.safeParse({ items: [item], total: 1, pagina: 1, por_pagina: 20 });
		expect(lista({ ...stockItem, faltante: 7, costo: 150.5, valorizado: 451.5 }).success).toBe(true);
		expect(lista({ ...stockItem, faltante: 7, costo: null, valorizado: null }).success).toBe(true);
	});

	it('exige faltante entero en el item y en el detalle', () => {
		const { faltante: _f, ...sinFaltante } = stockItem;
		expect(
			ListaStockSchema.safeParse({ items: [sinFaltante], total: 1, pagina: 1, por_pagina: 20 })
				.success
		).toBe(false);
		const detalle = {
			producto_id: 10,
			nombre: 'Yerba',
			sku: 'Y',
			unidad_medida: 'UNIDAD',
			activo: true,
			stock_actual: 3,
			stock_minimo: 5,
			bajo_minimo: true,
			desglose: []
		};
		expect(StockProductoSchema.safeParse(detalle).success).toBe(false);
		expect(StockProductoSchema.safeParse({ ...detalle, faltante: 2 }).success).toBe(true);
	});
});

describe('MOTIVOS_POR_TIPO', () => {
	it('sigue la matriz del alta manual y excluye los motivos reservados', () => {
		expect(MOTIVOS_POR_TIPO.ENTRADA).toEqual(['COMPRA', 'DEVOLUCION', 'OTRO']);
		expect(MOTIVOS_POR_TIPO.SALIDA).toEqual(['VENTA', 'DEVOLUCION', 'OTRO']);
		expect(MOTIVOS_POR_TIPO.MERMA).toEqual(['MERMA', 'OTRO']);
		expect(MOTIVOS_POR_TIPO.AJUSTE).toEqual(['AJUSTE', 'OTRO']);
		const todos = Object.values(MOTIVOS_POR_TIPO).flat();
		for (const reservado of ['TRANSFERENCIA', 'INVENTARIO', 'IMPORTACION']) {
			expect(todos).not.toContain(reservado);
		}
	});

	it('elige un motivo por defecto coherente con el tipo', () => {
		expect(motivoPorDefecto('ENTRADA')).toBe('COMPRA');
		expect(motivoPorDefecto('SALIDA')).toBe('VENTA');
		expect(motivoPorDefecto('MERMA')).toBe('MERMA');
		expect(motivoPorDefecto('AJUSTE')).toBe('AJUSTE');
	});
});
