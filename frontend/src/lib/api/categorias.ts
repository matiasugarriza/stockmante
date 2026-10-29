/**
 * Cliente del catálogo: categorías (003, contracts/api.md).
 * Lectura: cualquier sesión; escritura: familia GESTION.
 */
import { api } from './client';
import {
	ListaCategoriasSchema,
	CategoriaSchema,
	type ActualizarCategoriaInput,
	type CrearCategoriaInput,
	type Categoria
} from '../schemas/categorias';

export type FiltroActivo = 'true' | 'false' | 'all';

export async function listarCategorias(activo: FiltroActivo = 'true'): Promise<Categoria[]> {
	const query = new URLSearchParams({ activo });
	const body = await api<unknown>(`/api/categorias?${query.toString()}`);
	return ListaCategoriasSchema.parse(body);
}

export async function obtenerCategoria(id: number): Promise<Categoria> {
	const body = await api<unknown>(`/api/categorias/${id}`);
	return CategoriaSchema.parse(body);
}

export async function crearCategoria(datos: CrearCategoriaInput): Promise<Categoria> {
	const body = await api<unknown>('/api/categorias', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
	return CategoriaSchema.parse(body);
}

export async function actualizarCategoria(
	id: number,
	datos: ActualizarCategoriaInput
): Promise<Categoria> {
	const body = await api<unknown>(`/api/categorias/${id}`, {
		method: 'PUT',
		body: JSON.stringify(datos)
	});
	return CategoriaSchema.parse(body);
}

/** Baja lógica: nunca hay borrado físico. */
export async function desactivarCategoria(id: number): Promise<void> {
	await api(`/api/categorias/${id}/desactivar`, { method: 'POST' });
}

export async function reactivarCategoria(id: number): Promise<void> {
	await api(`/api/categorias/${id}/reactivar`, { method: 'POST' });
}
