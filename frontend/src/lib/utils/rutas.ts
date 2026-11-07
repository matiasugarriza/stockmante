/**
 * Rutas alcanzadas sin sesión — FR-070 (spec 001): el formulario de registro DEBE ser
 * "accesible públicamente". El guard del $layout sólo patea a /login desde rutas no públicas.
 */

export const RUTAS_PUBLICAS = ['/login', '/registro', '/verificar-correo'] as const;

export function esRutaPublica(pathname: string): boolean {
  const limpia = pathname.length > 1 && pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
  return RUTAS_PUBLICAS.some((ruta) => limpia === ruta || limpia.startsWith(`${ruta}/`));
}