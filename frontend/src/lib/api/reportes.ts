/**
 * Cliente de reportes (014). El backend exige familia GESTION (403 para OPERACION).
 * Las fechas son días locales `AAAA-MM-DD`; rotación y ventas exigen ambas.
 */
import { api, BASE } from './client';
import {
	KardexSchema,
	RotacionSchema,
	StockValorizadoSchema,
	VentasReporteSchema,
	type Kardex,
	type Rotacion,
	type StockValorizado,
	type VentasReporte
} from '../schemas/reportes';

export type NombreReporte = 'kardex' | 'stock-valorizado' | 'rotacion' | 'ventas';

export interface ParametrosReporte {
	productoId?: number;
	depositoId?: number;
	desde?: string;
	hasta?: string;
	agrupar?: 'dia' | 'producto';
}

function query(params: ParametrosReporte, extra: Record<string, string> = {}): string {
	const q = new URLSearchParams();
	if (params.productoId !== undefined) q.set('producto_id', String(params.productoId));
	if (params.depositoId !== undefined) q.set('deposito_id', String(params.depositoId));
	if (params.desde) q.set('desde', params.desde);
	if (params.hasta) q.set('hasta', params.hasta);
	if (params.agrupar) q.set('agrupar', params.agrupar);
	for (const [clave, valor] of Object.entries(extra)) q.set(clave, valor);
	const texto = q.toString();
	return texto ? `?${texto}` : '';
}

export async function obtenerStockValorizado(): Promise<StockValorizado> {
	const body = await api<unknown>('/api/reportes/stock-valorizado');
	return StockValorizadoSchema.parse(body);
}

export async function obtenerKardex(params: ParametrosReporte & { productoId: number }): Promise<Kardex> {
	const body = await api<unknown>(`/api/reportes/kardex${query(params)}`);
	return KardexSchema.parse(body);
}

export async function obtenerRotacion(params: ParametrosReporte): Promise<Rotacion> {
	const body = await api<unknown>(`/api/reportes/rotacion${query(params)}`);
	return RotacionSchema.parse(body);
}

export async function obtenerVentas(params: ParametrosReporte): Promise<VentasReporte> {
	const body = await api<unknown>(`/api/reportes/ventas${query(params)}`);
	return VentasReporteSchema.parse(body);
}

/** URL absoluta de la descarga CSV (la cookie de sesión viaja con el enlace). */
export function urlCsv(reporte: NombreReporte, params: ParametrosReporte = {}): string {
	return `${BASE}/api/reportes/${reporte}${query(params, { formato: 'csv' })}`;
}
