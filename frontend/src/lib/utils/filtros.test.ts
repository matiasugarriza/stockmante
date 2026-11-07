import { describe, expect, it } from 'vitest';
import { contarFiltrosActivos, resumenFiltros } from './filtros';

describe('contarFiltrosActivos', () => {
	it('cuenta solo los filtros con valor', () => {
		expect(
			contarFiltrosActivos([
				{ etiqueta: 'Buscar', valor: 'yerba' },
				{ etiqueta: 'Depósito', valor: '' },
				{ etiqueta: 'Categoría', valor: '3' },
				{ etiqueta: 'Bajo mínimo', valor: '' }
			])
		).toBe(2);
	});

	it('sin filtros devuelve 0', () => {
		expect(contarFiltrosActivos([])).toBe(0);
		expect(contarFiltrosActivos([{ etiqueta: 'Depósito', valor: '' }])).toBe(0);
	});
});

describe('resumenFiltros', () => {
	it('concuerda en singular y plural', () => {
		expect(resumenFiltros(0)).toBe('Filtros');
		expect(resumenFiltros(1)).toBe('1 filtro activo');
		expect(resumenFiltros(3)).toBe('3 filtros activos');
	});
});
