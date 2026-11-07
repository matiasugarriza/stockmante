import { afterEach, describe, expect, it, vi } from 'vitest';
import {
	obtenerKardex,
	obtenerRotacion,
	obtenerStockValorizado,
	obtenerVentas,
	urlCsv
} from './reportes';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

describe('cliente de reportes', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('obtenerStockValorizado hace GET y parsea items y total', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({
				items: [
					{
						producto_id: 10,
						nombre: 'Yerba 1 kg',
						sku: 'YER-001',
						cantidad: 10,
						costo_unitario: 1500.5,
						valor: 15005
					}
				],
				total: 15005
			})
		);
		vi.stubGlobal('fetch', fetchMock);

		const reporte = await obtenerStockValorizado();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/reportes/stock-valorizado');
		expect(reporte.items).toHaveLength(1);
		expect(reporte.total).toBe(15005);
	});

	it('propaga el 403 de un usuario sin gestión', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{ error: { codigo: 'sin_permisos', mensaje: 'No tenés permisos de gestión.' } },
					403
				)
			)
		);

		await expect(obtenerStockValorizado()).rejects.toMatchObject({
			name: 'ApiError',
			status: 403,
			codigo: 'sin_permisos'
		});
	});

	it('obtenerKardex arma producto, depósito y fechas, y parsea las líneas', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({
				producto_id: 10,
				producto_nombre: 'Yerba',
				deposito_id: 2,
				desde: '2026-10-01',
				hasta: '2026-10-31',
				saldo_inicial: 20,
				lineas: [
					{
						movimiento_id: 5,
						creado_en: '2026-10-05T12:00:00Z',
						tipo: 'SALIDA',
						motivo: 'VENTA',
						deposito_id: 2,
						deposito_nombre: 'Principal',
						cantidad: -4,
						saldo: 16
					}
				],
				saldo_final: 16
			})
		);
		vi.stubGlobal('fetch', fetchMock);

		const k = await obtenerKardex({ productoId: 10, depositoId: 2, desde: '2026-10-01', hasta: '2026-10-31' });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		const q = new URL(url, 'http://x').searchParams;
		expect(url).toContain('/api/reportes/kardex?');
		expect(q.get('producto_id')).toBe('10');
		expect(q.get('deposito_id')).toBe('2');
		expect(q.get('desde')).toBe('2026-10-01');
		expect(k.lineas[0].saldo).toBe(16);
	});

	it('obtenerRotacion y obtenerVentas exigen y envían el rango', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValueOnce(
				jsonResponse({
					desde: '2026-10-01',
					hasta: '2026-10-31',
					items: [
						{
							producto_id: 1,
							nombre: 'Yerba',
							unidades_salidas: 4,
							stock_inicial: 20,
							stock_final: 26,
							stock_promedio: 23,
							indice_rotacion: 0.1739
						}
					]
				})
			)
			.mockResolvedValueOnce(
				jsonResponse({
					desde: '2026-10-01',
					hasta: '2026-10-31',
					cantidad_ventas: 2,
					total: 300,
					por_dia: [{ dia: '2026-10-10', cantidad_ventas: 2, total: 300 }],
					por_producto: [{ producto_id: 1, nombre: 'Yerba', unidades: 3, total: 300 }]
				})
			);
		vi.stubGlobal('fetch', fetchMock);

		const r = await obtenerRotacion({ desde: '2026-10-01', hasta: '2026-10-31' });
		const v = await obtenerVentas({ desde: '2026-10-01', hasta: '2026-10-31' });

		expect(r.items[0].indice_rotacion).toBeCloseTo(0.1739);
		expect(v.por_dia[0].dia).toBe('2026-10-10');
		const [urlVentas] = fetchMock.mock.calls[1] as unknown as [string, RequestInit];
		expect(urlVentas).toContain('desde=2026-10-01');
		expect(urlVentas).toContain('hasta=2026-10-31');
	});

	it('propaga el 422 de un rango inválido', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{ error: { codigo: 'validacion', mensaje: 'La fecha desde no puede ser posterior a la fecha hasta.' } },
					422
				)
			)
		);
		await expect(obtenerVentas({ desde: '2026-11-02', hasta: '2026-11-01' })).rejects.toMatchObject({
			name: 'ApiError',
			status: 422
		});
	});

	it('urlCsv arma la ruta con formato=csv y los parámetros', () => {
		const url = urlCsv('ventas', { desde: '2026-10-01', hasta: '2026-10-31', agrupar: 'producto' });
		expect(url).toContain('/api/reportes/ventas?');
		const q = new URL(url).searchParams;
		expect(q.get('formato')).toBe('csv');
		expect(q.get('agrupar')).toBe('producto');
		expect(q.get('desde')).toBe('2026-10-01');
		expect(urlCsv('stock-valorizado')).toMatch(/\/api\/reportes\/stock-valorizado\?formato=csv$/);
	});
});
