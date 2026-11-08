import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarProducto,
	buscarProductos,
	crearProducto,
	desactivarProducto,
	listarProductos,
	obtenerProducto,
	reactivarProducto,
	resolverLectura
} from './productos';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const producto = {
	id: 9,
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: null,
	categoria: { id: 3, nombre: 'Almacén' },
	unidad_medida: 'UNIDAD',
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6,
	activo: true
};

const alta = {
	nombre: 'Yerba 1 kg',
	sku: 'YERBA-1K',
	codigo_barras: '',
	categoria_id: 3,
	unidad_medida: 'UNIDAD' as const,
	costo: 1200.5,
	precio_venta: 1800,
	stock_minimo: 6
};

describe('cliente de productos', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarProductos arma el query completo y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [producto], total: 1, pagina: 2, por_pagina: 50 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarProductos({
			q: 'yerba',
			categoriaId: 3,
			activo: 'all',
			pagina: 2,
			porPagina: 50
		});

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(
			'/api/productos?q=yerba&categoria_id=3&activo=all&pagina=2&por_pagina=50'
		);
		expect(lista.total).toBe(1);
	});

	it('listarProductos omite `activo` para usar el default del backend', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [], total: 0, pagina: 1, por_pagina: 20 })
		);
		vi.stubGlobal('fetch', fetchMock);

		await listarProductos();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos');
		expect(url).not.toContain('activo=');
	});

	it('crearProducto hace POST y devuelve el producto', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(producto, 201));
		vi.stubGlobal('fetch', fetchMock);

		const creado = await crearProducto(alta);

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(alta);
		expect(creado.categoria.id).toBe(3);
	});

	it('crearProducto propaga el 409 sku_duplicado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'sku_duplicado',
							mensaje: 'Ya existe un producto con el SKU YERBA-1K. Elegí otro.'
						}
					},
					409
				)
			)
		);

		await expect(crearProducto(alta)).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'sku_duplicado'
		} satisfies Partial<ApiError>);
	});

	it('obtenerProducto hace GET al id', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(producto));
		vi.stubGlobal('fetch', fetchMock);

		const encontrado = await obtenerProducto(9);

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/9');
		expect(encontrado.sku).toBe('YERBA-1K');
	});

	it('actualizarProducto hace PUT parcial', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...producto, precio_venta: 2000.75 }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizado = await actualizarProducto(9, { precio_venta: 2000.75 });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/9');
		expect(init.method).toBe('PUT');
		expect(actualizado.precio_venta).toBe(2000.75);
	});

	it('buscarProductos arma el query de caja y parsea los items', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [producto] }));
		vi.stubGlobal('fetch', fetchMock);

		const resultado = await buscarProductos({
			q: '  yerb  ',
			campo: 'nombre',
			incluirInactivos: true,
			categoriaId: 3,
			limite: 20
		});

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/buscar?');
		expect(url).toContain('q=yerb');
		expect(url).toContain('campo=nombre');
		expect(url).toContain('incluir_inactivos=true');
		expect(url).toContain('categoria_id=3');
		expect(url).toContain('limite=20');
		expect(resultado.items).toHaveLength(1);
	});

	it('buscarProductos omite q vacío, campo=todos e inactivos falsos', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [] }));
		vi.stubGlobal('fetch', fetchMock);

		await buscarProductos({ q: '   ', campo: 'todos', incluirInactivos: false });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/productos/buscar');
		expect(url).not.toContain('q=');
		expect(url).not.toContain('campo=');
		expect(url).not.toContain('incluir_inactivos=');
	});

	it('buscarProductos propaga el 403 de inactivos', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'sin_permisos',
							mensaje: 'No tenés permiso para realizar esta acción'
						}
					},
					403
				)
			)
		);

		await expect(buscarProductos({ q: 'yerba', incluirInactivos: true })).rejects.toMatchObject({
			name: 'ApiError',
			status: 403,
			codigo: 'sin_permisos'
		} satisfies Partial<ApiError>);
	});

	it.each([
		['desactivarProducto', desactivarProducto, '/api/productos/9/desactivar'],
		['reactivarProducto', reactivarProducto, '/api/productos/9/reactivar']
	])('%s hace POST y resuelve con 204', async (_nombre, accion, path) => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(accion(9)).resolves.toBeUndefined();

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(path);
		expect(init.method).toBe('POST');
	});
});

describe('resolverLectura (020, FR-138/FR-140/FR-141)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	function urls(fetchMock: ReturnType<typeof vi.fn>): string[] {
		return fetchMock.mock.calls.map((llamada) => String((llamada as unknown[])[0]));
	}

	it('acierta por código exacto y no consulta el SKU', async () => {
		const conCodigo = { ...producto, codigo_barras: '7791234567898' };
		const fetchMock = vi.fn(async () => jsonResponse({ items: [conCodigo] }));
		vi.stubGlobal('fetch', fetchMock);

		const resultado = await resolverLectura('7791234567898');

		expect(resultado).toEqual({ estado: 'encontrado', campo: 'codigo', producto: conCodigo });
		const pedidas = urls(fetchMock);
		expect(pedidas).toHaveLength(1);
		expect(pedidas[0]).toContain('campo=codigo');
		expect(pedidas[0]).toContain('q=7791234567898');
		expect(pedidas[0]).toContain('limite=2');
	});

	it('si no hay código, busca por SKU exacto', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValueOnce(jsonResponse({ items: [] }))
			.mockResolvedValueOnce(jsonResponse({ items: [producto] }));
		vi.stubGlobal('fetch', fetchMock);

		const resultado = await resolverLectura('yerba-1k');

		expect(resultado).toEqual({ estado: 'encontrado', campo: 'sku', producto });
		const pedidas = urls(fetchMock);
		expect(pedidas[1]).toContain('campo=sku');
		expect(pedidas.some((u) => u.includes('campo=todos'))).toBe(false);
		expect(pedidas.every((u) => u.includes('campo='))).toBe(true);
	});

	it('sin coincidencias exactas es no_encontrado (nunca aproxima)', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [] }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(resolverLectura('00000000')).resolves.toEqual({ estado: 'no_encontrado' });
		expect(fetchMock).toHaveBeenCalledTimes(2);
	});

	it('con incluirInactivos (gestión) informa el producto dado de baja', async () => {
		const inactivo = { ...producto, activo: false };
		const fetchMock = vi.fn(async () => jsonResponse({ items: [inactivo] }));
		vi.stubGlobal('fetch', fetchMock);

		const resultado = await resolverLectura('YERBA-1K', { incluirInactivos: true });

		expect(resultado).toEqual({ estado: 'inactivo', campo: 'codigo', producto: inactivo });
		expect(urls(fetchMock)[0]).toContain('incluir_inactivos=true');
	});

	it('sin incluirInactivos (operación) no pide inactivos para evitar el 403', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [] }));
		vi.stubGlobal('fetch', fetchMock);

		await resolverLectura('YERBA-1K');

		expect(urls(fetchMock).some((u) => u.includes('incluir_inactivos'))).toBe(false);
	});

	it('normaliza la lectura antes de buscar', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [producto] }));
		vi.stubGlobal('fetch', fetchMock);

		await resolverLectura(' 779 1234567898 ');

		expect(urls(fetchMock)[0]).toContain('q=7791234567898');
	});

	it('vacío o más de 160 caracteres es no_encontrado sin pedir nada', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ items: [] }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(resolverLectura('   ')).resolves.toEqual({ estado: 'no_encontrado' });
		await expect(resolverLectura('x'.repeat(161))).resolves.toEqual({ estado: 'no_encontrado' });
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('propaga el error de red para reintentar con el mismo valor (FR-155)', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new TypeError('Failed to fetch');
			})
		);

		await expect(resolverLectura('7791234567898')).rejects.toBeInstanceOf(Error);
	});
});
