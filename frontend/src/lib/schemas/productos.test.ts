import { describe, expect, it } from 'vitest';
import {
	ActualizarProductoSchema,
	CrearProductoSchema,
	ListaBusquedaSchema,
	ListaProductosSchema,
	ProductoSchema
} from './productos';

const producto = {
	id: 9,
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: null,
	categoria: { id: 3, nombre: 'Almacén' },
	unidad_medida: 'UNIDAD',
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6,
	activo: true
};

const alta = {
	nombre: 'Yerba 1 kg',
	sku: 'yerba-1k',
	codigo_barras: '',
	categoria_id: 3,
	unidad_medida: 'UNIDAD',
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6
};

describe('ProductoSchema', () => {
	it('parsea el item de GET /productos', () => {
		const r = ProductoSchema.safeParse(producto);
		expect(r.success).toBe(true);
		if (r.success) expect(r.data.categoria.nombre).toBe('Almacén');
	});

	it('rechaza una unidad fuera del catálogo', () => {
		expect(ProductoSchema.safeParse({ ...producto, unidad_medida: 'LITRO' }).success).toBe(
			false
		);
	});

	it('acepta código de barras nulo', () => {
		expect(ProductoSchema.safeParse({ ...producto, codigo_barras: null }).success).toBe(true);
	});
});

describe('ListaProductosSchema', () => {
	it('parsea la página completa', () => {
		const r = ListaProductosSchema.safeParse({
			items: [producto],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(r.success).toBe(true);
	});
});

describe('ListaBusquedaSchema', () => {
	it('parsea el resultado de búsqueda sin paginación', () => {
		const r = ListaBusquedaSchema.safeParse({ items: [producto] });
		expect(r.success).toBe(true);
	});

	it('acepta una lista vacía', () => {
		expect(ListaBusquedaSchema.safeParse({ items: [] }).success).toBe(true);
	});

	it('rechaza items con unidad fuera del catálogo', () => {
		expect(
			ListaBusquedaSchema.safeParse({
				items: [{ ...producto, unidad_medida: 'LITRO' }]
			}).success
		).toBe(false);
	});
});

describe('CrearProductoSchema', () => {
	it('normaliza el SKU a mayúsculas y recorta el nombre', () => {
		const r = CrearProductoSchema.safeParse({ ...alta, nombre: '  Yerba 1 kg  ' });
		expect(r.success).toBe(true);
		if (r.success) {
			expect(r.data.sku).toBe('YERBA-1K');
			expect(r.data.nombre).toBe('Yerba 1 kg');
		}
	});

	it('acepta código de barras vacío, nulo o ausente', () => {
		for (const codigo_barras of ['', null, undefined]) {
			expect(CrearProductoSchema.safeParse({ ...alta, codigo_barras }).success).toBe(true);
		}
	});

	it('acepta un código alfanumérico de 8 a 64', () => {
		expect(
			CrearProductoSchema.safeParse({ ...alta, codigo_barras: '7791234567890' }).success
		).toBe(true);
		expect(
			CrearProductoSchema.safeParse({ ...alta, codigo_barras: 'A1'.repeat(32) }).success
		).toBe(true);
	});

	it.each([
		['SKU de 2', { sku: 'AB' }],
		['SKU con guion inicial', { sku: '-ABC' }],
		['SKU con símbolo', { sku: 'BAD!' }],
		['SKU de 31', { sku: 'A'.repeat(31) }],
		['código de 7', { codigo_barras: '1234567' }],
		['código de 65', { codigo_barras: '9'.repeat(65) }],
		['código con guion', { codigo_barras: '7791-2345' }],
		['unidad inválida', { unidad_medida: 'LITRO' }],
		['nombre vacío', { nombre: '   ' }],
		['nombre de 161', { nombre: 'a'.repeat(161) }],
		['costo negativo', { costo: -1 }],
		['tres decimales', { costo: 10.999 }],
		['tope de monto', { precio_venta: 10_000_000_000 }],
		['stock negativo', { stock_minimo: -3 }],
		['stock decimal', { stock_minimo: 2.5 }],
		['categoría cero', { categoria_id: 0 }]
	])('rechaza %s', (_caso, cambio) => {
		expect(CrearProductoSchema.safeParse({ ...alta, ...cambio }).success).toBe(false);
	});
});

describe('ActualizarProductoSchema', () => {
	it('acepta cambios parciales', () => {
		expect(ActualizarProductoSchema.safeParse({ precio_venta: 2000.75 }).success).toBe(true);
	});

	it('rechaza un cuerpo vacío', () => {
		expect(ActualizarProductoSchema.safeParse({}).success).toBe(false);
	});

	it('rechaza `activo`: el estado se cambia por acciones', () => {
		expect(ActualizarProductoSchema.safeParse({ activo: false }).success).toBe(false);
	});
});
