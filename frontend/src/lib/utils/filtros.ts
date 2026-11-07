/** Filtros activos de las listas móviles (spec 019, FR-120 y FR-132). */

export interface FiltroActivo {
	etiqueta: string;
	valor: string;
}

/** Cuenta los filtros que aportan un valor (los vacíos no cuentan). */
export function contarFiltrosActivos(filtros: ReadonlyArray<FiltroActivo>): number {
	return filtros.filter((filtro) => filtro.valor.trim() !== '').length;
}

/** Texto del disparador del panel de filtros. */
export function resumenFiltros(cantidad: number): string {
	if (cantidad <= 0) return 'Filtros';
	if (cantidad === 1) return '1 filtro activo';
	return `${cantidad} filtros activos`;
}
