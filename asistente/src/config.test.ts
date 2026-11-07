import { describe, expect, it } from 'vitest';
import { leerConfig, MODELO_ANTHROPIC_POR_DEFECTO } from './config.ts';

describe('leerConfig', () => {
	it('sin LLM_API_KEY el LLM queda deshabilitado (17.4)', () => {
		const config = leerConfig({ LLM_PROVIDER: 'anthropic' });
		expect(config.llm).toBeNull();
		expect(config.motivoSinLlm).toContain('LLM_API_KEY');
	});

	it('usa defaults de puerto, API y modelo de Anthropic', () => {
		const config = leerConfig({ LLM_API_KEY: 'clave' });
		expect(config.puerto).toBe(3100);
		expect(config.apiBaseUrl).toBe('http://localhost:3000');
		expect(config.llm).toEqual({ proveedor: 'anthropic', apiKey: 'clave', modelo: MODELO_ANTHROPIC_POR_DEFECTO });
	});

	it('OpenAI exige LLM_MODEL explícito', () => {
		expect(leerConfig({ LLM_PROVIDER: 'openai', LLM_API_KEY: 'k' }).llm).toBeNull();
		expect(leerConfig({ LLM_PROVIDER: 'openai', LLM_API_KEY: 'k', LLM_MODEL: 'm' }).llm).toEqual({
			proveedor: 'openai',
			apiKey: 'k',
			modelo: 'm'
		});
	});

	it('un proveedor desconocido deshabilita el LLM con un motivo claro', () => {
		const config = leerConfig({ LLM_PROVIDER: 'otro', LLM_API_KEY: 'k' });
		expect(config.llm).toBeNull();
		expect(config.motivoSinLlm).toContain('LLM_PROVIDER');
	});

	it('un puerto inválido cae al default', () => {
		expect(leerConfig({ ASISTENTE_PUERTO: 'abc' }).puerto).toBe(3100);
		expect(leerConfig({ ASISTENTE_PUERTO: '4000', API_BASE_URL: 'http://api:3000/' }).apiBaseUrl).toBe('http://api:3000');
	});
});
