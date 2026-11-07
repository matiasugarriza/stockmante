import type { Deposito, Ubicacion } from '../schemas/depositos';

/** Nombre del depósito de la asignación; si no está cargado, un identificador estable. */
export function nombreDeposito(depositos: Deposito[], depositoId: number): string {
	return depositos.find((deposito) => deposito.id === depositoId)?.nombre ?? `Depósito #${depositoId}`;
}

/** Etiqueta de una ubicación permitida: `Depósito · Nombre` y marca de inactiva. */
export function textoUbicacion(ubicacion: Ubicacion, depositos: Deposito[]): string {
	const estado = ubicacion.activo ? '' : ' (inactiva)';
	return `${nombreDeposito(depositos, ubicacion.deposito_id)} · ${ubicacion.nombre}${estado}`;
}

/** Lista acotada para la tabla: hasta `max` etiquetas ordenadas y `+N` para el resto. */
export function textoUbicaciones(ubicaciones: Ubicacion[], depositos: Deposito[], max = 2): string {
	const textos = ubicaciones.map((ubicacion) => textoUbicacion(ubicacion, depositos)).sort();
	if (textos.length <= max) return textos.join('; ');
	return `${textos.slice(0, max).join('; ')} +${textos.length - max}`;
}
