/**
 * Índices del carrusel de métricas (spec 019, FR-114 y FR-132).
 *
 * Funciones puras: los componentes solo las usan y la prueba las cubre.
 */

export function normalizarIndice(indice: number, total: number): number {
	if (total <= 0) return 0;
	return ((indice % total) + total) % total;
}

export function indiceSiguiente(indice: number, total: number): number {
	if (total <= 0) return 0;
	return normalizarIndice(indice + 1, total);
}

export function indiceAnterior(indice: number, total: number): number {
	if (total <= 0) return 0;
	return normalizarIndice(indice - 1, total);
}
