/**
 * Cliente del catálogo: productos (003, contracts/api.md).
 * Listado de gestión y escritura: familia GESTION; detalle: cualquier sesión.
 */
import { api } from './client';
import {
	ListaProductosSchema,
	ProductoSchema,
	type ActualizarProductoInput,
	type CrearProductoInput,
	type ListaProductos,
	type Producto
} from '../schemas/productos';

export type FiltroActivo = 'true' | 'false' | 'all';

export interface ListarProductosParams {
	q?: string;
	categoriaId?: number;
	activo?: FiltroActivo;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarProductosParams): string {
	const query = new URLSearchParams();
	if (params.q) query.set('q', params.q);
	if (params.categoriaId !== undefined) query.set('categoria_id', String(params.categoriaId));
	if (params.activo) query.set('activo', params.activo);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

export async function listarProductos(
	params: ListarProductosParams = {}
): Promise<ListaProductos> {
	const body = await api<unknown>(`/api/productos${queryString(params)}`);
	return ListaProductosSchema.parse(body);
}

export async function obtenerProducto(id: number): Promise<Producto> {
	const body = await api<unknown>(`/api/productos/${id}`);
	return ProductoSchema.parse(body);
}

export async function crearProducto(datos: CrearProductoInput): Promise<Producto> {
	const body = await api<unknown>('/api/productos', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return ProductoSchema.parse(body);
}

export async function actualizarProducto(
	id: number,
	datos: ActualizarProductoInput
): Promise<Producto> {
	const body = await api<unknown>(`/api/productos/${id}`, {
		method: 'PUT',
		body: JSON.stringify(datos)
	});
	return ProductoSchema.parse(body);
}

/** Baja lógica: nunca hay borrado físico. */
export async function desactivarProducto(id: number): Promise<void> {
	await api(`/api/productos/${id}/desactivar`, { method: 'POST' });
}

export async function reactivarProducto(id: number): Promise<void> {
	await api(`/api/productos/${id}/reactivar`, { method: 'POST' });
}
