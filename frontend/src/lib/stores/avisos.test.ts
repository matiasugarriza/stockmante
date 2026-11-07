import { get } from 'svelte/store';
import { afterEach, describe, expect, it } from 'vitest';
import { aviso, cerrarAviso, mensajeMovimiento, mostrarAviso } from './avisos';

interface TemporizadorFalso {
	callbacks: Array<() => void>;
	api: {
		setTimeout: (callback: () => void, ms: number) => number;
		clearTimeout: (id: number) => void;
	};
}

function temporizadorFalso(): TemporizadorFalso {
	const callbacks: Array<() => void> = [];
	let id = 0;
	return {
		callbacks,
		api: {
			setTimeout: (callback: () => void) => {
				callbacks.push(callback);
				id += 1;
				return id;
			},
			clearTimeout: () => {}
		}
	};
}

describe('cola de avisos', () => {
	afterEach(() => {
		cerrarAviso();
	});

	it('publica el aviso con su variante', () => {
		mostrarAviso('Entrada registrada', 'exito', { temporizador: temporizadorFalso().api });
		expect(get(aviso)).toMatchObject({ mensaje: 'Entrada registrada', variante: 'exito' });
	});

	it('se cierra solo al expirar el temporizador inyectado', () => {
		const reloj = temporizadorFalso();
		mostrarAviso('Salida registrada', 'exito', { duracionMs: 5000, temporizador: reloj.api });
		expect(get(aviso)).not.toBeNull();
		expect(reloj.callbacks).toHaveLength(1);
		reloj.callbacks[0]?.();
		expect(get(aviso)).toBeNull();
	});

	it('se cierra manualmente', () => {
		mostrarAviso('Error de red', 'error', { temporizador: temporizadorFalso().api });
		cerrarAviso();
		expect(get(aviso)).toBeNull();
	});

	it('un aviso nuevo reemplaza al anterior', () => {
		const reloj = temporizadorFalso();
		mostrarAviso('Primero', 'exito', { temporizador: reloj.api });
		mostrarAviso('Segundo', 'error', { temporizador: reloj.api });
		expect(get(aviso)?.mensaje).toBe('Segundo');
		expect(get(aviso)?.variante).toBe('error');
	});
});

describe('mensajeMovimiento', () => {
	it('arma el resultado concreto de una entrada', () => {
		expect(mensajeMovimiento('ENTRADA', 'Yerba 1 kg', 'Central', 5, 12)).toBe(
			'Entrada registrada: Yerba 1 kg · Central · 5 → stock 12'
		);
	});

	it('usa el valor absoluto de la cantidad en salidas y mermas', () => {
		expect(mensajeMovimiento('SALIDA', 'Yerba 1 kg', 'Central', -5, 7)).toBe(
			'Salida registrada: Yerba 1 kg · Central · 5 → stock 7'
		);
		expect(mensajeMovimiento('MERMA', 'Yerba 1 kg', 'Central', -1, 6)).toBe(
			'Merma registrada: Yerba 1 kg · Central · 1 → stock 6'
		);
	});

	it('nombra el ajuste con su resultado', () => {
		expect(mensajeMovimiento('AJUSTE', 'Yerba 1 kg', 'Central', 3, 9)).toBe(
			'Ajuste registrado: Yerba 1 kg · Central · 3 → stock 9'
		);
	});
});
