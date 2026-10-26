/**
 * Cliente de depósitos, ubicaciones y asignaciones producto–ubicación (005).
 * Toda la superficie exige familia GESTION; el backend sigue siendo la autoridad.
 */
import { api } from './client';
import {
	ListaDepositosSchema,
	ListaUbicacionesSchema,
	DepositoSchema,
	UbicacionSchema,
	type ActualizarDepositoInput,
	type ActualizarUbicacionInput,
	type CrearDepositoInput,
	type CrearUbicacionInput,
	type Deposito,
	type Ubicacion
} from '../schemas/depositos';

export type FiltroActivo = 'true' | 'false' | 'all';

export async function listarDepositos(activo: FiltroActivo = 'true'): Promise<Deposito[]> {
	const query = new URLSearchParams({ activo });
	const body = await api<unknown>(`/api/depositos?${query.toString()}`);
	return ListaDepositosSchema.parse(body);
}

export async function obtenerDeposito(id: number): Promise<Deposito> {
	const body = await api<unknown>(`/api/depositos/${id}`);
	return DepositoSchema.parse(body);
}

export async function crearDeposito(datos: CrearDepositoInput): Promise<Deposito> {
	const body = await api<unknown>('/api/depositos', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return DepositoSchema.parse(body);
}

export async function actualizarDeposito(
	id: number,
	datos: ActualizarDepositoInput
): Promise<Deposito> {
	const body = await api<unknown>(`/api/depositos/${id}`, {
		method: 'PUT',
		body: JSON.stringify(datos)
	});
	return DepositoSchema.parse(body);
}

export async function desactivarDeposito(id: number): Promise<void> {
	await api(`/api/depositos/${id}/desactivar`, { method: 'POST' });
}

export async function reactivarDeposito(id: number): Promise<void> {
	await api(`/api/depositos/${id}/reactivar`, { method: 'POST' });
}

export async function eliminarDeposito(id: number): Promise<void> {
	await api(`/api/depositos/${id}`, { method: 'DELETE' });
}

/** Lista plana; el cliente arma el árbol con `padre_id`. */
export async function listarUbicaciones(depositoId: number): Promise<Ubicacion[]> {
	const body = await api<unknown>(`/api/depositos/${depositoId}/ubicaciones`);
	return ListaUbicacionesSchema.parse(body).items;
}

export async function crearUbicacion(datos: CrearUbicacionInput): Promise<Ubicacion> {
	const body = await api<unknown>('/api/ubicaciones', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return UbicacionSchema.parse(body);
}

export async function actualizarUbicacion(
	id: number,
	datos: ActualizarUbicacionInput
): Promise<Ubicacion> {
	const body = await api<unknown>(`/api/ubicaciones/${id}`, {
		method: 'PUT',
		body: JSON.stringify(datos)
	});
	return UbicacionSchema.parse(body);
}

export async function desactivarUbicacion(id: number): Promise<void> {
	await api(`/api/ubicaciones/${id}/desactivar`, { method: 'POST' });
}

export async function reactivarUbicacion(id: number): Promise<void> {
	await api(`/api/ubicaciones/${id}/reactivar`, { method: 'POST' });
}

export async function eliminarUbicacion(id: number): Promise<void> {
	await api(`/api/ubicaciones/${id}`, { method: 'DELETE' });
}

export async function ubicacionesDeProducto(productoId: number): Promise<Ubicacion[]> {
	const body = await api<unknown>(`/api/productos/${productoId}/ubicaciones`);
	return ListaUbicacionesSchema.parse(body).items;
}

/** Reemplaza el conjunto completo; `[]` lo limpia. */
export async function reemplazarUbicacionesDeProducto(
	productoId: number,
	ubicacionIds: number[]
): Promise<void> {
	await api(`/api/productos/${productoId}/ubicaciones`, {
		method: 'PUT',
		body: JSON.stringify({ ubicacion_ids: ubicacionIds })
	});
}
