import { afterEach, describe, expect, it } from 'vitest';
import { createServer, type Server } from 'node:http';
import type { AddressInfo } from 'node:net';
import { crearManejador, type DependenciasServidor, type LineaLog } from './servidor.ts';
import { crearFetchFalso, rutasCatalogo } from './pruebas/api-falsa.ts';
import { modeloConGuion, type PasoGuion } from './pruebas/modelo-falso.ts';

const COOKIE = 'stockmante_sesion=33333333-3333-3333-3333-333333333333';
let servidor: Server | null = null;

afterEach(async () => {
	await new Promise<void>((ok) => (servidor ? servidor.close(() => ok()) : ok()));
	servidor = null;
});

async function levantar(guion: PasoGuion[] | null) {
	const { fetch, llamadas } = crearFetchFalso(rutasCatalogo());
	const logs: LineaLog[] = [];
	const deps: DependenciasServidor = {
		apiBaseUrl: 'http://api.test',
		fetch,
		crearModelo: guion ? () => modeloConGuion(guion) : null,
		log: (linea) => logs.push(linea)
	};
	servidor = createServer(crearManejador(deps));
	await new Promise<void>((ok) => servidor?.listen(0, '127.0.0.1', ok));
	const { port } = servidor.address() as AddressInfo;
	const pedir = (body: unknown, headers: Record<string, string> = { 'x-stockmante-cookie': COOKIE }) =>
		fetch === undefined
			? Promise.reject(new Error('sin fetch'))
			: globalThis.fetch(`http://127.0.0.1:${port}/consulta`, {
					method: 'POST',
					headers: { 'content-type': 'application/json', ...headers },
					body: JSON.stringify(body)
				});
	return { pedir, logs, llamadas, port };
}

const PEDIDO = {
	mensaje: 'registrá una merma de 2 unidades de yerba suave 1 kg en Principal por vencimiento',
	historial: [{ rol: 'usuario', texto: 'SECRETO-HISTORIAL' }],
	usuario: { id: 10, nombre: 'Ana', familia_rol: 'OPERACION' }
};

describe('POST /consulta', () => {
	it('sin LLM configurado responde 503 asistente_no_disponible (17.4)', async () => {
		const { pedir } = await levantar(null);
		const res = await pedir(PEDIDO);
		expect(res.status).toBe(503);
		expect((await res.json()).error.codigo).toBe('asistente_no_disponible');
	});

	it('devuelve respuesta, propuesta y tools usadas', async () => {
		const { pedir, llamadas } = await levantar([
			{
				tools: [
					{
						name: 'proponer_movimiento',
						args: { tipo: 'MERMA', producto: 'Yerba Suave 1 kg', deposito: 'Principal', cantidad: 2, observacion: 'vencimiento' }
					}
				]
			},
			{ texto: 'Te propongo registrar la merma; confirmala para ejecutarla.' }
		]);
		const res = await pedir(PEDIDO);
		expect(res.status).toBe(200);
		const body = await res.json();
		expect(body.respuesta).toBe('Te propongo registrar la merma; confirmala para ejecutarla.');
		expect(body.accion_propuesta).toMatchObject({ tipo: 'MERMA', producto_id: 1, deposito_id: 1, cantidad_delta: -2 });
		expect(body.tools).toEqual(['proponer_movimiento']);
		expect(llamadas.every((l) => l.cookie === COOKIE && l.metodo === 'GET')).toBe(true);
	});

	it('sin la cookie reenviada responde 401 y no llama a la API', async () => {
		const { pedir, llamadas } = await levantar([{ texto: 'hola' }]);
		const res = await pedir(PEDIDO, {});
		expect(res.status).toBe(401);
		expect(llamadas).toHaveLength(0);
	});

	it('valida el pedido (422)', async () => {
		const { pedir } = await levantar([{ texto: 'hola' }]);
		const res = await pedir({ mensaje: '', usuario: PEDIDO.usuario });
		expect(res.status).toBe(422);
	});

	it('el log tiene solo metadatos, nunca texto (FR-078)', async () => {
		const { pedir, logs } = await levantar([
			{ tools: [{ name: 'consultar_stock', args: { producto: 'yerba suave 1 kg' } }] },
			{ texto: 'SECRETO-RESPUESTA' }
		]);
		await pedir({ ...PEDIDO, mensaje: 'SECRETO-MENSAJE yerba' });
		expect(logs).toHaveLength(1);
		expect(Object.keys(logs[0]).sort()).toEqual(['codigo', 'duracion_ms', 'evento', 'hubo_propuesta', 'tools', 'usuario_id']);
		expect(logs[0]).toMatchObject({ evento: 'consulta', usuario_id: 10, codigo: 200, tools: ['consultar_stock'], hubo_propuesta: false });
		const serializado = JSON.stringify(logs);
		expect(serializado).not.toContain('SECRETO');
		expect(serializado).not.toContain('Yerba');
	});

	it('GET /salud responde 200', async () => {
		const { port } = await levantar(null);
		const res = await globalThis.fetch(`http://127.0.0.1:${port}/salud`);
		expect(res.status).toBe(200);
	});
});
