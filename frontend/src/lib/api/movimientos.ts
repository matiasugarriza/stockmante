/**
 * Cliente de movimientos de stock (007, contracts/api.md).
 * Alta: cualquier sesión vigente (AJUSTE requiere GESTION); consulta: sesión vigente.
 */
import { api } from './client';
import {
	ListaMovimientosSchema,
	MovimientoDetalleSchema,
	type CrearMovimientoInput,
	type ListaMovimientos,
	type MovimientoDetalle
} from '../schemas/movimientos';

export interface ListarMovimientosParams {
	tipo?: string;
	depositoId?: number;
	productoId?: number;
	desde?: string;
	hasta?: string;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarMovimientosParams): string {
	const query = new URLSearchParams();
	if (params.tipo) query.set('tipo', params.tipo);
	if (params.depositoId !== undefined) query.set('deposito_id', String(params.depositoId));
	if (params.productoId !== undefined) query.set('producto_id', String(params.productoId));
	if (params.desde) query.set('desde', params.desde);
	if (params.hasta) query.set('hasta', params.hasta);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

export async function listarMovimientos(
	params: ListarMovimientosParams = {}
): Promise<ListaMovimientos> {
	const body = await api<unknown>(`/api/movimientos${queryString(params)}`);
	return ListaMovimientosSchema.parse(body);
}

export async function obtenerMovimiento(id: number): Promise<MovimientoDetalle> {
	const body = await api<unknown>(`/api/movimientos/${id}`);
	return MovimientoDetalleSchema.parse(body);
}

/** 201 con el movimiento y el stock resultante por línea. */
export async function crearMovimiento(datos: CrearMovimientoInput): Promise<MovimientoDetalle> {
	const body = await api<unknown>('/api/movimientos', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return MovimientoDetalleSchema.parse(body);
}
