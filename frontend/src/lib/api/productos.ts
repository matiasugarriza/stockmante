/**
 * Cliente del catálogo: productos (003, contracts/api.md).
 * Listado de gestión y escritura: familia GESTION; detalle: cualquier sesión.
 */
import { api } from './client';
import { normalizarLectura } from '../codigos/escaner';
import {
	ListaBusquedaSchema,
	ListaProductosSchema,
	ProductoSchema,
	type ActualizarProductoInput,
	type CrearProductoInput,
	type ListaBusqueda,
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

/** Campos de la búsqueda de caja (004). `todos` es el default del backend. */
export type CampoBusqueda = 'sku' | 'codigo' | 'nombre' | 'categoria' | 'todos';

export interface BuscarProductosParams {
	q?: string;
	campo?: CampoBusqueda;
	incluirInactivos?: boolean;
	categoriaId?: number;
	limite?: number;
}

/**
 * GET /api/productos/buscar. `q` se recorta; `campo=todos` y los valores
 * opcionales ausentes no se envían para usar los defaults del backend.
 */
export async function buscarProductos(params: BuscarProductosParams = {}): Promise<ListaBusqueda> {
	const query = new URLSearchParams();
	const q = params.q?.trim();
	if (q) query.set('q', q);
	if (params.campo && params.campo !== 'todos') query.set('campo', params.campo);
	if (params.incluirInactivos) query.set('incluir_inactivos', 'true');
	if (params.categoriaId !== undefined) query.set('categoria_id', String(params.categoriaId));
	if (params.limite !== undefined) query.set('limite', String(params.limite));
	const serializado = query.toString();
	const body = await api<unknown>(
		`/api/productos/buscar${serializado ? `?${serializado}` : ''}`
	);
	return ListaBusquedaSchema.parse(body);
}

/** Largo máximo de `q` en la búsqueda de caja (backend `Q_MAX`). */
const Q_MAX = 160;

export type ResultadoLectura =
	| { estado: 'encontrado' | 'inactivo'; campo: 'codigo' | 'sku'; producto: Producto }
	| { estado: 'no_encontrado' };

/**
 * 020 (R3): resuelve una lectura de cámara, lector o teclado a lo sumo a un producto, por
 * coincidencia exacta de código y luego de SKU. Nunca usa `campo=todos`, que aproximaría por
 * nombre (FR-140). `incluirInactivos` solo para gestión: el backend responde 403 a operación.
 * Los errores de red se propagan para que la pantalla reintente con el mismo valor (FR-155).
 */
export async function resolverLectura(
	valor: string,
	opciones: { incluirInactivos?: boolean } = {}
): Promise<ResultadoLectura> {
	const q = normalizarLectura(valor);
	if (q === null || q.length > Q_MAX) return { estado: 'no_encontrado' };

	for (const campo of ['codigo', 'sku'] as const) {
		const { items } = await buscarProductos({
			q,
			campo,
			incluirInactivos: opciones.incluirInactivos,
			limite: 2
		});
		const producto = items[0];
		if (producto) {
			return { estado: producto.activo ? 'encontrado' : 'inactivo', campo, producto };
		}
	}
	return { estado: 'no_encontrado' };
}
