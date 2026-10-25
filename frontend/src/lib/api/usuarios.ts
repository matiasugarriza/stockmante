/**
 * Cliente de gestión de usuarios (US2, contracts/api.md).
 * Todas las rutas exigen sesión de familia GESTION; el backend responde 403.
 */
import { api } from './client';
import {
	ListaUsuariosSchema,
	UsuarioSchema,
	type ActualizarUsuarioInput,
	type CrearUsuarioInput,
	type ListaUsuarios,
	type Usuario
} from '../schemas/usuarios';

export type FiltroActivo = 'true' | 'false' | 'all';

export interface ListarUsuariosParams {
	activo?: FiltroActivo;
	pagina?: number;
	porPagina?: number;
}

function queryString(params: ListarUsuariosParams): string {
	const query = new URLSearchParams();
	if (params.activo) query.set('activo', params.activo);
	if (params.pagina !== undefined) query.set('pagina', String(params.pagina));
	if (params.porPagina !== undefined) query.set('por_pagina', String(params.porPagina));
	const serializado = query.toString();
	return serializado ? `?${serializado}` : '';
}

export async function listarUsuarios(
	params: ListarUsuariosParams = {}
): Promise<ListaUsuarios> {
	const body = await api<unknown>(`/api/usuarios${queryString(params)}`);
	return ListaUsuariosSchema.parse(body);
}

export async function obtenerUsuario(id: number): Promise<Usuario> {
	const body = await api<unknown>(`/api/usuarios/${id}`);
	return UsuarioSchema.parse(body);
}

export async function crearUsuario(datos: CrearUsuarioInput): Promise<Usuario> {
	const body = await api<unknown>('/api/usuarios', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return UsuarioSchema.parse(body);
}

export async function actualizarUsuario(
	id: number,
	datos: ActualizarUsuarioInput
): Promise<Usuario> {
	const body = await api<unknown>(`/api/usuarios/${id}`, {
		method: 'PUT',
		body: JSON.stringify(datos)
	});
	return UsuarioSchema.parse(body);
}

/** Baja lógica; el backend revoca las sesiones del usuario. */
export async function desactivarUsuario(id: number): Promise<void> {
	await api(`/api/usuarios/${id}/desactivar`, { method: 'POST' });
}

export async function reactivarUsuario(id: number): Promise<void> {
	await api(`/api/usuarios/${id}/reactivar`, { method: 'POST' });
}

/** Regenera la temporal y la reenvía por correo; revoca sesiones vigentes. */
export async function regenerarPasswordTemporal(id: number): Promise<void> {
	await api(`/api/usuarios/${id}/password-temporal`, { method: 'POST' });
}
