/**
 * Reglas puras del escáner de códigos (020, research R2/R9). La cámara vive en
 * `EscanerCamara.svelte`; acá solo se normaliza, se clasifica y se elige.
 */

/** Formatos que la cámara reconoce (FR-157), con los nombres de la API `BarcodeDetector`. */
export const FORMATOS_LECTURA = [
	'ean_13',
	'ean_8',
	'upc_a',
	'code_128',
	'code_39',
	'qr_code'
] as const;

export type FormatoCamara = (typeof FORMATOS_LECTURA)[number];
/** `teclado` = lector tipo pistola o ingreso manual. */
export type FormatoLectura = FormatoCamara | 'teclado';

export interface Lectura {
	valor: string;
	formato: FormatoLectura;
}

/** Subconjunto de `DetectedBarcode` que usa el escáner. */
export interface Deteccion {
	rawValue: string;
	format: string;
	boundingBox: { width: number; height: number };
}

export type ErrorCamara =
	| 'inseguro'
	| 'sin_soporte'
	| 'permiso_denegado'
	| 'sin_camara'
	| 'ocupada'
	| 'desconocido';

export interface EntornoCamara {
	contextoSeguro: boolean;
	tieneMediaDevices: boolean;
}

/** Qué pasó, por qué y qué hacer (P-III); contrato en `contracts/ui.md`. */
export const MENSAJES_CAMARA: Record<ErrorCamara, string> = {
	inseguro:
		'La cámara no está disponible: la página no se abrió por HTTPS. Abrí Stockmante con https:// o usá el lector o la búsqueda manual.',
	sin_soporte:
		'Este navegador no permite usar la cámara. Usá el lector físico o escribí el código.',
	permiso_denegado:
		'No diste permiso para usar la cámara. Habilitalo en los ajustes del navegador o escribí el código.',
	sin_camara:
		'No encontramos una cámara en este dispositivo. Usá el lector físico o escribí el código.',
	ocupada: 'La cámara está en uso por otra aplicación. Cerrala y tocá Reintentar.',
	desconocido: 'No se pudo iniciar la cámara. Tocá Reintentar o escribí el código.'
};

// Espacios y caracteres de control (incluye el separador GS `\x1d` de Code 128/GS1).
const NO_IMPRIMIBLES = /[\s\u0000-\u001f\u007f]+/g;
const PATRON_CODIGO = /^[0-9A-Za-z]{8,64}$/;

/** Limpia lo que envía la cámara o el lector; vacío → `null` (FR-137). */
export function normalizarLectura(valor: string): string | null {
	const limpio = valor.replace(NO_IMPRIMIBLES, '');
	return limpio.length > 0 ? limpio : null;
}

/** Forma de un código de barras admitido por el backend (003): 8–64 alfanuméricos. */
export function esProbableCodigo(valor: string): boolean {
	return PATRON_CODIGO.test(valor);
}

/** Traduce el entorno o el error de `getUserMedia` a un motivo accionable (FR-151). */
export function clasificarErrorCamara(error: unknown, entorno: EntornoCamara): ErrorCamara {
	if (!entorno.contextoSeguro) return 'inseguro';
	if (!entorno.tieneMediaDevices) return 'sin_soporte';
	const nombre = error instanceof Error ? error.name : '';
	switch (nombre) {
		case 'NotAllowedError':
		case 'SecurityError':
			return 'permiso_denegado';
		case 'NotFoundError':
		case 'OverconstrainedError':
			return 'sin_camara';
		case 'NotReadableError':
			return 'ocupada';
		default:
			return 'desconocido';
	}
}

/** Una sola lectura por cuadro: la detección válida de mayor área. */
export function elegirLectura(detecciones: readonly Deteccion[]): Lectura | null {
	let mejor: { lectura: Lectura; area: number } | null = null;
	for (const deteccion of detecciones) {
		const valor = normalizarLectura(deteccion.rawValue);
		if (valor === null) continue;
		const area = deteccion.boundingBox.width * deteccion.boundingBox.height;
		if (mejor === null || area > mejor.area) {
			mejor = { lectura: { valor, formato: deteccion.format as FormatoCamara }, area };
		}
	}
	return mejor?.lectura ?? null;
}

/** Forma mínima del resultado de `resolverLectura` que necesita el mensaje. */
export type ResultadoParaMensaje =
	| { estado: 'encontrado' | 'inactivo'; producto: { nombre: string; sku: string } }
	| { estado: 'no_encontrado' };

/**
 * Aviso para el usuario tras una lectura (contracts/ui.md); `null` si no hace falta.
 * `veInactivos` es falso para operación: el backend no le muestra productos dados de baja,
 * así que solo puede afirmar que no hay un producto *activo* con ese código.
 */
export function mensajeLectura(
	resultado: ResultadoParaMensaje,
	valor: string,
	veInactivos: boolean
): string | null {
	switch (resultado.estado) {
		case 'encontrado':
			return null;
		case 'inactivo':
			return `El producto ${resultado.producto.nombre} (SKU ${resultado.producto.sku}) está dado de baja; no se puede agregar.`;
		case 'no_encontrado':
			return `No se encontró un producto${veInactivos ? '' : ' activo'} con el código ${valor}. Buscalo por nombre o SKU.`;
	}
}
