import { describe, expect, it } from 'vitest';
import { indiceAnterior, indiceSiguiente, normalizarIndice } from './carrusel';

describe('normalizarIndice', () => {
	it('deja el índice dentro del rango con vuelta', () => {
		expect(normalizarIndice(0, 3)).toBe(0);
		expect(normalizarIndice(3, 3)).toBe(0);
		expect(normalizarIndice(5, 3)).toBe(2);
		expect(normalizarIndice(-1, 3)).toBe(2);
	});

	it('con cero tarjetas devuelve 0', () => {
		expect(normalizarIndice(2, 0)).toBe(0);
	});
});

describe('indiceSiguiente', () => {
	it('avanza y vuelve al inicio', () => {
		expect(indiceSiguiente(0, 3)).toBe(1);
		expect(indiceSiguiente(1, 3)).toBe(2);
		expect(indiceSiguiente(2, 3)).toBe(0);
	});

	it('sin tarjetas se queda en 0', () => {
		expect(indiceSiguiente(0, 0)).toBe(0);
	});
});

describe('indiceAnterior', () => {
	it('retrocede y vuelve al final', () => {
		expect(indiceAnterior(2, 3)).toBe(1);
		expect(indiceAnterior(0, 3)).toBe(2);
	});

	it('sin tarjetas se queda en 0', () => {
		expect(indiceAnterior(0, 0)).toBe(0);
	});
});
