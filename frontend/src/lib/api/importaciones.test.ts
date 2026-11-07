import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import { importarProductos } from './importaciones';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const resumen = {
	filas_leidas: 3,
	validas: 2,
	con_error: 1,
	errores: [{ fila: 4, columna: 'costo', mensaje: 'Costo inválido.' }],
	creados: 0,
	aplicado: false
};

describe('cliente de importación (016)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('manda el archivo en FormData con el modo en la query y parsea el resumen', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(resumen));
		vi.stubGlobal('fetch', fetchMock);
		const archivo = new File(['nombre,sku'], 'productos.csv', { type: 'text/csv' });

		const r = await importarProductos(archivo, 'validar');

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/importaciones/productos?modo=validar');
		expect(init.body).toBeInstanceOf(FormData);
		expect((init.body as FormData).get('archivo')).toBeInstanceOf(File);
		expect(r.errores[0].columna).toBe('costo');
	});

	it('en aplicar con errores propaga ApiError con los detalles', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{
						error: {
							codigo: 'importacion_invalida',
							mensaje: 'La importación tiene 1 fila con errores.',
							detalles: [{ fila: 4, columna: 'sku', mensaje: "El SKU 'YERBA-1K' ya existe." }]
						}
					},
					422
				)
			)
		);
		const archivo = new File(['x'], 'p.csv');

		const error = await importarProductos(archivo, 'aplicar').catch((e: unknown) => e);

		expect(error).toBeInstanceOf(ApiError);
		expect((error as ApiError).codigo).toBe('importacion_invalida');
		expect((error as ApiError).detalles?.[0].fila).toBe(4);
	});
});
