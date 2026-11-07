/**
 * Clasificación del dispositivo y layout vigente (spec 019, FR-107/FR-113).
 *
 * La lógica pura se prueba sin DOM; el store se inicializa desde `matchMedia`/`innerWidth`
 * en el layout raíz. Teléfono y tablet comparten el layout móvil (< 1024 px).
 */
import { writable } from 'svelte/store';

export type CategoriaDispositivo = 'telefono' | 'tablet' | 'escritorio';
export type Orientacion = 'vertical' | 'horizontal';

export interface EstadoDispositivo {
	ancho: number;
	alto: number;
	layout: CategoriaDispositivo;
	esMovil: boolean;
	orientacion: Orientacion;
}

export const ANCHO_TABLET = 768;
export const ANCHO_ESCRITORIO = 1024;

export function clasificarDispositivo(ancho: number): CategoriaDispositivo {
	if (ancho >= ANCHO_ESCRITORIO) return 'escritorio';
	if (ancho >= ANCHO_TABLET) return 'tablet';
	return 'telefono';
}

/** El layout móvil abarca teléfono y tablet (hasta 1023 px). */
export function esLayoutMovil(ancho: number): boolean {
	return ancho < ANCHO_ESCRITORIO;
}

/** Un cuadrado se considera vertical. */
export function orientacionDe(ancho: number, alto: number): Orientacion {
	return ancho > alto ? 'horizontal' : 'vertical';
}

export function estadoDispositivo(ancho: number, alto: number): EstadoDispositivo {
	return {
		ancho,
		alto,
		layout: clasificarDispositivo(ancho),
		esMovil: esLayoutMovil(ancho),
		orientacion: orientacionDe(ancho, alto)
	};
}

function estadoInicial(): EstadoDispositivo {
	if (typeof window === 'undefined') return estadoDispositivo(ANCHO_ESCRITORIO, 768);
	return estadoDispositivo(window.innerWidth, window.innerHeight);
}

export const dispositivo = writable<EstadoDispositivo>(estadoInicial());

/** Arranca el seguimiento de tamaño/orientación; devuelve la limpieza para `onMount`. */
export function iniciarDispositivo(): () => void {
	if (typeof window === 'undefined') return () => {};
	const actualizar = () => {
		dispositivo.set(estadoDispositivo(window.innerWidth, window.innerHeight));
	};
	actualizar();
	window.addEventListener('resize', actualizar);
	window.addEventListener('orientationchange', actualizar);
	return () => {
		window.removeEventListener('resize', actualizar);
		window.removeEventListener('orientationchange', actualizar);
	};
}
