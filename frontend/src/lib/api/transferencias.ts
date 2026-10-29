/**
 * Cliente de transferencias entre depósitos (008, contracts/api.md).
 */
import { api } from './client';
import {
	CrearTransferenciaSchema,
	ListaTransferenciasSchema,
	TransferenciaSchema,
	type CrearTransferenciaInput,
	type ListaTransferencias,
	type Transferencia
} from '../schemas/transferencias';

export interface ListarTransferenciasParams {
	productoId?: number;
	depositoOrigenId?: number;
	depositoDestinoId?: number;
	desde?: string;
	hasta?: string;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarTransferenciasParams): string {
	const query = new URLSearchParams();
	if (params.productoId !== undefined) query.set('producto_id', String(params.productoId));
	if (params.depositoOrigenId !== undefined) query.set('deposito_origen_id', String(params.depositoOrigenId));
	if (params.depositoDestinoId !== undefined) query.set('deposito_destino_id', String(params.depositoDestinoId));
	if (params.desde) query.set('desde', params.desde);
	if (params.hasta) query.set('hasta', params.hasta);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

export async function listarTransferencias(
	params: ListarTransferenciasParams = {}
): Promise<ListaTransferencias> {
	const body = await api<unknown>(`/api/transferencias${queryString(params)}`);
	return ListaTransferenciasSchema.parse(body);
}

export async function obtenerTransferencia(id: number): Promise<Transferencia> {
	const body = await api<unknown>(`/api/transferencias/${id}`);
	return TransferenciaSchema.parse(body);
}

export async function crearTransferencia(datos: CrearTransferenciaInput): Promise<Transferencia> {
	const body = await api<unknown>('/api/transferencias', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return TransferenciaSchema.parse(body);
}
