import { describe, expect, it } from 'vitest';
import {
	clasificarStock,
	etiquetaStock,
	validarCantidadMovil,
	varianteStock
} from './inventario';

describe('clasificarStock', () => {
	it('stock en cero o negativo es crítico', () => {
		expect(clasificarStock(0, true)).toBe('critico');
		expect(clasificarStock(-2, false)).toBe('critico');
	});

	it('con stock positivo y bajo el mínimo es bajo', () => {
		expect(clasificarStock(3, true)).toBe('bajo');
		expect(clasificarStock(1, true)).toBe('bajo');
	});

	it('sobre el mínimo es normal', () => {
		expect(clasificarStock(12, false)).toBe('normal');
	});
});

describe('etiquetaStock', () => {
	it('usa etiquetas textuales para no depender del color', () => {
		expect(etiquetaStock('critico')).toBe('Crítico');
		expect(etiquetaStock('bajo')).toBe('Bajo');
		expect(etiquetaStock('normal')).toBe('Normal');
	});
});

describe('varianteStock', () => {
	it('mapea el estado a la variante del badge', () => {
		expect(varianteStock('critico')).toBe('peligro');
		expect(varianteStock('bajo')).toBe('aviso');
		expect(varianteStock('normal')).toBe('exito');
	});
});

describe('validarCantidadMovil', () => {
	it('rechaza vacío, cero, negativo y no numérico', () => {
		expect(
			validarCantidadMovil({ tipo: 'ENTRADA', cantidad: '', disponible: null, esGestion: true })
		).toBe('Ingresá una cantidad.');
		expect(
			validarCantidadMovil({ tipo: 'ENTRADA', cantidad: '0', disponible: null, esGestion: true })
		).toBe('La cantidad debe ser mayor a cero.');
		expect(
			validarCantidadMovil({ tipo: 'ENTRADA', cantidad: '-2', disponible: null, esGestion: true })
		).toBe('La cantidad debe ser mayor a cero.');
		expect(
			validarCantidadMovil({ tipo: 'ENTRADA', cantidad: 'abc', disponible: null, esGestion: true })
		).toBe('La cantidad debe ser un número entero.');
	});

	it('rechaza decimales', () => {
		expect(
			validarCantidadMovil({ tipo: 'ENTRADA', cantidad: '1.5', disponible: null, esGestion: true })
		).toBe('La cantidad debe ser un número entero.');
	});

	it('no permite sacar más de lo disponible', () => {
		expect(
			validarCantidadMovil({ tipo: 'SALIDA', cantidad: '5', disponible: 3, esGestion: true })
		).toBe('No podés mover 5: hay 3 disponibles en el depósito.');
		expect(
			validarCantidadMovil({ tipo: 'SALIDA', cantidad: '3', disponible: 3, esGestion: true })
		).toBeNull();
	});

	it('exige stock conocido para salidas y mermas', () => {
		expect(
			validarCantidadMovil({ tipo: 'MERMA', cantidad: '1', disponible: null, esGestion: true })
		).toBe('Elegí un depósito con stock disponible.');
	});

	it('el ajuste requiere gestión y cantidad distinta de cero', () => {
		expect(
			validarCantidadMovil({ tipo: 'AJUSTE', cantidad: '1', disponible: null, esGestion: false })
		).toBe('El ajuste requiere permisos de gestión.');
		expect(
			validarCantidadMovil({ tipo: 'AJUSTE', cantidad: '1', disponible: null, esGestion: true })
		).toBeNull();
	});

	it('acepta una entrada válida', () => {
		expect(
			validarCantidadMovil({ tipo: 'ENTRADA', cantidad: '4', disponible: null, esGestion: false })
		).toBeNull();
	});
});
