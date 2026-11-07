/**
 * Búsqueda de inventario para la cabecera móvil (spec 019, FR-111).
 *
 * Compone los clientes existentes: `/api/stock` cubre nombre y SKU; el código de barras
 * se resuelve con `/api/productos/buscar` y luego se completa el stock por ids. Sin
 * endpoints nuevos.
 */
import { buscarProductos } from './productos';
import { listarStock } from './stock';
import type { StockItem } from '../schemas/movimientos';

/** Forma de un código de barras admitido por el backend (008–64 alfanuméricos). */
export const PATRON_CODIGO = /^[0-9A-Za-z]{8,64}$/;

export const LIMITE_INVENTARIO = 20;

/**
 * Devuelve hasta `limite` productos con stock para el texto buscado.
 * Con menos de dos caracteres no consulta.
 */
export async function buscarInventario(
	texto: string,
	limite: number = LIMITE_INVENTARIO
): Promise<StockItem[]> {
	const q = texto.trim();
	if (q.length < 2) return [];

	const porStock = await listarStock({ q, porPagina: limite });
	if (porStock.items.length > 0) return porStock.items;
	if (!PATRON_CODIGO.test(q)) return [];

	const productos = await buscarProductos({ q, campo: 'codigo', limite });
	const ids = productos.items.map((producto) => producto.id).slice(0, 50);
	if (ids.length === 0) return [];

	const porId = await listarStock({ productoIds: ids, porPagina: limite });
	return porId.items;
}
