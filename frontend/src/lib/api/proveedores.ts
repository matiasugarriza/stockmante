/**
 * Cliente de proveedores (012). Toda la superficie exige familia GESTION;
 * el backend sigue siendo la autoridad (403 `sin_permisos`).
 */
import { api } from './client';
import {
	CrearProveedorSchema,
	ListaProveedoresSchema,
	ProveedorSchema,
	type ActualizarProveedorInput,
	type CrearProveedorInput,
	type ListaProveedores,
	type Proveedor
} from '../schemas/proveedores';

export type FiltroActivo = 'true' | 'false' | 'all';

export interface ListarProveedoresParams {
	activo?: FiltroActivo;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarProveedoresParams): string {
	const query = new URLSearchParams();
	if (params.activo) query.set('activo', params.activo);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

/** El backend distingue null de ausente; los campos vacíos se envían como null. */
function payload(datos: CrearProveedorInput) {
	const p = CrearProveedorSchema.parse(datos);
	return {
		nombre: p.nombre,
		cuit: p.cuit || null,
		email: p.email || null,
		telefono: p.telefono || null,
		direccion: p.direccion || null
	};
}

export async function listarProveedores(
	params: ListarProveedoresParams = {}
): Promise<ListaProveedores> {
	const body = await api<unknown>(`/api/proveedores${queryString(params)}`);
	return ListaProveedoresSchema.parse(body);
}

export async function obtenerProveedor(id: number): Promise<Proveedor> {
	const body = await api<unknown>(`/api/proveedores/${id}`);
	return ProveedorSchema.parse(body);
}

export async function crearProveedor(datos: CrearProveedorInput): Promise<Proveedor> {
	const body = await api<unknown>('/api/proveedores', {
		method: 'POST',
		body: JSON.stringify(payload(datos))
	});
	return ProveedorSchema.parse(body);
}

export async function actualizarProveedor(
	id: number,
	datos: ActualizarProveedorInput
): Promise<Proveedor> {
	const body = await api<unknown>(`/api/proveedores/${id}`, {
		method: 'PUT',
		body: JSON.stringify(payload(datos))
	});
	return ProveedorSchema.parse(body);
}

/** Baja lógica: nunca hay borrado físico. Devuelve el proveedor actualizado. */
export async function desactivarProveedor(id: number): Promise<Proveedor> {
	const body = await api<unknown>(`/api/proveedores/${id}/desactivar`, { method: 'POST' });
	return ProveedorSchema.parse(body);
}

export async function reactivarProveedor(id: number): Promise<Proveedor> {
	const body = await api<unknown>(`/api/proveedores/${id}/activar`, { method: 'POST' });
	return ProveedorSchema.parse(body);
}