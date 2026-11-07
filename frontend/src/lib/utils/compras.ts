/**
 * Aviso del resultado de confirmar o anular una compra (012, Principio III): qué compra,
 * en qué depósito y con qué stock quedó cada producto.
 */
export interface ResultadoParaAviso {
	compra: { id: number };
	movimiento: {
		tipo: string;
		deposito: { nombre: string };
		lineas: ReadonlyArray<{ producto_nombre: string; stock_resultante: number }>;
	};
}

export function avisoResultadoCompra(r: ResultadoParaAviso): string {
	const accion = r.movimiento.tipo === 'ENTRADA' ? 'confirmada' : 'anulada';
	const detalle = r.movimiento.lineas
		.map((l) => `${l.producto_nombre} → ${l.stock_resultante} u.`)
		.join(', ');
	return `Compra #${r.compra.id} ${accion} en ${r.movimiento.deposito.nombre}: ${detalle}`;
}
