/** Fábrica del LLM real según `LLM_PROVIDER`. Los tests usan `pruebas/modelo-falso.ts`. */
import { ChatAnthropic } from '@langchain/anthropic';
import { ChatOpenAI } from '@langchain/openai';
import type { StructuredToolInterface } from '@langchain/core/tools';
import type { ModeloConTools } from './agente.ts';
import type { ConfigLlm } from './config.ts';

export type FabricaModelo = (tools: StructuredToolInterface[]) => ModeloConTools;

export function crearFabricaModelo(config: ConfigLlm): FabricaModelo {
	return (tools) => {
		const chat =
			config.proveedor === 'anthropic'
				? new ChatAnthropic({ model: config.modelo, apiKey: config.apiKey, maxRetries: 1 })
				: new ChatOpenAI({ model: config.modelo, apiKey: config.apiKey, maxRetries: 1 });
		const conTools = chat.bindTools(tools);
		return { invoke: (mensajes) => conTools.invoke(mensajes) };
	};
}
