/**
 * Cliente de consulta de stock (007, FR-029).
 * Cualquier sesión vigente puede consultar.
 */
import { api } from './client';
import {
	ListaStockSchema,
	StockProductoSchema,
	type ListaStock,
	type StockProducto
} from '../schemas/movimientos';

export interface ListarStockParams {
	q?: string;
	depositoId?: number;
	categoriaId?: number;
	bajoMinimo?: boolean;
	/** Ausente = solo activos (default del backend). */
	activo?: 'true' | 'false' | 'all';
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarStockParams): string {
	const query = new URLSearchParams();
	if (params.q) query.set('q', params.q);
	if (params.depositoId !== undefined) query.set('deposito_id', String(params.depositoId));
	if (params.categoriaId !== undefined) query.set('categoria_id', String(params.categoriaId));
	if (params.bajoMinimo) query.set('bajo_minimo', 'true');
	if (params.activo) query.set('activo', params.activo);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

export async function listarStock(params: ListarStockParams = {}): Promise<ListaStock> {
	const body = await api<unknown>(`/api/stock${queryString(params)}`);
	return ListaStockSchema.parse(body);
}

/** Desglose por depósito y ubicación de un producto. */
export async function obtenerStockProducto(productoId: number): Promise<StockProducto> {
	const body = await api<unknown>(`/api/stock/${productoId}`);
	return StockProductoSchema.parse(body);
}
