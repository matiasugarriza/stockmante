/**
 * Cliente de solo lectura de la API Rust, con la cookie de sesión del usuario.
 * No expone métodos de escritura: el asistente nunca escribe (FR-067).
 */

export interface RespuestaApi {
	ok: boolean;
	status: number;
	body: unknown;
}

export class ClienteApi {
	private readonly baseUrl: string;
	private readonly cookie: string;
	private readonly fetchImpl: typeof fetch;

	constructor(baseUrl: string, cookie: string, fetchImpl: typeof fetch = fetch) {
		this.baseUrl = baseUrl.replace(/\/+$/, '');
		this.cookie = cookie;
		this.fetchImpl = fetchImpl;
	}

	async get(ruta: string, params: Record<string, string | number | boolean | undefined> = {}): Promise<RespuestaApi> {
		const url = new URL(`${this.baseUrl}${ruta}`);
		for (const [clave, valor] of Object.entries(params)) {
			if (valor !== undefined && valor !== '') url.searchParams.set(clave, String(valor));
		}
		const res = await this.fetchImpl(url, { method: 'GET', headers: { cookie: this.cookie } });
		const texto = await res.text();
		let body: unknown = null;
		try {
			body = texto ? JSON.parse(texto) : null;
		} catch {
			body = null;
		}
		return { ok: res.ok, status: res.status, body };
	}
}

/** Mensaje real de la API (`{ error: { mensaje } }`) para devolvérselo al modelo. */
export function mensajeDeError(respuesta: RespuestaApi): string {
	const mensaje = (respuesta.body as { error?: { mensaje?: string } } | null)?.error?.mensaje;
	if (respuesta.status === 401) {
		return `${mensaje ?? 'La sesión no es válida.'} No reintentes: pedile al usuario que vuelva a iniciar sesión.`;
	}
	return mensaje ?? `La API respondió ${respuesta.status}.`;
}
