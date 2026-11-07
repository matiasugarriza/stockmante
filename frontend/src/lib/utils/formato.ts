/**
 * Formato de números y fechas para la interfaz (spec 018, FR-093).
 * Locale es-AR; funciones puras y deterministas salvo la zona horaria local.
 */

const MONEDA = new Intl.NumberFormat('es-AR', {
	style: 'currency',
	currency: 'ARS',
	minimumFractionDigits: 2,
	maximumFractionDigits: 2
});

const ENTERO = new Intl.NumberFormat('es-AR', { maximumFractionDigits: 0 });

const FECHA = new Intl.DateTimeFormat('es-AR', {
	day: '2-digit',
	month: '2-digit',
	year: 'numeric'
});

function esNumero(valor: number): boolean {
	return typeof valor === 'number' && Number.isFinite(valor);
}

export function formatearMoneda(valor: number): string {
	return esNumero(valor) ? MONEDA.format(valor) : '—';
}

export function formatearNumero(valor: number): string {
	return esNumero(valor) ? ENTERO.format(valor) : '—';
}

export function formatearPorcentaje(valor: number, decimales = 0): string {
	if (!esNumero(valor)) return '—';
	const formato = new Intl.NumberFormat('es-AR', {
		minimumFractionDigits: decimales,
		maximumFractionDigits: decimales
	});
	return `${formato.format(valor)} %`;
}

export function formatearFechaCorta(iso: string | null | undefined): string {
	if (!iso) return '—';
	const fecha = new Date(iso);
	return Number.isNaN(fecha.getTime()) ? '—' : FECHA.format(fecha);
}

/** Recorta a `max` caracteres incluyendo la elipsis. */
export function truncar(texto: string, max: number): string {
	if (max <= 0) return '';
	if (texto.length <= max) return texto;
	if (max === 1) return '…';
	return `${texto.slice(0, max - 1)}…`;
}
