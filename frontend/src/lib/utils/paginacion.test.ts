import { describe, expect, it } from 'vitest';
import { paginaValida, totalPaginas } from './paginacion';

describe('paginación', () => {
	const casos: Array<[number, number]> = [
		[0, 1],
		[1, 1],
		[20, 1],
		[21, 2],
		[40, 2],
		[41, 3],
		[100, 5]
	];

	it.each(casos)('totalPaginas(%i, 20) = %i', (total, esperado) => {
		expect(totalPaginas(total, 20)).toBe(esperado);
	});

	it('totalPaginas nunca baja de 1 aunque el tamaño de página sea inválido', () => {
		expect(totalPaginas(0, 0)).toBe(1);
		expect(totalPaginas(10, -5)).toBe(1);
	});

	it('paginaValida recorta al rango disponible', () => {
		expect(paginaValida(3, 100, 20)).toBe(3);
		expect(paginaValida(8, 100, 20)).toBe(5);
		expect(paginaValida(0, 100, 20)).toBe(1);
		expect(paginaValida(-2, 100, 20)).toBe(1);
		expect(paginaValida(4, 0, 20)).toBe(1);
	});

	it('totalPaginas cubre los bordes de cada página', () => {
		expect(totalPaginas(19, 20)).toBe(1);
		expect(totalPaginas(39, 20)).toBe(2);
		expect(totalPaginas(60, 20)).toBe(3);
		expect(totalPaginas(61, 20)).toBe(4);
		expect(totalPaginas(101, 20)).toBe(6);
	});

	it('entradas no finitas no rompen la cuenta', () => {
		expect(totalPaginas(Number.NaN, 20)).toBe(1);
		expect(totalPaginas(Number.POSITIVE_INFINITY, 20)).toBe(1);
		expect(totalPaginas(10, Number.NaN)).toBe(1);
		expect(paginaValida(Number.POSITIVE_INFINITY, 100, 20)).toBe(1);
	});

	it('recorta a la última página válida tras bajas', () => {
		expect(paginaValida(3, 39, 20)).toBe(2);
		expect(paginaValida(2, 19, 20)).toBe(1);
		expect(paginaValida(5, 99, 20)).toBe(5);
	});
});
