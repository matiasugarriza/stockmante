/**
 * API Rust falsa para tests: responde por `MÉTODO /ruta` y registra cada llamada.
 * Permite afirmar que el servicio nunca hace otra cosa que GET (FR-067, SC-025).
 */

export interface Llamada {
	metodo: string;
	ruta: string;
	cookie: string | null;
}

export interface RespuestaFalsa {
	status: number;
	body: unknown;
}

export type Rutas = Record<string, RespuestaFalsa | ((url: URL) => RespuestaFalsa)>;

export function crearFetchFalso(rutas: Rutas): { fetch: typeof fetch; llamadas: Llamada[] } {
	const llamadas: Llamada[] = [];
	const fetchFalso = (async (entrada: string | URL | Request, init?: RequestInit) => {
		const url = new URL(String(entrada));
		const metodo = (init?.method ?? 'GET').toUpperCase();
		const headers = new Headers(init?.headers);
		llamadas.push({ metodo, ruta: url.pathname + url.search, cookie: headers.get('cookie') });
		const definida = rutas[`${metodo} ${url.pathname}`];
		const respuesta: RespuestaFalsa =
			typeof definida === 'function'
				? definida(url)
				: (definida ?? {
						status: 404,
						body: { error: { codigo: 'recurso_no_encontrado', mensaje: 'No existe.' } }
					});
		return new Response(JSON.stringify(respuesta.body), {
			status: respuesta.status,
			headers: { 'content-type': 'application/json' }
		});
	}) as typeof fetch;
	return { fetch: fetchFalso, llamadas };
}

export function producto(id: number, nombre: string, sku: string, descripcion = '') {
	return {
		id,
		nombre: descripcion ? `${nombre} ${descripcion}` : nombre,
		sku,
		codigo_barras: null,
		categoria: { id: 1, nombre: 'Almacén' },
		unidad_medida: 'UNIDAD',
		costo: 100,
		precio_venta: 150,
		stock_minimo: 5,
		activo: true
	};
}

export function stockDe(id: number, nombre: string, sku: string, desglose: Array<[number, string, number]>) {
	return {
		producto_id: id,
		nombre,
		sku,
		unidad_medida: 'UNIDAD',
		activo: true,
		stock_actual: desglose.reduce((t, [, , c]) => t + c, 0),
		stock_minimo: 5,
		bajo_minimo: false,
		desglose: desglose.map(([deposito_id, deposito_nombre, cantidad]) => ({
			deposito_id,
			deposito_nombre,
			ubicacion_id: null,
			ubicacion_nombre: null,
			cantidad
		}))
	};
}

/** Catálogo de la spec: tres yerbas (escenario 17.10) y un depósito Principal. */
export function rutasCatalogo(): Rutas {
	const yerbas = [
		producto(1, 'Yerba Suave 1 kg', 'YER-SUA-1K'),
		producto(2, 'Yerba Suave 500 g', 'YER-SUA-500'),
		producto(3, 'Yerba Compuesta 1 kg', 'YER-COM-1K')
	];
	return {
		'GET /api/productos/buscar': (url) => {
			const q = (url.searchParams.get('q') ?? '').toLowerCase();
			const items = yerbas.filter(
				(p) => p.nombre.toLowerCase().includes(q) || p.sku.toLowerCase() === q
			);
			return { status: 200, body: { items } };
		},
		'GET /api/stock/1': { status: 200, body: stockDe(1, 'Yerba Suave 1 kg', 'YER-SUA-1K', [[1, 'Principal', 12]]) },
		'GET /api/stock/2': { status: 200, body: stockDe(2, 'Yerba Suave 500 g', 'YER-SUA-500', [[1, 'Principal', 3], [2, 'Sucursal', 8]]) },
		'GET /api/stock/3': { status: 200, body: stockDe(3, 'Yerba Compuesta 1 kg', 'YER-COM-1K', [[1, 'Principal', 0]]) },
		'GET /api/stock': (url) => {
			const q = (url.searchParams.get('q') ?? '').toLowerCase();
			const items = yerbas
				.filter((p) => p.nombre.toLowerCase().includes(q))
				.map((p) => ({ producto_id: p.id, nombre: p.nombre, sku: p.sku }));
			return { status: 200, body: { items, total: items.length, pagina: 1, por_pagina: 5 } };
		},
		'GET /api/depositos': {
			status: 200,
			body: [
				{ id: 1, nombre: 'Principal', identificador: 'PRI', direccion: null, activo: true },
				{ id: 2, nombre: 'Sucursal', identificador: 'SUC', direccion: null, activo: true },
				{ id: 3, nombre: 'Depósito Norte', identificador: 'NOR', direccion: null, activo: true }
			]
		}
	};
}
