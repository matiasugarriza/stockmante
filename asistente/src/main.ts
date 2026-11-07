/** Arranque del servicio de asistente: `npm start` (Node 24 ejecuta TypeScript nativo). */
import { createServer } from 'node:http';
import { leerConfig } from './config.ts';
import { crearFabricaModelo } from './modelo.ts';
import { crearManejador } from './servidor.ts';

const config = leerConfig();
if (!config.llm) {
	console.warn(`[asistente] LLM deshabilitado: ${config.motivoSinLlm} POST /consulta responderá 503.`);
}

const servidor = createServer(
	crearManejador({
		apiBaseUrl: config.apiBaseUrl,
		fetch,
		crearModelo: config.llm ? crearFabricaModelo(config.llm) : null,
		log: (linea) => console.log(JSON.stringify(linea))
	})
);

// Solo loopback: lo consume el backend en la misma máquina, no el navegador.
servidor.listen(config.puerto, '127.0.0.1', () => {
	console.log(`[asistente] escuchando en http://127.0.0.1:${config.puerto} (API: ${config.apiBaseUrl})`);
});
