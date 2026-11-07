import { describe, expect, it } from 'vitest';
import { cumpleMinimo, luminanciaRelativa, parsearHex, ratioContraste } from './contraste';

describe('parsearHex', () => {
	it('acepta #rrggbb y #rgb', () => {
		expect(parsearHex('#ffffff')).toEqual({ r: 255, g: 255, b: 255 });
		expect(parsearHex('#000')).toEqual({ r: 0, g: 0, b: 0 });
		expect(parsearHex('2de2e6')).toEqual({ r: 45, g: 226, b: 230 });
	});

	it('rechaza valores inválidos', () => {
		expect(parsearHex('')).toBeNull();
		expect(parsearHex('#12')).toBeNull();
		expect(parsearHex('#gggggg')).toBeNull();
	});
});

describe('luminanciaRelativa', () => {
	it('blanco = 1 y negro = 0', () => {
		expect(luminanciaRelativa('#ffffff')).toBeCloseTo(1, 5);
		expect(luminanciaRelativa('#000000')).toBeCloseTo(0, 5);
	});

	it('calcula la luminancia del cian de la paleta', () => {
		// #2de2e6 sobre #0d1117 es el par de navegación aprobado.
		expect(luminanciaRelativa('#2de2e6')).toBeGreaterThan(0.5);
	});
});

describe('ratioContraste', () => {
	it('negro sobre blanco da 21:1', () => {
		expect(ratioContraste('#000000', '#ffffff')).toBeCloseTo(21, 1);
	});

	it('el mismo color da 1:1', () => {
		expect(ratioContraste('#0d1117', '#0d1117')).toBeCloseTo(1, 5);
	});

	it('el gris #777777 sobre blanco ronda 4,48:1', () => {
		expect(ratioContraste('#777777', '#ffffff')).toBeCloseTo(4.48, 1);
	});

	it('el orden de los colores no altera el ratio', () => {
		expect(ratioContraste('#0d1117', '#2de2e6')).toBeCloseTo(
			ratioContraste('#2de2e6', '#0d1117'),
			5
		);
	});
});

describe('cumpleMinimo', () => {
	it('compara contra el mínimo WCAG', () => {
		expect(cumpleMinimo(7.38, 7)).toBe(true);
		expect(cumpleMinimo(6.99, 7)).toBe(false);
		expect(cumpleMinimo(3.19, 3)).toBe(true);
	});
});

/* ---------------------------------------------------------------------------
 * Guard de la paleta móvil (SC-042 y SC-049). Lee los tokens reales de
 * `diseno/movil.css`; si un valor cambia por debajo de su mínimo, la suite falla.
 * ------------------------------------------------------------------------- */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const movilCss = readFileSync(
	fileURLToPath(new URL('../diseno/movil.css', import.meta.url)),
	'utf8'
);

function bloqueDesde(css: string, marcador: string, desde = 0): string {
	const posicion = css.indexOf(marcador, desde);
	if (posicion < 0) throw new Error(`No se encontró el marcador: ${marcador}`);
	const inicio = css.indexOf('{', posicion);
	let profundidad = 0;
	for (let indice = inicio; indice < css.length; indice += 1) {
		if (css[indice] === '{') profundidad += 1;
		else if (css[indice] === '}') {
			profundidad -= 1;
			if (profundidad === 0) return css.slice(inicio + 1, indice);
		}
	}
	throw new Error(`Bloque sin cierre: ${marcador}`);
}

function tokensDe(bloque: string): Map<string, string> {
	const tokens = new Map<string, string>();
	const patron = /(--[a-z0-9-]+)\s*:\s*(#[0-9a-fA-F]{3,8})\s*;/g;
	for (const coincidencia of bloque.matchAll(patron)) {
		tokens.set(coincidencia[1] as string, coincidencia[2] as string);
	}
	return tokens;
}

const base = tokensDe(bloqueDesde(movilCss, "html[data-tema='dark']"));
const contraste = tokensDe(
	bloqueDesde(
		movilCss,
		"html[data-tema='dark']",
		movilCss.indexOf('prefers-contrast: more')
	)
);
const altaLegibilidad = new Map([...base, ...contraste]);

interface Par {
	fg: string;
	bg: string;
	minimo: number;
}

const MATRIZ: Par[] = [
	{ fg: '--texto', bg: '--fondo', minimo: 7 },
	{ fg: '--texto', bg: '--superficie', minimo: 7 },
	{ fg: '--texto', bg: '--superficie-2', minimo: 7 },
	{ fg: '--texto-suave', bg: '--superficie', minimo: 7 },
	{ fg: '--texto-suave', bg: '--superficie-2', minimo: 7 },
	{ fg: '--texto-suave', bg: '--superficie-3', minimo: 7 },
	{ fg: '--texto-tenue', bg: '--fondo', minimo: 7 },
	{ fg: '--texto-tenue', bg: '--superficie', minimo: 7 },
	{ fg: '--texto-tenue', bg: '--superficie-2', minimo: 7 },
	{ fg: '--primario-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--primario-texto', bg: '--fondo', minimo: 7 },
	{ fg: '--primario-texto', bg: '--primario-suave', minimo: 7 },
	{ fg: '--texto-suave', bg: '--primario-suave', minimo: 7 },
	{ fg: '--primario', bg: '--superficie', minimo: 3 },
	{ fg: '--primario-contraste', bg: '--primario', minimo: 7 },
	{ fg: '--violeta-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--violeta-texto', bg: '--superficie-2', minimo: 7 },
	{ fg: '--acento-secundario', bg: '--superficie', minimo: 3 },
	{ fg: '--acento-secundario', bg: '--superficie-2', minimo: 3 },
	{ fg: '--exito-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--exito-texto', bg: '--exito-suave', minimo: 7 },
	{ fg: '--ambar-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--ambar-texto', bg: '--ambar-suave', minimo: 7 },
	{ fg: '--peligro-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--peligro-texto', bg: '--peligro-suave', minimo: 7 },
	{ fg: '--peligro', bg: '--superficie', minimo: 3 },
	{ fg: '--borde-control', bg: '--superficie', minimo: 3 },
	{ fg: '--foco', bg: '--superficie', minimo: 3 }
];

const MATRIZ_CONTRASTE: Par[] = [
	{ fg: '--texto', bg: '--superficie', minimo: 7 },
	{ fg: '--texto-suave', bg: '--superficie', minimo: 7 },
	{ fg: '--texto-tenue', bg: '--superficie', minimo: 7 },
	{ fg: '--texto-tenue', bg: '--superficie-3', minimo: 7 },
	{ fg: '--borde', bg: '--superficie', minimo: 3 },
	{ fg: '--borde-control', bg: '--superficie', minimo: 3 },
	{ fg: '--foco', bg: '--superficie', minimo: 3 },
	{ fg: '--primario-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--violeta-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--ambar-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--peligro-texto', bg: '--superficie', minimo: 7 },
	{ fg: '--exito-texto', bg: '--superficie', minimo: 7 }
];

function verificar(tokens: Map<string, string>, matriz: Par[], etiqueta: string): void {
	for (const par of matriz) {
		const fg = tokens.get(par.fg);
		const bg = tokens.get(par.bg);
		expect(fg, `${etiqueta}: falta ${par.fg}`).toBeDefined();
		expect(bg, `${etiqueta}: falta ${par.bg}`).toBeDefined();
		const ratio = ratioContraste(fg as string, bg as string);
		expect(
			ratio,
			`${etiqueta}: ${par.fg} sobre ${par.bg} da ${ratio.toFixed(2)}:1 y necesita ${par.minimo}:1`
		).toBeGreaterThanOrEqual(par.minimo);
	}
}

describe('guard de contraste de la paleta móvil', () => {
	it('todos los pares aprobados alcanzan su mínimo AAA/AA', () => {
		verificar(base, MATRIZ, 'paleta base');
	});

	it('con prefers-contrast: more ningún par baja de su mínimo', () => {
		verificar(altaLegibilidad, MATRIZ_CONTRASTE, 'alto contraste');
	});

	it('los tonos crudos de estado no se usan como color de texto', () => {
		expect(movilCss).not.toMatch(/color:\s*var\(--peligro\)/);
		expect(movilCss).not.toMatch(/color:\s*var\(--acento-secundario\)/);
	});

	it('el guard falla si un valor queda por debajo del mínimo', () => {
		const roto = new Map(base);
		roto.set('--texto-tenue', '#2a3440');
		expect(() => verificar(roto, MATRIZ, 'roto')).toThrow();
	});
});
