/**
 * Cliente de clientes (013). Lectura para cualquier sesión; escritura solo GESTION.
 */
import { api } from './client';
import {
	ClienteSchema,
	ListaClientesSchema,
	NuevoClienteSchema,
	type Cliente,
	type ListaClientes,
	type NuevoClienteInput
} from '../schemas/clientes';

export interface ListarClientesParams {
	activo?: boolean;
	pagina?: number;
	porPagina?: number;
}

export async function listarClientes(params: ListarClientesParams = {}): Promise<ListaClientes> {
	const query = new URLSearchParams();
	if (params.activo !== undefined) query.set('activo', String(params.activo));
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const qs = query.toString();
	const body = await api<unknown>(`/api/clientes${qs ? `?${qs}` : ''}`);
	return ListaClientesSchema.parse(body);
}

export async function obtenerCliente(id: number): Promise<Cliente> {
	return ClienteSchema.parse(await api<unknown>(`/api/clientes/${id}`));
}

export async function crearCliente(datos: NuevoClienteInput): Promise<Cliente> {
	const body = await api<unknown>('/api/clientes', {
		method: 'POST',
		body: JSON.stringify(NuevoClienteSchema.parse(datos))
	});
	return ClienteSchema.parse(body);
}

export async function actualizarCliente(id: number, datos: NuevoClienteInput): Promise<Cliente> {
	const body = await api<unknown>(`/api/clientes/${id}`, {
		method: 'PUT',
		body: JSON.stringify(NuevoClienteSchema.parse(datos))
	});
	return ClienteSchema.parse(body);
}

export async function activarCliente(id: number): Promise<Cliente> {
	return ClienteSchema.parse(await api<unknown>(`/api/clientes/${id}/activar`, { method: 'POST' }));
}

export async function desactivarCliente(id: number): Promise<Cliente> {
	return ClienteSchema.parse(
		await api<unknown>(`/api/clientes/${id}/desactivar`, { method: 'POST' })
	);
}
