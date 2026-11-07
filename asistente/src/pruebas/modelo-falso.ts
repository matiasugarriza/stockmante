/**
 * LLM falso y determinista: cada `invoke` devuelve el siguiente mensaje del guion.
 * Reemplaza al proveedor real en los tests (Principio IV: sin red ni aleatoriedad).
 */
import { AIMessage, type BaseMessage } from '@langchain/core/messages';
import type { ModeloConTools } from '../agente.ts';

export interface PasoGuion {
	texto?: string;
	tools?: Array<{ name: string; args: Record<string, unknown> }>;
}

export function modeloConGuion(guion: PasoGuion[]): ModeloConTools & { vistos: BaseMessage[][] } {
	let paso = 0;
	const vistos: BaseMessage[][] = [];
	return {
		vistos,
		async invoke(mensajes: BaseMessage[]) {
			vistos.push([...mensajes]);
			const actual = guion[Math.min(paso, guion.length - 1)];
			paso += 1;
			return new AIMessage({
				content: actual.texto ?? '',
				tool_calls: (actual.tools ?? []).map((t, i) => ({
					id: `llamada-${paso}-${i}`,
					name: t.name,
					args: t.args,
					type: 'tool_call' as const
				}))
			});
		}
	};
}
