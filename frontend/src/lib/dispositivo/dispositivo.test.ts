import { describe, expect, it } from 'vitest';
import {
	ANCHO_ESCRITORIO,
	ANCHO_TABLET,
	clasificarDispositivo,
	esLayoutMovil,
	estadoDispositivo,
	orientacionDe
} from './dispositivo';

describe('clasificarDispositivo', () => {
	it('clasifica teléfono por debajo del corte de tablet', () => {
		expect(clasificarDispositivo(320)).toBe('telefono');
		expect(clasificarDispositivo(390)).toBe('telefono');
		expect(clasificarDispositivo(ANCHO_TABLET - 1)).toBe('telefono');
	});

	it('clasifica tablet desde 768 hasta 1023', () => {
		expect(clasificarDispositivo(ANCHO_TABLET)).toBe('tablet');
		expect(clasificarDispositivo(844)).toBe('tablet');
		expect(clasificarDispositivo(ANCHO_ESCRITORIO - 1)).toBe('tablet');
	});

	it('clasifica escritorio desde 1024', () => {
		expect(clasificarDispositivo(ANCHO_ESCRITORIO)).toBe('escritorio');
		expect(clasificarDispositivo(1440)).toBe('escritorio');
	});
});

describe('esLayoutMovil', () => {
	it('el layout móvil abarca teléfono y tablet', () => {
		expect(esLayoutMovil(320)).toBe(true);
		expect(esLayoutMovil(1023)).toBe(true);
		expect(esLayoutMovil(1024)).toBe(false);
	});
});

describe('orientacionDe', () => {
	it('detecta vertical y horizontal', () => {
		expect(orientacionDe(390, 844)).toBe('vertical');
		expect(orientacionDe(844, 390)).toBe('horizontal');
	});

	it('trata el cuadrado como vertical', () => {
		expect(orientacionDe(600, 600)).toBe('vertical');
	});
});

describe('estadoDispositivo', () => {
	it('arma el estado completo del teléfono en vertical', () => {
		expect(estadoDispositivo(390, 844)).toEqual({
			ancho: 390,
			alto: 844,
			layout: 'telefono',
			esMovil: true,
			orientacion: 'vertical'
		});
	});

	it('arma el estado de escritorio', () => {
		expect(estadoDispositivo(1440, 900)).toEqual({
			ancho: 1440,
			alto: 900,
			layout: 'escritorio',
			esMovil: false,
			orientacion: 'horizontal'
		});
	});
});
