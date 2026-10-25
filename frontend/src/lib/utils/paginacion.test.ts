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
});
