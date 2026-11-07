import { describe, expect, it } from 'vitest';
import {
	CLAVE_TEMA,
	aplicarTema,
	esTema,
	guardarPreferencia,
	leerPreferencia,
	resolverTema
} from './tema';

describe('resolverTema', () => {
	it('respeta una elección explícita por encima del sistema', () => {
		expect(resolverTema('light', true)).toBe('light');
		expect(resolverTema('dark', false)).toBe('dark');
	});

	it('en modo system sigue la preferencia del sistema', () => {
		expect(resolverTema('system', true)).toBe('dark');
		expect(resolverTema('system', false)).toBe('light');
	});
});

describe('esTema', () => {
	it('acepta solo los tres valores válidos', () => {
		expect(esTema('light')).toBe(true);
		expect(esTema('dark')).toBe(true);
		expect(esTema('system')).toBe(true);
		expect(esTema('azul')).toBe(false);
		expect(esTema(null)).toBe(false);
	});
});

describe('leerPreferencia', () => {
	it('lee un valor guardado válido', () => {
		expect(leerPreferencia({ getItem: () => 'dark' })).toBe('dark');
	});

	it('cae a system cuando no hay valor o es inválido', () => {
		expect(leerPreferencia({ getItem: () => null })).toBe('system');
		expect(leerPreferencia({ getItem: () => 'azul' })).toBe('system');
	});

	it('cae a system cuando el almacén no está disponible', () => {
		expect(leerPreferencia(null)).toBe('system');
	});

	it('no propaga un fallo del almacén', () => {
		const roto = {
			getItem: () => {
				throw new Error('bloqueado');
			}
		};
		expect(leerPreferencia(roto)).toBe('system');
	});
});

describe('guardarPreferencia', () => {
	it('persiste el valor bajo la clave del tema', () => {
		const guardados: Array<[string, string]> = [];
		guardarPreferencia({ setItem: (clave, valor) => guardados.push([clave, valor]) }, 'dark');
		expect(guardados).toEqual([[CLAVE_TEMA, 'dark']]);
	});

	it('no falla si el almacén no está disponible', () => {
		expect(() => guardarPreferencia(null, 'light')).not.toThrow();
	});

	it('no propaga un fallo del almacén', () => {
		const roto = {
			setItem: () => {
				throw new Error('lleno');
			}
		};
		expect(() => guardarPreferencia(roto, 'system')).not.toThrow();
	});
});

describe('aplicarTema', () => {
	it('fija data-tema y color-scheme en la raíz', () => {
		const atributos: Record<string, string> = {};
		const raiz = {
			setAttribute: (nombre: string, valor: string) => {
				atributos[nombre] = valor;
			},
			style: { colorScheme: '' }
		};

		aplicarTema(raiz, 'dark');

		expect(atributos['data-tema']).toBe('dark');
		expect(raiz.style.colorScheme).toBe('dark');
	});
});

describe('resolverTema en layout móvil', () => {
	it('fuerza dark en móvil aunque la preferencia sea light o system', () => {
		expect(resolverTema('light', false, true)).toBe('dark');
		expect(resolverTema('system', false, true)).toBe('dark');
		expect(resolverTema('light', true, true)).toBe('dark');
	});

	it('respeta la preferencia en escritorio', () => {
		expect(resolverTema('light', false, false)).toBe('light');
		expect(resolverTema('dark', false, false)).toBe('dark');
		expect(resolverTema('system', true, false)).toBe('dark');
	});

	it('sin el tercer argumento se comporta como escritorio', () => {
		expect(resolverTema('light', false)).toBe('light');
	});
});
