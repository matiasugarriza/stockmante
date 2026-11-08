import { describe, expect, it } from 'vitest';
import {
	FORMATOS_LECTURA,
	MENSAJES_CAMARA,
	clasificarErrorCamara,
	elegirLectura,
	esProbableCodigo,
	mensajeLectura,
	normalizarLectura,
	type EntornoCamara
} from './escaner';

const entornoOk: EntornoCamara = { contextoSeguro: true, tieneMediaDevices: true };

function errorDom(nombre: string): Error {
	const error = new Error(nombre);
	error.name = nombre;
	return error;
}

describe('normalizarLectura (FR-137)', () => {
	it('recorta y quita espacios internos', () => {
		expect(normalizarLectura('  779 1234 567898 ')).toBe('7791234567898');
	});

	it('quita el separador GS y otros caracteres de control de los lectores', () => {
		expect(normalizarLectura('\x1d0107791234567898\r\n')).toBe('0107791234567898');
		expect(normalizarLectura('ABC\t123\u0000')).toBe('ABC123');
	});

	it('vacío o solo espacios es null', () => {
		expect(normalizarLectura('')).toBeNull();
		expect(normalizarLectura('   \n')).toBeNull();
	});

	it('conserva mayúsculas y minúsculas: el backend normaliza el SKU', () => {
		expect(normalizarLectura('yerba-1k')).toBe('yerba-1k');
	});
});

describe('esProbableCodigo', () => {
	it('acepta 8–64 alfanuméricos sin espacios', () => {
		expect(esProbableCodigo('7791234567898')).toBe(true);
		expect(esProbableCodigo('INT00001')).toBe(true);
		expect(esProbableCodigo('A'.repeat(64))).toBe(true);
	});

	it('rechaza texto libre, cortos, largos y con símbolos', () => {
		expect(esProbableCodigo('yerba mate')).toBe(false);
		expect(esProbableCodigo('1234567')).toBe(false);
		expect(esProbableCodigo('A'.repeat(65))).toBe(false);
		expect(esProbableCodigo('YERBA-1K')).toBe(false);
	});
});

describe('clasificarErrorCamara (FR-151)', () => {
	it('sin contexto seguro es inseguro, antes que cualquier otro motivo', () => {
		expect(clasificarErrorCamara(null, { contextoSeguro: false, tieneMediaDevices: false })).toBe(
			'inseguro'
		);
	});

	it('sin mediaDevices en contexto seguro es sin_soporte', () => {
		expect(clasificarErrorCamara(null, { contextoSeguro: true, tieneMediaDevices: false })).toBe(
			'sin_soporte'
		);
	});

	it('mapea los errores de getUserMedia', () => {
		expect(clasificarErrorCamara(errorDom('NotAllowedError'), entornoOk)).toBe('permiso_denegado');
		expect(clasificarErrorCamara(errorDom('SecurityError'), entornoOk)).toBe('permiso_denegado');
		expect(clasificarErrorCamara(errorDom('NotFoundError'), entornoOk)).toBe('sin_camara');
		expect(clasificarErrorCamara(errorDom('OverconstrainedError'), entornoOk)).toBe('sin_camara');
		expect(clasificarErrorCamara(errorDom('NotReadableError'), entornoOk)).toBe('ocupada');
		expect(clasificarErrorCamara(errorDom('AbortError'), entornoOk)).toBe('desconocido');
		expect(clasificarErrorCamara('raro', entornoOk)).toBe('desconocido');
	});

	it('cada tipo tiene un mensaje con qué pasó y qué hacer (P-III)', () => {
		for (const mensaje of Object.values(MENSAJES_CAMARA)) {
			expect(mensaje.length).toBeGreaterThan(30);
			expect(mensaje).toMatch(/\. /);
		}
		expect(MENSAJES_CAMARA.inseguro).toContain('HTTPS');
		expect(MENSAJES_CAMARA.permiso_denegado).toContain('permiso');
	});
});

describe('elegirLectura', () => {
	it('sin detecciones devuelve null', () => {
		expect(elegirLectura([])).toBeNull();
	});

	it('toma una sola lectura aunque el código aparezca dos veces', () => {
		const lectura = elegirLectura([
			{ rawValue: '7791234567898', format: 'ean_13', boundingBox: { width: 10, height: 10 } },
			{ rawValue: '7791234567898', format: 'ean_13', boundingBox: { width: 40, height: 20 } }
		]);
		expect(lectura).toEqual({ valor: '7791234567898', formato: 'ean_13' });
	});

	it('si hay códigos distintos se queda con el de mayor área', () => {
		const lectura = elegirLectura([
			{ rawValue: 'CHICO0001', format: 'code_128', boundingBox: { width: 5, height: 5 } },
			{ rawValue: 'https://x.y', format: 'qr_code', boundingBox: { width: 50, height: 50 } }
		]);
		expect(lectura).toEqual({ valor: 'https://x.y', formato: 'qr_code' });
	});

	it('ignora detecciones vacías tras normalizar', () => {
		expect(
			elegirLectura([{ rawValue: '  ', format: 'qr_code', boundingBox: { width: 9, height: 9 } }])
		).toBeNull();
	});
});

describe('FORMATOS_LECTURA (FR-157)', () => {
	it('incluye los 6 formatos de la spec', () => {
		expect([...FORMATOS_LECTURA].sort()).toEqual(
			['code_128', 'code_39', 'ean_13', 'ean_8', 'qr_code', 'upc_a'].sort()
		);
	});
});

describe('mensajeLectura (P-III)', () => {
	const producto = { nombre: 'Yerba 1 kg', sku: 'YERBA-1K' };

	it('no encontrado ofrece la búsqueda manual y, sin inactivos, aclara «activo»', () => {
		expect(mensajeLectura({ estado: 'no_encontrado' }, '00000000', true)).toBe(
			'No se encontró un producto con el código 00000000. Buscalo por nombre o SKU.'
		);
		expect(mensajeLectura({ estado: 'no_encontrado' }, '00000000', false)).toBe(
			'No se encontró un producto activo con el código 00000000. Buscalo por nombre o SKU.'
		);
	});

	it('inactivo informa la baja con nombre y SKU', () => {
		expect(mensajeLectura({ estado: 'inactivo', producto }, 'X', true)).toBe(
			'El producto Yerba 1 kg (SKU YERBA-1K) está dado de baja; no se puede agregar.'
		);
	});

	it('encontrado no genera aviso', () => {
		expect(mensajeLectura({ estado: 'encontrado', producto }, 'X', true)).toBeNull();
	});
});
