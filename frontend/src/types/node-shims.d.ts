/**
 * Tipos mínimos de Node para las pruebas que leen archivos del repositorio
 * (guard de contraste). Evita agregar `@types/node` como dependencia.
 */
declare module 'node:fs' {
	export function readFileSync(ruta: string, codificacion: 'utf8'): string;
}

declare module 'node:url' {
	export function fileURLToPath(url: URL): string;
}
