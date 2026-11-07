/** Cantidad de resultados que pide `/caja`; el aviso de truncados usa el mismo valor. */
export const LIMITE_CAJA = 20;

export type TareaBusqueda = (esVigente: () => boolean) => Promise<void> | void;

export interface BusquedaDiferida {
	programar(tarea: TareaBusqueda): void;
	cancelar(): void;
}

/**
 * Espera una pausa de tipeo antes de buscar y permite descartar respuestas que llegan después
 * de una búsqueda más nueva: cada tarea recibe `esVigente`, que deja de ser `true` en cuanto se
 * programa otra o se cancela.
 */
export function crearBusquedaDiferida(esperaMs: number): BusquedaDiferida {
	let turno = 0;
	let temporizador: ReturnType<typeof setTimeout> | undefined;

	function limpiarTemporizador() {
		if (temporizador !== undefined) {
			clearTimeout(temporizador);
			temporizador = undefined;
		}
	}

	return {
		programar(tarea) {
			limpiarTemporizador();
			const propio = ++turno;
			temporizador = setTimeout(() => {
				temporizador = undefined;
				void tarea(() => propio === turno);
			}, esperaMs);
		},
		cancelar() {
			limpiarTemporizador();
			turno++;
		}
	};
}

/** Aviso para cuando la API devolvió el máximo pedido: puede haber más coincidencias. */
export function mensajeTruncados(cantidad: number, limite: number): string | null {
	if (limite <= 0 || cantidad !== limite) return null;
	return `Se muestran los primeros ${limite} resultados. Refiná la búsqueda para encontrar el producto.`;
}
