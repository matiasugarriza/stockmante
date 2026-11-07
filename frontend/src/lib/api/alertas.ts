/**
 * Cliente de alertas de stock mínimo (011). Solo GESTION; OPERACION recibe 403.
 */
import { api } from './client';
import {
	ListaAlertasSchema,
	RespuestaCicloSchema,
	type ListaAlertas,
	type RespuestaCiclo
} from '../schemas/alertas';

export interface ListarAlertasParams {
	pagina?: number;
	porPagina?: number;
}

export async function listarAlertas(params: ListarAlertasParams = {}): Promise<ListaAlertas> {
	const query = new URLSearchParams();
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	const body = await api<unknown>(`/api/alertas${serializado ? `?${serializado}` : ''}`);
	return ListaAlertasSchema.parse(body);
}

/** Dispara el ciclo de digest por correo (idempotente en el día). */
export async function enviarAlertas(): Promise<RespuestaCiclo> {
	const body = await api<unknown>('/api/alertas/enviar', { method: 'POST' });
	return RespuestaCicloSchema.parse(body);
}
