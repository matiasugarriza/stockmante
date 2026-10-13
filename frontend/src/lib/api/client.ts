/**
 * Cliente HTTP central del frontend (plan.md Principio II — un solo fetch tipado).
 * Usa cookies de sesión (`credentials: 'include'`); el backend setea la cookie HttpOnly.
 */

const BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3000';

export interface ErrorBody {
  error: { codigo: string; mensaje: string };
}

export class ApiError extends Error {
  readonly status: number;
  readonly codigo: string;

  constructor(status: number, codigo: string, mensaje: string) {
    super(mensaje);
    this.name = 'ApiError';
    this.status = status;
    this.codigo = codigo;
  }
}

export async function api<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...(init.headers as Record<string, string> | undefined),
    },
    ...init,
  });

  if (res.status === 204) {
    return undefined as T;
  }

  const text = await res.text();
  let body: unknown = undefined;
  if (text) {
    try {
      body = JSON.parse(text);
    } catch {
      body = undefined;
    }
  }

  if (!res.ok) {
    const err = (body as ErrorBody | undefined)?.error;
    throw new ApiError(
      res.status,
      err?.codigo ?? 'desconocido',
      err?.mensaje ?? `Error ${res.status}`,
    );
  }

  return body as T;
}
