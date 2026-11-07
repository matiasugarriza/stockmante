import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import {
	agregarAlHistorial,
	confirmarPropuesta,
	consultarAsistente,
	MAX_HISTORIAL
} from './asistente';
import type { AccionPropuesta, MensajeHistorial } from '../schemas/asistente';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const propuesta: AccionPropuesta = {
	tipo: 'MERMA',
	deposito_id: 1,
	deposito_nombre: 'Principal',
	producto_id: 7,
	producto_nombre: 'Yerba Suave 500 g',
	sku: 'YER-SUA-500',
	cantidad_delta: -2,
	motivo: 'MERMA',
	observacion: 'vencimiento'
};

function mensajes(n: number): MensajeHistorial[] {
	return Array.from({ length: n }, (_, i) => ({
		rol: i % 2 === 0 ? 'usuario' : 'asistente',
		texto: `m${i}`
	}));
}

describe('cliente del asistente (017)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('consultarAsistente envía mensaje e historial recortado y parsea la propuesta', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ respuesta: 'Te propongo esta merma.', accion_propuesta: propuesta }));
		vi.stubGlobal('fetch', fetchMock);

		const res = await consultarAsistente('registrá una merma', mensajes(12));

		expect(res.accion_propuesta).toEqual(propuesta);
		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/asistente\/consulta$/);
		expect(init.method).toBe('POST');
		expect(init.credentials).toBe('include');
		const body = JSON.parse(String(init.body));
		expect(body.mensaje).toBe('registrá una merma');
		expect(body.historial).toHaveLength(MAX_HISTORIAL);
		expect(body.historial[0].texto).toBe('m2');
	});

	it('acepta respuestas sin propuesta', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => jsonResponse({ respuesta: 'Hay 12.', accion_propuesta: null })));
		const res = await consultarAsistente('¿cuánto hay?', []);
		expect(res).toEqual({ respuesta: 'Hay 12.', accion_propuesta: null });
	});

	it.each([
		[401, 'no_autenticado', 'Necesitás iniciar sesión para continuar'],
		[429, 'asistente_limite_excedido', 'Superaste el límite de 20 consultas por minuto. Esperá unos segundos y volvé a intentar.'],
		[503, 'asistente_no_disponible', 'El asistente no está configurado en este entorno.'],
		[504, 'asistente_tiempo_agotado', 'El asistente tardó demasiado en responder. Probá con una pregunta más concreta.']
	])('propaga el %i con el mensaje real de la API', async (status, codigo, mensaje) => {
		vi.stubGlobal('fetch', vi.fn(async () => jsonResponse({ error: { codigo, mensaje } }, status)));
		const error = await consultarAsistente('hola', []).catch((e: unknown) => e);
		expect(error).toBeInstanceOf(ApiError);
		expect(error).toMatchObject({ status, codigo, message: mensaje });
	});

	it('rechaza una propuesta con forma inválida', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => jsonResponse({ respuesta: 'x', accion_propuesta: { ...propuesta, tipo: 'BORRAR' } }))
		);
		await expect(consultarAsistente('hola', [])).rejects.toThrow();
	});

	it('confirmarPropuesta llama a POST /api/movimientos con el formato del alta (FR-067)', async () => {
		const movimiento = {
			id: 99,
			tipo: 'MERMA',
			deposito: { id: 1, nombre: 'Principal' },
			motivo: 'MERMA',
			observacion: 'vencimiento',
			usuario: { id: 2, nombre: 'Ana' },
			creado_en: '2026-11-06T12:00:00Z',
			lineas: [
				{
					producto_id: 7,
					producto_nombre: 'Yerba Suave 500 g',
					sku: 'YER-SUA-500',
					ubicacion_id: null,
					ubicacion_nombre: null,
					cantidad_delta: -2,
					stock_resultante: 1
				}
			]
		};
		const fetchMock = vi.fn(async () => jsonResponse(movimiento, 201));
		vi.stubGlobal('fetch', fetchMock);

		const res = await confirmarPropuesta(propuesta);

		expect(res.lineas[0].stock_resultante).toBe(1);
		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/movimientos$/);
		expect(JSON.parse(String(init.body))).toEqual({
			tipo: 'MERMA',
			deposito_id: 1,
			motivo: 'MERMA',
			observacion: 'vencimiento',
			lineas: [{ producto_id: 7, cantidad_delta: -2 }]
		});
	});

	it('confirmarPropuesta propaga el stock insuficiente de la API', async () => {
		const mensaje = 'Stock insuficiente en Principal: 1 disponibles, 2 solicitadas.';
		vi.stubGlobal('fetch', vi.fn(async () => jsonResponse({ error: { codigo: 'stock_insuficiente', mensaje } }, 409)));
		await expect(confirmarPropuesta(propuesta)).rejects.toMatchObject({ status: 409, message: mensaje });
	});
});

describe('agregarAlHistorial (FR-077)', () => {
	it('agrega al final y conserva solo los últimos 10', () => {
		const resultado = agregarAlHistorial(mensajes(10), { rol: 'usuario', texto: 'nuevo' });
		expect(resultado).toHaveLength(MAX_HISTORIAL);
		expect(resultado[0].texto).toBe('m1');
		expect(resultado.at(-1)?.texto).toBe('nuevo');
	});

	it('no muta el historial original', () => {
		const original = mensajes(2);
		agregarAlHistorial(original, { rol: 'asistente', texto: 'x' });
		expect(original).toHaveLength(2);
	});
});
