import { describe, expect, it } from 'vitest';
import { arcoDona, escalaBarras, porcentaje } from './graficos';

describe('porcentaje', () => {
	it('calcula la proporción sobre el total', () => {
		expect(porcentaje(25, 100)).toBe(25);
		expect(porcentaje(1, 3)).toBeCloseTo(33.333, 3);
	});

	it('devuelve 0 con total no positivo o valores inválidos', () => {
		expect(porcentaje(10, 0)).toBe(0);
		expect(porcentaje(10, -5)).toBe(0);
		expect(porcentaje(Number.NaN, 100)).toBe(0);
	});

	it('recorta al rango 0–100', () => {
		expect(porcentaje(-10, 100)).toBe(0);
		expect(porcentaje(150, 100)).toBe(100);
	});
});

describe('escalaBarras', () => {
	it('escala la barra más alta al alto disponible', () => {
		const { barras, max } = escalaBarras(
			[
				{ etiqueta: 'A', valor: 10 },
				{ etiqueta: 'B', valor: 20 }
			],
			{ ancho: 200, alto: 100, separacion: 0 }
		);

		expect(max).toBe(20);
		expect(barras).toHaveLength(2);
		expect(barras[0]?.alto).toBe(50);
		expect(barras[1]?.alto).toBe(100);
		expect(barras[1]?.y).toBe(0);
		expect(barras[0]?.y).toBe(50);
		expect(barras[0]?.ancho).toBe(100);
	});

	it('normaliza valores negativos a cero', () => {
		const { barras } = escalaBarras([{ etiqueta: 'A', valor: -5 }]);
		expect(barras[0]?.alto).toBe(0);
		expect(barras[0]?.porcentaje).toBe(0);
	});

	it('maneja una serie vacía sin dividir por cero', () => {
		const { barras, max } = escalaBarras([]);
		expect(barras).toHaveLength(0);
		expect(max).toBe(1);
	});
});

describe('arcoDona', () => {
	it('reparte dos mitades en 50 % cada una', () => {
		const segmentos = arcoDona([
			{ etiqueta: 'Bajo mínimo', valor: 1 },
			{ etiqueta: 'Normal', valor: 1 }
		]);

		expect(segmentos).toHaveLength(2);
		expect(segmentos[0]?.porcentaje).toBeCloseTo(50, 5);
		expect(segmentos[1]?.porcentaje).toBeCloseTo(50, 5);
		expect(segmentos[0]?.d.startsWith('M ')).toBe(true);
		expect(segmentos[0]?.fin).toBeGreaterThan(segmentos[0]?.inicio ?? 0);
	});

	it('dibuja un anillo completo cuando hay un solo valor', () => {
		const segmentos = arcoDona([{ etiqueta: 'Todo', valor: 7 }]);
		expect(segmentos).toHaveLength(1);
		expect(segmentos[0]?.porcentaje).toBe(100);
		expect(segmentos[0]?.d.length).toBeGreaterThan(10);
	});

	it('devuelve vacío si el total es cero', () => {
		expect(arcoDona([{ etiqueta: 'A', valor: 0 }])).toEqual([]);
		expect(arcoDona([])).toEqual([]);
	});
});
