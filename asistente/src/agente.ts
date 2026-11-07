/**
 * Bucle del agente: el modelo decide qué tools llamar, se ejecutan y su salida vuelve al
 * modelo hasta que redacta la respuesta. El modelo se inyecta (Principio IV).
 */
import {
	AIMessage,
	HumanMessage,
	SystemMessage,
	ToolMessage,
	type AIMessageChunk,
	type BaseMessage
} from '@langchain/core/messages';
import type { StructuredToolInterface } from '@langchain/core/tools';
import { promptSistema } from './prompt.ts';
import type { MensajeHistorial, UsuarioSesion } from './tipos.ts';

/** Mínimo que el agente necesita del LLM: el real es `chat.bindTools(tools)`. */
export interface ModeloConTools {
	invoke(mensajes: BaseMessage[]): Promise<AIMessage | AIMessageChunk>;
}

export const MAX_PASOS = 6;

export interface EntradaConsulta {
	modelo: ModeloConTools;
	tools: StructuredToolInterface[];
	mensaje: string;
	historial: MensajeHistorial[];
	usuario: UsuarioSesion;
}

function texto(contenido: BaseMessage['content']): string {
	if (typeof contenido === 'string') return contenido;
	return contenido
		.map((parte) => (typeof parte === 'object' && 'text' in parte ? String(parte.text) : ''))
		.join('');
}

export async function ejecutarConsulta({ modelo, tools, mensaje, historial, usuario }: EntradaConsulta): Promise<string> {
	const mensajes: BaseMessage[] = [
		new SystemMessage(promptSistema(usuario)),
		...historial.map((m) => (m.rol === 'usuario' ? new HumanMessage(m.texto) : new AIMessage(m.texto))),
		new HumanMessage(mensaje)
	];
	const porNombre = new Map(tools.map((t) => [t.name, t]));

	for (let paso = 0; paso < MAX_PASOS; paso++) {
		const respuesta = await modelo.invoke(mensajes);
		mensajes.push(respuesta);
		const llamadas = respuesta.tool_calls ?? [];
		if (llamadas.length === 0) return texto(respuesta.content);

		for (const llamada of llamadas) {
			const herramienta = porNombre.get(llamada.name);
			let salida: string;
			if (!herramienta) {
				salida = `La tool ${llamada.name} no existe. Usá solo las tools disponibles.`;
			} else {
				try {
					salida = String(await herramienta.invoke(llamada.args));
				} catch {
					salida = `Los argumentos de ${llamada.name} no son válidos. Revisalos y volvé a intentar.`;
				}
			}
			mensajes.push(new ToolMessage({ content: salida, tool_call_id: llamada.id ?? llamada.name }));
		}
	}
	return 'No pude completar la consulta en pocos pasos. Probá con una pregunta más concreta.';
}
