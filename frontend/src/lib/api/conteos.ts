/**
 * Cliente de conteos de inventario (009, contracts/api.md).
 */
import { api } from './client';
import {
	CargarLineasSchema,
	ConteoSchema,
	CrearConteoSchema,
	ListaConteosSchema,
	type Conteo,
	type CargarLineasInput,
	type CrearConteoInput,
	type ListaConteos
} from '../schemas/conteos';

export interface ListarConteosParams {
	depositoId?: number;
	estado?: string;
	desde?: string;
	hasta?: string;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarConteosParams): string {
	const query = new URLSearchParams();
	if (params.depositoId !== undefined) query.set('deposito_id', String(params.depositoId));
	if (params.estado) query.set('estado', params.estado);
	if (params.desde) query.set('desde', params.desde);
	if (params.hasta) query.set('hasta', params.hasta);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

export async function listarConteos(params: ListarConteosParams = {}): Promise<ListaConteos> {
	const body = await api<unknown>(`/api/conteos${queryString(params)}`);
	return ListaConteosSchema.parse(body);
}

export async function obtenerConteo(id: number): Promise<Conteo> {
	const body = await api<unknown>(`/api/conteos/${id}`);
	return ConteoSchema.parse(body);
}

export async function crearConteo(datos: CrearConteoInput): Promise<Conteo> {
	const body = await api<unknown>('/api/conteos', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return ConteoSchema.parse(body);
}

export async function cargarLineas(id: number, datos: CargarLineasInput): Promise<Conteo> {
	const body = await api<unknown>(`/api/conteos/${id}/lineas`, {
		method: 'PATCH',
		body: JSON.stringify(datos)
	});
	return ConteoSchema.parse(body);
}

export async function cerrarConteo(id: number): Promise<Conteo> {
	const body = await api<unknown>(`/api/conteos/${id}/cerrar`, { method: 'POST' });
	return ConteoSchema.parse(body);
}

export async function cancelarConteo(id: number): Promise<Conteo> {
	const body = await api<unknown>(`/api/conteos/${id}/cancelar`, { method: 'POST' });
	return ConteoSchema.parse(body);
}