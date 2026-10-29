/**
 * Cliente de autenticación (Habilidad M1-F): las 4 rutas de contracts/api.md.
 * El backend setea/borra la cookie HttpOnly; acá solo pedimos con credentials.
 */
import { api } from './client';
import {
  SesionResponseSchema,
  type CambioPasswordInput,
  type LoginInput,
  type UsuarioSesion
} from '../schemas/auth';

export async function login(datos: LoginInput): Promise<void> {
  await api('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify(datos)
  });
}

export async function logout(): Promise<void> {
  await api('/api/auth/logout', { method: 'POST' });
}

/** Usuario de la sesión vigente; lanza ApiError 401 si no hay sesión. */
export async function obtenerSesion(): Promise<UsuarioSesion> {
  const body = await api<unknown>('/api/auth/sesion');
  return SesionResponseSchema.parse(body).usuario;
}

export async function cambiarPassword(datos: CambioPasswordInput): Promise<void> {
	await api('/api/auth/cambiar-password', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
}

export async function registro(datos: { nombre: string; email: string; password: string }): Promise<void> {
	await api('/api/auth/registro', {
		method: 'POST',
		body: JSON.stringify(datos)
	});
}
