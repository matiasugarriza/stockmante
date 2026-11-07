/**
 * Clasificación de stock para las tarjetas móviles (spec 019, FR-118).
 *
 * Sin I/O: la usan las tarjetas y la prueba `inventario.test.ts`.
 */

export type EstadoStock = 'normal' | 'bajo' | 'critico';

/**
 * Crítico = sin stock; bajo = marcado bajo mínimo; normal = resto.
 * Se apoya en `stock_actual` y `bajo_minimo`, los únicos datos que expone la API.
 */
export function clasificarStock(stockActual: number, bajoMinimo: boolean): EstadoStock {
	if (stockActual <= 0) return 'critico';
	if (bajoMinimo) return 'bajo';
	return 'normal';
}

/** Etiqueta textual: el color nunca es la única señal (FR-118). */
export function etiquetaStock(estado: EstadoStock): string {
	if (estado === 'critico') return 'Crítico';
	if (estado === 'bajo') return 'Bajo';
	return 'Normal';
}

/** Variante del badge existente (`EtiquetaEstado`). */
export function varianteStock(estado: EstadoStock): 'exito' | 'aviso' | 'peligro' {
	if (estado === 'critico') return 'peligro';
	if (estado === 'bajo') return 'aviso';
	return 'exito';
}

export interface EntradaValidacionCantidad {
	tipo: 'ENTRADA' | 'SALIDA' | 'MERMA' | 'AJUSTE';
	cantidad: string | number;
	disponible: number | null;
	esGestion: boolean;
}

/**
 * Validación previa de la hoja de movimiento (FR-124). Devuelve el mensaje o `null`.
 * El backend sigue siendo la autoridad final.
 */
export function validarCantidadMovil({
	tipo,
	cantidad,
	disponible,
	esGestion
}: EntradaValidacionCantidad): string | null {
	const texto = typeof cantidad === 'string' ? cantidad.trim() : String(cantidad);
	if (texto === '') return 'Ingresá una cantidad.';
	const numero = Number(texto);
	if (!Number.isFinite(numero) || !Number.isInteger(numero)) {
		return 'La cantidad debe ser un número entero.';
	}
	if (numero <= 0) return 'La cantidad debe ser mayor a cero.';
	if (tipo === 'AJUSTE' && !esGestion) return 'El ajuste requiere permisos de gestión.';
	if (tipo === 'SALIDA' || tipo === 'MERMA') {
		if (disponible === null) return 'Elegí un depósito con stock disponible.';
		if (numero > disponible) {
			return `No podés mover ${numero}: hay ${disponible} disponibles en el depósito.`;
		}
	}
	return null;
}
