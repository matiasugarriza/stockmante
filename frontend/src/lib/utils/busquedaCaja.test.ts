import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { crearBusquedaDiferida, LIMITE_CAJA, mensajeTruncados } from './busquedaCaja';

describe('crearBusquedaDiferida', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('no ejecuta la tarea antes de la espera y sí al cumplirse', () => {
		const busqueda = crearBusquedaDiferida(250);
		const tarea = vi.fn();

		busqueda.programar(tarea);
		vi.advanceTimersByTime(249);
		expect(tarea).not.toHaveBeenCalled();

		vi.advanceTimersByTime(1);
		expect(tarea).toHaveBeenCalledTimes(1);
	});

	it('dos programaciones seguidas ejecutan solo la última', () => {
		const busqueda = crearBusquedaDiferida(250);
		const primera = vi.fn();
		const segunda = vi.fn();

		busqueda.programar(primera);
		vi.advanceTimersByTime(100);
		busqueda.programar(segunda);
		vi.advanceTimersByTime(250);

		expect(primera).not.toHaveBeenCalled();
		expect(segunda).toHaveBeenCalledTimes(1);
	});

	it('esVigente pasa a false si se programa otra búsqueda mientras la primera espera respuesta', () => {
		const busqueda = crearBusquedaDiferida(250);
		let vigentePrimera: (() => boolean) | undefined;
		let vigenteSegunda: (() => boolean) | undefined;

		busqueda.programar((esVigente) => {
			vigentePrimera = esVigente;
		});
		vi.advanceTimersByTime(250);
		expect(vigentePrimera?.()).toBe(true);

		busqueda.programar((esVigente) => {
			vigenteSegunda = esVigente;
		});
		expect(vigentePrimera?.()).toBe(false);

		vi.advanceTimersByTime(250);
		expect(vigenteSegunda?.()).toBe(true);
	});

	it('cancelar evita la ejecución pendiente e invalida la búsqueda en curso', () => {
		const busqueda = crearBusquedaDiferida(250);
		const pendiente = vi.fn();
		let vigenteEnCurso: (() => boolean) | undefined;

		busqueda.programar((esVigente) => {
			vigenteEnCurso = esVigente;
		});
		vi.advanceTimersByTime(250);
		busqueda.programar(pendiente);
		busqueda.cancelar();
		vi.advanceTimersByTime(500);

		expect(pendiente).not.toHaveBeenCalled();
		expect(vigenteEnCurso?.()).toBe(false);
	});
});

describe('mensajeTruncados', () => {
	it('no avisa sin resultados, con menos que el límite ni con límite inválido', () => {
		expect(mensajeTruncados(0, 20)).toBeNull();
		expect(mensajeTruncados(19, 20)).toBeNull();
		expect(mensajeTruncados(0, 0)).toBeNull();
		expect(mensajeTruncados(-1, -1)).toBeNull();
	});

	it('avisa y propone refinar cuando la cantidad iguala el límite', () => {
		expect(mensajeTruncados(20, 20)).toBe(
			'Se muestran los primeros 20 resultados. Refiná la búsqueda para encontrar el producto.'
		);
	});

	it('el límite de caja es 20', () => {
		expect(LIMITE_CAJA).toBe(20);
	});
});
