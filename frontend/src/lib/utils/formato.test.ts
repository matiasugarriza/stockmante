import { describe, expect, it } from 'vitest';
import {
	formatearFechaCorta,
	formatearMoneda,
	formatearNumero,
	formatearPorcentaje,
	truncar
} from './formato';

describe('formatearMoneda', () => {
	it('formatea en pesos argentinos con dos decimales', () => {
		const texto = formatearMoneda(1234.5);
		expect(texto).toContain('1.234,50');
		expect(texto).toMatch(/\$/);
	});

	it('devuelve un guion para valores no finitos', () => {
		expect(formatearMoneda(Number.NaN)).toBe('—');
		expect(formatearMoneda(Number.POSITIVE_INFINITY)).toBe('—');
	});
});

describe('formatearNumero', () => {
	it('separa miles y redondea', () => {
		expect(formatearNumero(12345.6)).toBe('12.346');
	});

	it('devuelve un guion para valores no finitos', () => {
		expect(formatearNumero(Number.NaN)).toBe('—');
	});
});

describe('formatearPorcentaje', () => {
	it('usa coma decimal y agrega el símbolo', () => {
		expect(formatearPorcentaje(12.5, 1)).toBe('12,5 %');
		expect(formatearPorcentaje(50)).toBe('50 %');
	});

	it('devuelve un guion para valores no finitos', () => {
		expect(formatearPorcentaje(Number.NaN)).toBe('—');
	});
});

describe('formatearFechaCorta', () => {
	it('formatea una fecha ISO como dd/mm/aaaa', () => {
		const texto = formatearFechaCorta('2026-11-07T12:00:00Z');
		expect(texto).toMatch(/^\d{2}\/\d{2}\/\d{4}$/);
		expect(texto).toContain('2026');
	});

	it('devuelve un guion para nulos o fechas inválidas', () => {
		expect(formatearFechaCorta(null)).toBe('—');
		expect(formatearFechaCorta(undefined)).toBe('—');
		expect(formatearFechaCorta('no-es-fecha')).toBe('—');
	});
});

describe('truncar', () => {
	it('no toca textos que entran', () => {
		expect(truncar('Coso', 10)).toBe('Coso');
	});

	it('recorta e incluye elipsis cuando excede', () => {
		expect(truncar('Yerba Suave 1 kg', 8)).toBe('Yerba S…');
		expect(truncar('abc', 1)).toBe('…');
	});

	it('devuelve vacío con ancho no positivo', () => {
		expect(truncar('abc', 0)).toBe('');
	});
});
