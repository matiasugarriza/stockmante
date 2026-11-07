import { describe, expect, it } from 'vitest';
import type { Deposito, Ubicacion } from '../schemas/depositos';
import { nombreDeposito, textoUbicacion, textoUbicaciones } from './ubicaciones';

const deposito: Deposito = {
	id: 4,
	nombre: 'Depósito Central',
	identificador: 'DEP-CENTRAL',
	direccion: null,
	activo: true
};

function ubicacion(overrides: Partial<Ubicacion> = {}): Ubicacion {
	return {
		id: 10,
		deposito_id: 4,
		padre_id: null,
		nombre: 'Estantería A',
		tipo: 'ESTANTERIA',
		activo: true,
		...overrides
	};
}

describe('formato de ubicaciones', () => {
	it('nombreDeposito resuelve el nombre o cae a un identificador estable', () => {
		expect(nombreDeposito([deposito], 4)).toBe('Depósito Central');
		expect(nombreDeposito([], 9)).toBe('Depósito #9');
	});

	it('textoUbicacion marca las inactivas', () => {
		expect(textoUbicacion(ubicacion(), [deposito])).toBe('Depósito Central · Estantería A');
		expect(textoUbicacion(ubicacion({ activo: false }), [deposito])).toBe(
			'Depósito Central · Estantería A (inactiva)'
		);
	});

	it('textoUbicaciones ordena, acota y resume el resto', () => {
		const tres = [
			ubicacion({ id: 12, nombre: 'Nivel 2' }),
			ubicacion({ id: 10, nombre: 'Estantería A' }),
			ubicacion({ id: 11, nombre: 'Nivel 1' })
		];

		expect(textoUbicaciones([], [deposito])).toBe('');
		expect(textoUbicaciones([tres[1]], [deposito])).toBe('Depósito Central · Estantería A');
		expect(textoUbicaciones(tres, [deposito])).toBe(
			'Depósito Central · Estantería A; Depósito Central · Nivel 1 +1'
		);
		expect(textoUbicaciones(tres, [deposito], 1)).toBe('Depósito Central · Estantería A +2');
	});
});
