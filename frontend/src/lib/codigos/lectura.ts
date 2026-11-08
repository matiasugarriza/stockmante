/**
 * 020: une `resolverLectura` con el aviso que ve el usuario. Lo usan las 5 pantallas con
 * buscador de producto para que cámara, lector y teclado se comporten igual.
 */
import { resolverLectura, type ResultadoLectura } from '../api/productos';
import { mensajeError } from '../utils/errores';
import { mensajeLectura } from './escaner';

export interface AvisoLectura {
	variante: 'advertencia' | 'error';
	texto: string;
	/** Valor leído, para reintentar sin volver a escanear (FR-155). */
	valor: string;
}

export interface ResultadoLeer {
	/** `null` si la búsqueda falló (red o servidor). */
	resultado: ResultadoLectura | null;
	aviso: AvisoLectura | null;
}

/** `veInactivos` solo para gestión (el backend responde 403 a operación). */
export async function leerProducto(valor: string, veInactivos: boolean): Promise<ResultadoLeer> {
	try {
		const resultado = await resolverLectura(valor, { incluirInactivos: veInactivos });
		const texto = mensajeLectura(resultado, valor, veInactivos);
		return { resultado, aviso: texto ? { variante: 'advertencia', texto, valor } : null };
	} catch (error) {
		const motivo = mensajeError(error, 'error desconocido').replace(/\.$/, '');
		return {
			resultado: null,
			aviso: {
				variante: 'error',
				texto: `No se pudo buscar el código ${valor}: ${motivo}. Tocá Reintentar.`,
				valor
			}
		};
	}
}
