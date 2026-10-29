import { describe, expect, it } from 'vitest';
import {
	ActualizarDepositoSchema,
	ActualizarUbicacionSchema,
	CrearDepositoSchema,
	CrearUbicacionSchema,
	DepositoSchema,
	ListaUbicacionesSchema,
	UbicacionSchema
} from './depositos';

const deposito = {
	id: 4,
	nombre: 'Depósito Central',
	identificador: 'DEP-CENTRAL',
	direccion: 'Av. Siempre Viva 742',
	activo: true
};

const ubicacion = {
	id: 10,
	deposito_id: 4,
	padre_id: null,
	nombre: 'Pasillo 1',
	tipo: 'PASILLO',
	activo: true
};

describe('DepositoSchema', () => {
	it('parsea un depósito con dirección', () => {
		const r = DepositoSchema.safeParse(deposito);
		expect(r.success).toBe(true);
	});

	it('acepta dirección nula', () => {
		expect(DepositoSchema.safeParse({ ...deposito, direccion: null }).success).toBe(true);
	});

	it('rechaza un id no entero', () => {
		expect(DepositoSchema.safeParse({ ...deposito, id: 4.5 }).success).toBe(false);
	});
});

describe('UbicacionSchema', () => {
	it('parsea una raíz con padre nulo', () => {
		const r = UbicacionSchema.safeParse(ubicacion);
		expect(r.success).toBe(true);
	});

	it('parsea una hija con padre', () => {
		const r = UbicacionSchema.safeParse({ ...ubicacion, padre_id: 10 });
		expect(r.success).toBe(true);
	});

	it('rechaza un tipo fuera del enum', () => {
		expect(UbicacionSchema.safeParse({ ...ubicacion, tipo: 'DEPOSITO' }).success).toBe(false);
	});
});

describe('ListaUbicacionesSchema', () => {
	it('parsea la lista plana', () => {
		const r = ListaUbicacionesSchema.safeParse({ items: [ubicacion] });
		expect(r.success).toBe(true);
	});

	it('acepta una lista vacía', () => {
		expect(ListaUbicacionesSchema.safeParse({ items: [] }).success).toBe(true);
	});
});

describe('CrearDepositoSchema', () => {
	it('recorta nombre e identificador', () => {
		const r = CrearDepositoSchema.safeParse({
			nombre: '  Central  ',
			identificador: ' DEP-1 '
		});
		expect(r.success).toBe(true);
		if (r.success) {
			expect(r.data.nombre).toBe('Central');
			expect(r.data.identificador).toBe('DEP-1');
		}
	});

	it('rechaza nombre vacío o demasiado largo', () => {
		expect(CrearDepositoSchema.safeParse({ nombre: '   ', identificador: 'X' }).success).toBe(
			false
		);
		expect(
			CrearDepositoSchema.safeParse({ nombre: 'a'.repeat(121), identificador: 'X' }).success
		).toBe(false);
	});
});

describe('ActualizarDepositoSchema', () => {
	it('acepta un único campo', () => {
		expect(ActualizarDepositoSchema.safeParse({ nombre: 'Nuevo' }).success).toBe(true);
	});

	it('acepta limpiar la dirección con null', () => {
		expect(ActualizarDepositoSchema.safeParse({ direccion: null }).success).toBe(true);
	});
});

describe('CrearUbicacionSchema', () => {
	it('exige depósito y un tipo válido', () => {
		const r = CrearUbicacionSchema.safeParse({
			deposito_id: 4,
			nombre: 'Estantería A',
			tipo: 'ESTANTERIA'
		});
		expect(r.success).toBe(true);
		expect(
			CrearUbicacionSchema.safeParse({ deposito_id: 0, nombre: 'A', tipo: 'NIVEL' }).success
		).toBe(false);
		expect(
			CrearUbicacionSchema.safeParse({ deposito_id: 4, nombre: 'A', tipo: 'OTRO' }).success
		).toBe(false);
	});
});

describe('ActualizarUbicacionSchema', () => {
	it('permite mover a la raíz con padre_id null', () => {
		const r = ActualizarUbicacionSchema.safeParse({ padre_id: null });
		expect(r.success).toBe(true);
		if (r.success) expect(r.data.padre_id).toBeNull();
	});
});
