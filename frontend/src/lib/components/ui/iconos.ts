/**
 * Iconografía SVG propia (spec 018, FR-086/FR-093).
 * Trazo uniforme de 24×24; el componente `Icono.svelte` los hereda del color actual.
 */

export type NombreIcono =
	| 'inicio'
	| 'caja'
	| 'movimientos'
	| 'transferencias'
	| 'conteos'
	| 'asistente'
	| 'productos'
	| 'categorias'
	| 'depositos'
	| 'stock'
	| 'proveedores'
	| 'compras'
	| 'usuarios'
	| 'auditoria'
	| 'menu'
	| 'cerrar'
	| 'sol'
	| 'luna'
	| 'sistema'
	| 'buscar'
	| 'flecha'
	| 'alerta'
	| 'check'
	| 'mas'
	| 'editar'
	| 'basura'
	| 'salir';

export type ElementoIcono =
	| { tag: 'path'; d: string }
	| { tag: 'circle'; cx: number; cy: number; r: number }
	| { tag: 'rect'; x: number; y: number; width: number; height: number; rx?: number };

export const ICONOS: Record<NombreIcono, ElementoIcono[]> = {
	inicio: [
		{ tag: 'path', d: 'M3 10.5 12 3l9 7.5' },
		{ tag: 'path', d: 'M5 9.5V21h14V9.5' },
		{ tag: 'path', d: 'M9 21v-6h6v6' }
	],
	caja: [
		{ tag: 'path', d: 'M6 7h12l1 14H5L6 7Z' },
		{ tag: 'path', d: 'M9 7V5a3 3 0 0 1 6 0v2' }
	],
	movimientos: [
		{ tag: 'path', d: 'M7 4v16M7 4 4 7M7 4l3 3' },
		{ tag: 'path', d: 'M17 20V4M17 20l-3-3M17 20l3-3' }
	],
	transferencias: [
		{ tag: 'path', d: 'M4 7h13M17 7l-3-3M17 7l-3 3' },
		{ tag: 'path', d: 'M20 17H7M7 17l3-3M7 17l3 3' }
	],
	conteos: [
		{ tag: 'path', d: 'M9 4h6v3H9z' },
		{ tag: 'path', d: 'M8 5H6v16h12V5h-2' },
		{ tag: 'path', d: 'M9 12h6M9 16h4' }
	],
	asistente: [
		{ tag: 'path', d: 'M4 5h16v11H9l-5 4V5Z' },
		{ tag: 'path', d: 'M9 10h.01M12.5 10h.01M16 10h.01' }
	],
	productos: [
		{ tag: 'path', d: 'M3 7l9-4 9 4-9 4-9-4Z' },
		{ tag: 'path', d: 'M3 7v10l9 4 9-4V7' },
		{ tag: 'path', d: 'M12 11v10' }
	],
	categorias: [
		{ tag: 'path', d: 'M3 12V4h8l9 9-8 8-9-9Z' },
		{ tag: 'circle', cx: 7.5, cy: 7.5, r: 0.75 }
	],
	depositos: [
		{ tag: 'path', d: 'M3 21V9l9-5 9 5v12' },
		{ tag: 'path', d: 'M7 21v-7h10v7' },
		{ tag: 'path', d: 'M3 21h18' }
	],
	stock: [
		{ tag: 'path', d: 'M12 3 3 8l9 5 9-5-9-5Z' },
		{ tag: 'path', d: 'M3 12l9 5 9-5' },
		{ tag: 'path', d: 'M3 16l9 5 9-5' }
	],
	proveedores: [
		{ tag: 'path', d: 'M3 6h11v10H3z' },
		{ tag: 'path', d: 'M14 9h4l3 3v4h-7' },
		{ tag: 'circle', cx: 7, cy: 18, r: 1.75 },
		{ tag: 'circle', cx: 18, cy: 18, r: 1.75 }
	],
	compras: [
		{ tag: 'path', d: 'M6 3h12v18l-3-2-3 2-3-2-3 2V3Z' },
		{ tag: 'path', d: 'M9 8h6M9 12h6M9 16h4' }
	],
	usuarios: [
		{ tag: 'path', d: 'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2' },
		{ tag: 'circle', cx: 9, cy: 7, r: 4 },
		{ tag: 'path', d: 'M22 21v-2a4 4 0 0 0-3-3.87' },
		{ tag: 'path', d: 'M16 3.13a4 4 0 0 1 0 7.75' }
	],
	auditoria: [
		{ tag: 'path', d: 'M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6l8-3Z' },
		{ tag: 'path', d: 'M9 12l2 2 4-4' }
	],
	menu: [{ tag: 'path', d: 'M4 6h16M4 12h16M4 18h16' }],
	cerrar: [{ tag: 'path', d: 'M6 6l12 12M18 6 6 18' }],
	sol: [
		{ tag: 'circle', cx: 12, cy: 12, r: 4 },
		{
			tag: 'path',
			d: 'M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4'
		}
	],
	luna: [{ tag: 'path', d: 'M20 14.5A8.5 8.5 0 0 1 9.5 4 7 7 0 1 0 20 14.5Z' }],
	sistema: [
		{ tag: 'rect', x: 2.5, y: 4, width: 19, height: 13, rx: 2 },
		{ tag: 'path', d: 'M8 21h8M12 17v4' }
	],
	buscar: [
		{ tag: 'circle', cx: 11, cy: 11, r: 7 },
		{ tag: 'path', d: 'm20 20-3.5-3.5' }
	],
	flecha: [{ tag: 'path', d: 'M9 6l6 6-6 6' }],
	alerta: [
		{ tag: 'path', d: 'M12 3 2.5 20h19L12 3Z' },
		{ tag: 'path', d: 'M12 9v5M12 17h.01' }
	],
	check: [{ tag: 'path', d: 'M5 12l5 5L20 7' }],
	mas: [{ tag: 'path', d: 'M12 5v14M5 12h14' }],
	editar: [
		{ tag: 'path', d: 'M4 20h4L20 8l-4-4L4 16v4Z' },
		{ tag: 'path', d: 'M14 6l4 4' }
	],
	basura: [
		{ tag: 'path', d: 'M4 7h16M9 7V4h6v3' },
		{ tag: 'path', d: 'M6 7l1 14h10l1-14' }
	],
	salir: [
		{ tag: 'path', d: 'M9 21H5V3h4' },
		{ tag: 'path', d: 'M16 17l5-5-5-5M21 12H9' }
	]
};

export const NOMBRES_ICONO = Object.keys(ICONOS) as NombreIcono[];
