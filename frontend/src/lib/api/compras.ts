/**
 * Cliente de compras (012). Toda la superficie exige familia GESTION.
 * Crear/editar solo aplica a BORRADOR; confirmar y anular son acciones
 * transaccionales del backend.
 */
import { api } from './client';
import {
	CompraDetalleSchema,
	CrearCompraSchema,
	ListaComprasSchema,
	type ActualizarCompraInput,
	type CompraDetalle,
	type CrearCompraInput,
	type EstadoCompra,
	type ListaCompras
} from '../schemas/compras';

export interface ListarComprasParams {
	proveedorId?: number;
	estado?: EstadoCompra;
	desde?: string;
	hasta?: string;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarComprasParams): string {
	const query = new URLSearchParams();
	if (params.proveedorId !== undefined) query.set('proveedor_id', String(params.proveedorId));
	if (params.estado) query.set('estado', params.estado);
	if (params.desde) query.set('desde', params.desde);
	if (params.hasta) query.set('hasta', params.hasta);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

/** `numero`/`observacion` vacíos se envían como null, no como cadena vacía. */
function payload(datos: CrearCompraInput) {
	const p = CrearCompraSchema.parse(datos);
	return {
		proveedor_id: p.proveedor_id,
		deposito_id: p.deposito_id,
		numero: p.numero || null,
		observacion: p.observacion || null,
		lineas: p.lineas
	};
}

export async function listarCompras(params: ListarComprasParams = {}): Promise<ListaCompras> {
	const body = await api<unknown>(`/api/compras${queryString(params)}`);
	return ListaComprasSchema.parse(body);
}

export async function obtenerCompra(id: number): Promise<CompraDetalle> {
	const body = await api<unknown>(`/api/compras/${id}`);
	return CompraDetalleSchema.parse(body);
}

export async function crearCompra(datos: CrearCompraInput): Promise<CompraDetalle> {
	const body = await api<unknown>('/api/compras', {
		method: 'POST',
		body: JSON.stringify(payload(datos))
	});
	return CompraDetalleSchema.parse(body);
}

export async function actualizarCompra(
	id: number,
	datos: ActualizarCompraInput
): Promise<CompraDetalle> {
	const body = await api<unknown>(`/api/compras/${id}`, {
		method: 'PUT',
		body: JSON.stringify(payload(datos))
	});
	return CompraDetalleSchema.parse(body);
}

/** Solo un BORRADOR se puede borrar; el backend responde 204. */
export async function eliminarCompra(id: number): Promise<void> {
	await api(`/api/compras/${id}`, { method: 'DELETE' });
}

export async function confirmarCompra(id: number): Promise<CompraDetalle> {
	const body = await api<unknown>(`/api/compras/${id}/confirmar`, { method: 'POST' });
	return CompraDetalleSchema.parse(body);
}

export async function anularCompra(id: number): Promise<CompraDetalle> {
	const body = await api<unknown>(`/api/compras/${id}/anular`, { method: 'POST' });
	return CompraDetalleSchema.parse(body);
}