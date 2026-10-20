/**
 * Store de sesión (Habilidad M1-F): única fuente de verdad del estado de login.
 * El layout lo refresca al arrancar y redirige a /login si no hay sesión.
 */
import { writable } from 'svelte/store';
import { logout as apiLogout, obtenerSesion } from '../api/auth';
import type { UsuarioSesion } from '../schemas/auth';

export type EstadoSesion =
  | { estado: 'cargando' }
  | { estado: 'sin_sesion' }
  | { estado: 'activa'; usuario: UsuarioSesion };

export const sesion = writable<EstadoSesion>({ estado: 'cargando' });

/** Consulta la sesión al backend. 401 u otro fallo → sin_sesion. */
export async function refrescarSesion(): Promise<void> {
  try {
    const usuario = await obtenerSesion();
    sesion.set({ estado: 'activa', usuario });
  } catch {
    // sin sesión (401) o backend inaccesible: el login reintenta desde cero
    sesion.set({ estado: 'sin_sesion' });
  }
}

/** Cierra la sesión (204) y limpia el estado local. */
export async function cerrarSesion(): Promise<void> {
  try {
    await apiLogout();
  } finally {
    sesion.set({ estado: 'sin_sesion' });
  }
}
