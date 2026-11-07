/**
 * Modelo de navegación del marco (spec 018, FR-089).
 *
 * Funciones puras con dependencias inyectadas: no tocan `window`, `page` ni
 * stores. La barra lateral solo renderiza; la visibilidad por rol y el estado
 * activo se prueban en `navegacion.test.ts`.
 */
import type { NombreIcono } from '../components/ui/iconos';

export type FamiliaRol = 'GESTION' | 'OPERACION';
export type GrupoNavegacion = 'operacion' | 'administracion';

export interface ContextoNavegacion {
	familiaRol: FamiliaRol;
	alertasStock: number;
}

export interface ItemNavegacion {
	etiqueta: string;
	ruta: string;
	icono: NombreIcono;
	grupo: GrupoNavegacion;
	roles: FamiliaRol[];
	contador?: 'alertas-stock';
}

export interface ItemNavegacionVisible extends ItemNavegacion {
	activo: boolean;
	contadorVisible: number | null;
}

export interface GrupoNavegacionVisible {
	id: GrupoNavegacion;
	etiqueta: string;
	items: ItemNavegacionVisible[];
}

export const GRUPOS: ReadonlyArray<{ id: GrupoNavegacion; etiqueta: string }> = [
	{ id: 'operacion', etiqueta: 'Operación' },
	{ id: 'administracion', etiqueta: 'Administración' }
];

export const ITEMS_NAVEGACION: readonly ItemNavegacion[] = [
	{ etiqueta: 'Inicio', ruta: '/', icono: 'inicio', grupo: 'operacion', roles: ['GESTION', 'OPERACION'] },
	{ etiqueta: 'Caja', ruta: '/caja', icono: 'caja', grupo: 'operacion', roles: ['GESTION', 'OPERACION'] },
	{
		etiqueta: 'Movimientos',
		ruta: '/movimientos',
		icono: 'movimientos',
		grupo: 'operacion',
		roles: ['GESTION', 'OPERACION']
	},
	{
		etiqueta: 'Transferencias',
		ruta: '/transferencias',
		icono: 'transferencias',
		grupo: 'operacion',
		roles: ['GESTION', 'OPERACION']
	},
	{ etiqueta: 'Conteos', ruta: '/conteos', icono: 'conteos', grupo: 'operacion', roles: ['GESTION'] },
	{
		etiqueta: 'Asistente',
		ruta: '/asistente',
		icono: 'asistente',
		grupo: 'operacion',
		roles: ['GESTION', 'OPERACION']
	},
	{
		etiqueta: 'Productos',
		ruta: '/admin/productos',
		icono: 'productos',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Categorías',
		ruta: '/admin/categorias',
		icono: 'categorias',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Depósitos',
		ruta: '/admin/depositos',
		icono: 'depositos',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Stock',
		ruta: '/admin/stock',
		icono: 'stock',
		grupo: 'administracion',
		roles: ['GESTION'],
		contador: 'alertas-stock'
	},
	{
		etiqueta: 'Proveedores',
		ruta: '/admin/proveedores',
		icono: 'proveedores',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Compras',
		ruta: '/admin/compras',
		icono: 'compras',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Ventas',
		ruta: '/admin/ventas',
		icono: 'caja',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Clientes',
		ruta: '/admin/clientes',
		icono: 'usuarios',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Reportes',
		ruta: '/admin/reportes',
		icono: 'stock',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Importar',
		ruta: '/admin/importacion',
		icono: 'productos',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Usuarios',
		ruta: '/admin/usuarios',
		icono: 'usuarios',
		grupo: 'administracion',
		roles: ['GESTION']
	},
	{
		etiqueta: 'Auditoría',
		ruta: '/admin/auditoria',
		icono: 'auditoria',
		grupo: 'administracion',
		roles: ['GESTION']
	}
];

/** `/` solo coincide exacto; el resto también con subrutas (`/admin/productos/1`). */
export function esRutaActiva(rutaActual: string, rutaItem: string): boolean {
	const limpia =
		rutaActual.length > 1 && rutaActual.endsWith('/') ? rutaActual.slice(0, -1) : rutaActual;
	if (rutaItem === '/') return limpia === '/';
	return limpia === rutaItem || limpia.startsWith(`${rutaItem}/`);
}

export function gruposVisibles(
	rutaActual: string,
	contexto: ContextoNavegacion
): GrupoNavegacionVisible[] {
	return GRUPOS.map((grupo) => ({
		...grupo,
		items: ITEMS_NAVEGACION.filter(
			(item) => item.grupo === grupo.id && item.roles.includes(contexto.familiaRol)
		).map<ItemNavegacionVisible>((item) => ({
			...item,
			activo: esRutaActiva(rutaActual, item.ruta),
			contadorVisible:
				item.contador === 'alertas-stock' &&
				contexto.familiaRol === 'GESTION' &&
				contexto.alertasStock > 0
					? contexto.alertasStock
					: null
		}))
	})).filter((grupo) => grupo.items.length > 0);
}

/** Título de la barra superior para la ruta actual. */
export function tituloDeRuta(rutaActual: string): string {
	const item = ITEMS_NAVEGACION.find((item) => esRutaActiva(rutaActual, item.ruta));
	return item?.etiqueta ?? 'Stockmante';
}
