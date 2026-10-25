/** Cantidad de páginas para `total` elementos con `porPagina` (mínimo 1). */
export function totalPaginas(total: number, porPagina: number): number {
	if (!Number.isFinite(porPagina) || porPagina <= 0) return 1;
	if (!Number.isFinite(total) || total <= 0) return 1;
	return Math.max(1, Math.ceil(total / porPagina));
}

/** Recorta `pagina` al rango [1, totalPaginas]. */
export function paginaValida(pagina: number, total: number, porPagina: number): number {
	const maximo = totalPaginas(total, porPagina);
	if (!Number.isFinite(pagina)) return 1;
	return Math.min(Math.max(1, Math.trunc(pagina)), maximo);
}
