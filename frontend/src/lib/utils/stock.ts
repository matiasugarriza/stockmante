/**
 * Lógica pura de la consulta de stock (010): índices, depósito recordado y texto
 * del valorizado. Sin I/O: los componentes la usan y los tests la cubren.
 */

/** Mapa `producto_id → stock en el depósito elegido` (null cuenta como 0). */
export function indexarPorProducto(
	items: ReadonlyArray<{ producto_id: number; stock_deposito: number | null }>
): Map<number, number> {
	return new Map(items.map((item) => [item.producto_id, item.stock_deposito ?? 0]));
}

/**
 * Depósito a usar: el guardado si sigue entre los activos; si no, el primero activo;
 * `null` si no hay ninguno.
 */
export function elegirDeposito(
	guardado: number | null | undefined,
	activos: ReadonlyArray<{ id: number }>
): number | null {
	if (guardado != null && activos.some((d) => d.id === guardado)) return guardado;
	return activos[0]?.id ?? null;
}

const IMPORTE = new Intl.NumberFormat('es-AR', {
	minimumFractionDigits: 2,
	maximumFractionDigits: 2
});

/** `$ 1.234,50`, o `—` cuando la sesión no ve costos. */
export function textoValorizado(item: { valorizado: number | null }): string {
	return item.valorizado === null ? '—' : `$ ${IMPORTE.format(item.valorizado)}`;
}
