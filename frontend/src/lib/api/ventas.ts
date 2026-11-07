/**
 * Cliente de ventas de caja (013). Vender y consultar: cualquier sesión; anular: GESTION.
 */
import { api } from './client';
import {
	ListaVentasSchema,
	NuevaVentaSchema,
	VentaDetalleSchema,
	type EstadoVenta,
	type ListaVentas,
	type NuevaVentaInput,
	type VentaDetalle
} from '../schemas/ventas';

export interface ListarVentasParams {
	depositoId?: number;
	clienteId?: number;
	estado?: EstadoVenta;
	desde?: string;
	hasta?: string;
	pagina?: number;
	porPagina?: number;
}

export async function crearVenta(datos: NuevaVentaInput): Promise<VentaDetalle> {
	const body = await api<unknown>('/api/ventas', {
		method: 'POST',
		body: JSON.stringify(NuevaVentaSchema.parse(datos))
	});
	return VentaDetalleSchema.parse(body);
}

export async function listarVentas(params: ListarVentasParams = {}): Promise<ListaVentas> {
	const query = new URLSearchParams();
	if (params.depositoId !== undefined) query.set('deposito_id', String(params.depositoId));
	if (params.clienteId !== undefined) query.set('cliente_id', String(params.clienteId));
	if (params.estado) query.set('estado', params.estado);
	if (params.desde) query.set('desde', params.desde);
	if (params.hasta) query.set('hasta', params.hasta);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const qs = query.toString();
	const body = await api<unknown>(`/api/ventas${qs ? `?${qs}` : ''}`);
	return ListaVentasSchema.parse(body);
}

export async function obtenerVenta(id: number): Promise<VentaDetalle> {
	return VentaDetalleSchema.parse(await api<unknown>(`/api/ventas/${id}`));
}

export async function anularVenta(id: number): Promise<VentaDetalle> {
	return VentaDetalleSchema.parse(
		await api<unknown>(`/api/ventas/${id}/anular`, { method: 'POST' })
	);
}
