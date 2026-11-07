/** Configuración por entorno del servicio de asistente (spec 017, Supuestos). */

export type Proveedor = 'anthropic' | 'openai';

export interface ConfigLlm {
	proveedor: Proveedor;
	apiKey: string;
	modelo: string;
}

export interface ConfigServicio {
	puerto: number;
	/** Base de la API Rust; las tools le pegan con la cookie del usuario. */
	apiBaseUrl: string;
	/** `null` = sin LLM: `POST /consulta` responde 503 (17.4). */
	llm: ConfigLlm | null;
	motivoSinLlm: string | null;
}

export const PUERTO_POR_DEFECTO = 3100;
export const MODELO_ANTHROPIC_POR_DEFECTO = 'claude-sonnet-5-5';

type Entorno = Record<string, string | undefined>;

function leerLlm(env: Entorno): { llm: ConfigLlm | null; motivo: string | null } {
	const proveedor = (env.LLM_PROVIDER?.trim() || 'anthropic').toLowerCase();
	const apiKey = env.LLM_API_KEY?.trim() ?? '';
	const modelo = env.LLM_MODEL?.trim() ?? '';
	if (proveedor !== 'anthropic' && proveedor !== 'openai') {
		return { llm: null, motivo: `LLM_PROVIDER debe ser anthropic u openai (recibido: ${proveedor}).` };
	}
	if (!apiKey) return { llm: null, motivo: 'Falta LLM_API_KEY.' };
	if (proveedor === 'openai' && !modelo) return { llm: null, motivo: 'Con LLM_PROVIDER=openai hay que definir LLM_MODEL.' };
	return {
		llm: { proveedor, apiKey, modelo: modelo || MODELO_ANTHROPIC_POR_DEFECTO },
		motivo: null
	};
}

export function leerConfig(env: Entorno = process.env): ConfigServicio {
	const puerto = Number.parseInt(env.ASISTENTE_PUERTO ?? '', 10);
	const { llm, motivo } = leerLlm(env);
	return {
		puerto: Number.isInteger(puerto) && puerto > 0 && puerto < 65536 ? puerto : PUERTO_POR_DEFECTO,
		apiBaseUrl: (env.API_BASE_URL?.trim() || 'http://localhost:3000').replace(/\/+$/, ''),
		llm,
		motivoSinLlm: motivo
	};
}
