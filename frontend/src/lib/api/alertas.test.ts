import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import { enviarAlertas, listarAlertas } from './alertas';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const alerta = {
	producto_id: 12,
	nombre: 'Cemento 50 kg',
	sku: 'CEM-50',
	stock_actual: 4,
	stock_minimo: 10,
	faltante: 6
};

describe('cliente de alertas (011)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarAlertas arma la paginación y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [alerta], total: 1, pagina: 2, por_pagina: 10 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const pagina = await listarAlertas({ pagina: 2, porPagina: 10 });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/alertas?');
		expect(url).toContain('pagina=2');
		expect(url).toContain('por_pagina=10');
		expect(pagina.items[0].faltante).toBe(6);
	});

	it('listarAlertas sin parámetros no agrega query', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [], total: 0, pagina: 1, por_pagina: 50 })
		);
		vi.stubGlobal('fetch', fetchMock);

		await listarAlertas();

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url.endsWith('/api/alertas')).toBe(true);
	});

	it('rechaza un item sin faltante (schema)', async () => {
		const { faltante: _f, ...incompleta } = alerta;
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => jsonResponse({ items: [incompleta], total: 1, pagina: 1, por_pagina: 50 }))
		);
		await expect(listarAlertas()).rejects.toThrow();
	});

	it('propaga el 403 de una sesión OPERACION', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse({ error: { codigo: 'sin_permisos', mensaje: 'Sin permisos' } }, 403)
			)
		);
		const error = await listarAlertas().catch((e: unknown) => e);
		expect(error).toBeInstanceOf(ApiError);
		expect((error as ApiError).codigo).toBe('sin_permisos');
	});

	it('enviarAlertas hace POST y devuelve los conteos del ciclo', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ fecha: '2026-11-07', productos: 3, destinatarios: 2, enviados: 6, fallidos: 0 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const r = await enviarAlertas();

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/alertas/enviar');
		expect(init.method).toBe('POST');
		expect(r.enviados).toBe(6);
	});
});
