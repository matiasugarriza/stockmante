/**
 * HTTP del servicio (solo lo llama el backend Rust):
 * - `POST /consulta` con `{ mensaje, historial, usuario }` y la cookie de sesión en
 *   `x-stockmante-cookie`.
 * - `GET /salud`.
 * Un log por consulta con metadatos y nunca texto (FR-078).
 */
import type { IncomingMessage, RequestListener, ServerResponse } from 'node:http';
import { z } from 'zod';
import { ejecutarConsulta, type ModeloConTools } from './agente.ts';
import { ClienteApi } from './api.ts';
import { crearTools, type RegistroConsulta } from './tools.ts';
import type { RespuestaConsulta } from './tipos.ts';
import type { StructuredToolInterface } from '@langchain/core/tools';

export const HEADER_COOKIE = 'x-stockmante-cookie';
const MAX_CUERPO_BYTES = 64 * 1024;

export interface LineaLog {
	evento: 'consulta';
	usuario_id: number | null;
	duracion_ms: number;
	tools: string[];
	hubo_propuesta: boolean;
	codigo: number;
}

export interface DependenciasServidor {
	apiBaseUrl: string;
	fetch: typeof fetch;
	/** `null` = sin LLM configurado: responde 503 (17.4). */
	crearModelo: ((tools: StructuredToolInterface[]) => ModeloConTools) | null;
	log: (linea: LineaLog) => void;
}

const PedidoSchema = z.object({
	mensaje: z.string().trim().min(1).max(2000),
	historial: z
		.array(z.object({ rol: z.enum(['usuario', 'asistente']), texto: z.string().max(4000) }))
		.max(10)
		.default([]),
	usuario: z.object({
		id: z.number().int(),
		nombre: z.string(),
		familia_rol: z.enum(['GESTION', 'OPERACION'])
	})
});

class ErrorHttp extends Error {
	readonly status: number;
	readonly codigo: string;
	constructor(status: number, codigo: string, mensaje: string) {
		super(mensaje);
		this.status = status;
		this.codigo = codigo;
	}
}

function responder(res: ServerResponse, status: number, body: unknown) {
	res.writeHead(status, { 'content-type': 'application/json; charset=utf-8' });
	res.end(JSON.stringify(body));
}

async function leerJson(req: IncomingMessage): Promise<unknown> {
	const partes: Buffer[] = [];
	let total = 0;
	for await (const parte of req) {
		total += (parte as Buffer).length;
		if (total > MAX_CUERPO_BYTES) throw new ErrorHttp(413, 'validacion', 'El pedido es demasiado grande.');
		partes.push(parte as Buffer);
	}
	try {
		return JSON.parse(Buffer.concat(partes).toString('utf8'));
	} catch {
		throw new ErrorHttp(422, 'validacion', 'El cuerpo debe ser JSON válido.');
	}
}

async function atenderConsulta(
	req: IncomingMessage,
	deps: DependenciasServidor,
	registro: RegistroConsulta,
	alIdentificar: (usuarioId: number) => void
): Promise<RespuestaConsulta> {
	const cookie = req.headers[HEADER_COOKIE];
	if (typeof cookie !== 'string' || !cookie.trim()) {
		throw new ErrorHttp(401, 'no_autenticado', 'Falta la sesión del usuario.');
	}
	const parseado = PedidoSchema.safeParse(await leerJson(req));
	if (!parseado.success) throw new ErrorHttp(422, 'validacion', 'El pedido al asistente no es válido.');
	const pedido = parseado.data;
	alIdentificar(pedido.usuario.id);
	if (!deps.crearModelo) {
		throw new ErrorHttp(503, 'asistente_no_disponible', 'El servicio de asistente no tiene un LLM configurado.');
	}
	const api = new ClienteApi(deps.apiBaseUrl, cookie, deps.fetch);
	const tools = crearTools(api, pedido.usuario, registro);
	const respuesta = await ejecutarConsulta({
		modelo: deps.crearModelo(tools),
		tools,
		mensaje: pedido.mensaje,
		historial: pedido.historial,
		usuario: pedido.usuario
	});
	return { respuesta, accion_propuesta: registro.propuesta, tools: registro.tools };
}

export function crearManejador(deps: DependenciasServidor): RequestListener {
	return async (req, res) => {
		const ruta = new URL(req.url ?? '/', 'http://local').pathname;
		if (req.method === 'GET' && ruta === '/salud') {
			responder(res, 200, { estado: 'ok', llm_configurado: deps.crearModelo !== null });
			return;
		}
		if (req.method !== 'POST' || ruta !== '/consulta') {
			responder(res, 404, { error: { codigo: 'recurso_no_encontrado', mensaje: 'Ruta inexistente.' } });
			return;
		}

		const inicio = performance.now();
		const registro: RegistroConsulta = { tools: [], propuesta: null };
		let usuarioId: number | null = null;
		let codigo = 200;
		try {
			const respuesta = await atenderConsulta(req, deps, registro, (id) => (usuarioId = id));
			responder(res, 200, respuesta);
		} catch (error) {
			if (error instanceof ErrorHttp) {
				codigo = error.status;
				responder(res, error.status, { error: { codigo: error.codigo, mensaje: error.message } });
			} else {
				// Falla del LLM o de red: el backend lo traduce a 503 "no responde". El detalle
				// no se loguea porque podría incluir texto de la conversación (FR-078).
				codigo = 500;
				responder(res, 500, { error: { codigo: 'interno', mensaje: 'El asistente falló al procesar la consulta.' } });
			}
		} finally {
			deps.log({
				evento: 'consulta',
				usuario_id: usuarioId,
				duracion_ms: Math.round(performance.now() - inicio),
				tools: [...registro.tools],
				hubo_propuesta: registro.propuesta !== null,
				codigo
			});
		}
	};
}
