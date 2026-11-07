import { describe, expect, it } from 'vitest';
import {
	CargarLineasSchema,
	ConteoLineaSchema,
	ConteoSchema,
	CrearConteoSchema,
	ListaConteosSchema
} from './conteos';

const linea = {
	id: 1,
	producto_id: 10,
	producto_nombre: 'Yerba 1 kg',
	sku: 'YER-001',
	ubicacion_id: null,
	ubicacion_nombre: null,
	cantidad_sistema: 10,
	cantidad_contada: null,
	diferencia: null
};

const conteo = {
	id: 1,
	deposito_id: 1,
	deposito_nombre: 'Principal',
	estado: 'ABIERTO',
	usuario: { id: 2, nombre: 'Gerente' },
	creado_en: '2026-10-29T12:00:00Z',
	cerrado_en: null,
	cancelado_en: null,
	movimiento_ajuste_id: null,
	observacion: null,
	lineas: [linea]
};

describe('schemas de conteos (009)', () => {
	it('parsea un conteo abierto y una página', () => {
		expect(ConteoSchema.parse(conteo).lineas[0].cantidad_contada).toBeNull();
		const pagina = ListaConteosSchema.parse({ items: [conteo], total: 1, pagina: 1, por_pagina: 20 });
		expect(pagina.items).toHaveLength(1);
	});

	it('acepta cantidad_contada y diferencia nulas o enteras', () => {
		expect(ConteoLineaSchema.safeParse(linea).success).toBe(true);
		expect(
			ConteoLineaSchema.safeParse({ ...linea, cantidad_contada: 8, diferencia: -2 }).success
		).toBe(true);
		expect(ConteoLineaSchema.safeParse({ ...linea, cantidad_contada: 1.5 }).success).toBe(false);
	});

	it('rechaza un estado desconocido', () => {
		expect(ConteoSchema.safeParse({ ...conteo, estado: 'EN_CURSO' }).success).toBe(false);
	});

	it('la observación es opcional: vacía o ausente queda en null', () => {
		expect(CrearConteoSchema.parse({ deposito_id: 1 }).observacion).toBeNull();
		expect(CrearConteoSchema.parse({ deposito_id: 1, observacion: '   ' }).observacion).toBeNull();
		expect(CrearConteoSchema.parse({ deposito_id: 1, observacion: ' mes ' }).observacion).toBe('mes');
		expect(CrearConteoSchema.safeParse({ deposito_id: 1, observacion: 'x'.repeat(501) }).success).toBe(
			false
		);
		expect(CrearConteoSchema.safeParse({ deposito_id: 0 }).success).toBe(false);
	});

	it('rechaza cantidades contadas negativas', () => {
		const resultado = CargarLineasSchema.safeParse({
			lineas: [{ linea_id: 1, cantidad_contada: -1 }]
		});
		expect(resultado.success).toBe(false);
		expect(
			CargarLineasSchema.safeParse({ lineas: [{ linea_id: 1, cantidad_contada: 0 }] }).success
		).toBe(true);
	});
});
