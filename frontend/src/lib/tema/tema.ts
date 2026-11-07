/**
 * Preferencia de tema de la interfaz (spec 018 FR-079/FR-080; spec 019 FR-104).
 *
 * La lógica vive en funciones puras con sus dependencias inyectadas (almacén y raíz del
 * documento) para poder probarla en entorno `node` sin DOM. El script inline de `app.html`
 * aplica el tema antes del primer pintado; este módulo lo mantiene sincronizado al hidratar.
 *
 * En el layout móvil (menos de 1024 px) la identidad oscura cyberpunk es obligatoria: la
 * preferencia claro/oscuro no se aplica y se conserva para cuando el usuario vuelve al
 * escritorio.
 */
import { get, writable } from 'svelte/store';
import { dispositivo } from '../dispositivo/dispositivo';

export type Tema = 'light' | 'dark';
export type PreferenciaTema = Tema | 'system';

export const CLAVE_TEMA = 'stockmante:tema';
export const PREFERENCIA_POR_DEFECTO: PreferenciaTema = 'system';

export interface AlmacenLectura {
	getItem(clave: string): string | null;
}

export interface AlmacenEscritura {
	setItem(clave: string, valor: string): void;
}

export interface RaizDocumento {
	setAttribute(nombre: string, valor: string): void;
	style: { colorScheme: string };
}

const PREFERENCIAS: readonly PreferenciaTema[] = ['light', 'dark', 'system'];

export function esTema(valor: unknown): valor is PreferenciaTema {
	return typeof valor === 'string' && (PREFERENCIAS as readonly string[]).includes(valor);
}

/**
 * `system` delega en la preferencia del sistema; una elección explícita manda.
 * En layout móvil siempre gana `dark` (FR-104).
 */
export function resolverTema(
	preferencia: PreferenciaTema,
	prefiereOscuro: boolean,
	esLayoutMovil = false
): Tema {
	if (esLayoutMovil) return 'dark';
	if (preferencia === 'system') return prefiereOscuro ? 'dark' : 'light';
	return preferencia;
}

export function leerPreferencia(almacen: AlmacenLectura | null): PreferenciaTema {
	if (!almacen) return PREFERENCIA_POR_DEFECTO;
	try {
		const guardado = almacen.getItem(CLAVE_TEMA);
		return esTema(guardado) ? guardado : PREFERENCIA_POR_DEFECTO;
	} catch {
		// Almacenamiento bloqueado (modo privado): se usa el default sin romper el login.
		return PREFERENCIA_POR_DEFECTO;
	}
}

export function guardarPreferencia(
	almacen: AlmacenEscritura | null,
	preferencia: PreferenciaTema
): void {
	if (!almacen) return;
	try {
		almacen.setItem(CLAVE_TEMA, preferencia);
	} catch {
		// Sin persistencia la elección sigue viva en memoria durante la sesión de la página.
	}
}

export function aplicarTema(raiz: RaizDocumento, tema: Tema): void {
	raiz.setAttribute('data-tema', tema);
	raiz.style.colorScheme = tema;
}

/** Preferencia vigente: `light`, `dark` o `system`. */
export const preferencia = writable<PreferenciaTema>(PREFERENCIA_POR_DEFECTO);

function almacenLocal(): Storage | null {
	try {
		return typeof window === 'undefined' ? null : window.localStorage;
	} catch {
		return null;
	}
}

function prefiereOscuro(): boolean {
	return typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function aplicarPreferencia(valor: PreferenciaTema, oscuro: boolean): void {
	if (typeof document === 'undefined') return;
	aplicarTema(document.documentElement, resolverTema(valor, oscuro, get(dispositivo).esMovil));
}

/**
 * Arranca el tema al montar la pantalla y devuelve la limpieza para `onMount`.
 * Con preferencia `system` escucha cambios del sistema operativo en vivo y, además,
 * re-aplica el tema cuando el layout cruza el corte móvil/escritorio.
 */
export function iniciarTema(): () => void {
	if (typeof window === 'undefined') return () => {};

	const valor = leerPreferencia(almacenLocal());
	preferencia.set(valor);
	aplicarPreferencia(valor, prefiereOscuro());

	const consulta = window.matchMedia('(prefers-color-scheme: dark)');
	const alCambiarSistema = () => {
		const vigente = get(preferencia);
		if (vigente === 'system') aplicarPreferencia('system', consulta.matches);
	};
	consulta.addEventListener('change', alCambiarSistema);

	const dejarDeSeguirLayout = dispositivo.subscribe(() => {
		aplicarPreferencia(get(preferencia), prefiereOscuro());
	});

	return () => {
		consulta.removeEventListener('change', alCambiarSistema);
		dejarDeSeguirLayout();
	};
}

export function elegirPreferencia(valor: PreferenciaTema): void {
	guardarPreferencia(almacenLocal(), valor);
	preferencia.set(valor);
	aplicarPreferencia(valor, prefiereOscuro());
}
