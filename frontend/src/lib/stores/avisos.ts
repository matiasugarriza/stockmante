/**
 * Avisos flotantes de la interfaz móvil (spec 019, FR-125).
 *
 * El temporizador es inyectable para poder probar la cola sin relojes reales.
 */
import { writable } from 'svelte/store';

export type VarianteAviso = 'exito' | 'error';

export interface Aviso {
	id: number;
	mensaje: string;
	variante: VarianteAviso;
}

export interface Temporizador {
	setTimeout(callback: () => void, ms: number): unknown;
	clearTimeout(id: unknown): void;
}

export interface OpcionesAviso {
	duracionMs?: number;
	temporizador?: Temporizador;
}

export const DURACION_AVISO_MS = 5000;

const temporizadorReal: Temporizador = {
	setTimeout: (callback, ms) => setTimeout(callback, ms),
	clearTimeout: (id) => clearTimeout(id as ReturnType<typeof setTimeout>)
};

export const aviso = writable<Aviso | null>(null);

let secuencia = 0;
let pendiente: { id: unknown; temporizador: Temporizador } | null = null;

/** Cierra el aviso visible y cancela su expiración. */
export function cerrarAviso(): void {
	if (pendiente) {
		pendiente.temporizador.clearTimeout(pendiente.id);
		pendiente = null;
	}
	aviso.set(null);
}

/** Publica un aviso y programa su cierre. Un aviso nuevo reemplaza al anterior. */
export function mostrarAviso(
	mensaje: string,
	variante: VarianteAviso = 'exito',
	opciones: OpcionesAviso = {}
): void {
	if (pendiente) pendiente.temporizador.clearTimeout(pendiente.id);
	secuencia += 1;
	aviso.set({ id: secuencia, mensaje, variante });
	const temporizador = opciones.temporizador ?? temporizadorReal;
	const id = temporizador.setTimeout(() => {
		pendiente = null;
		aviso.set(null);
	}, opciones.duracionMs ?? DURACION_AVISO_MS);
	pendiente = { id, temporizador };
}

const ETIQUETAS_TIPO: Record<string, string> = {
	ENTRADA: 'Entrada registrada',
	SALIDA: 'Salida registrada',
	MERMA: 'Merma registrada',
	AJUSTE: 'Ajuste registrado'
};

/**
 * Resultado concreto exigido por el Principio III: tipo, producto, depósito, cantidad
 * (valor absoluto) y stock resultante.
 */
export function mensajeMovimiento(
	tipo: string,
	producto: string,
	deposito: string,
	cantidad: number,
	stockResultante: number
): string {
	const encabezado = ETIQUETAS_TIPO[tipo] ?? 'Movimiento registrado';
	return `${encabezado}: ${producto} · ${deposito} · ${Math.abs(cantidad)} → stock ${stockResultante}`;
}
