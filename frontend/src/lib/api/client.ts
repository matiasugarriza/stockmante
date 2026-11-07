/**
 * Cliente HTTP central del frontend (plan.md Principio II — un solo fetch tipado).
 * Usa cookies de sesión (`credentials: 'include'`); el backend setea la cookie HttpOnly.
 */

/** Origen del backend; también lo usan los enlaces de navegación (`<a>`),
 * que no pasan por `fetch` ni por el dev server de Vite. */
export const BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3000';

/** Detalle por celda de `importacion_invalida` (016). */
export interface DetalleError {
  fila: number;
  columna: string;
  mensaje: string;
}

export interface ErrorBody {
  error: { codigo: string; mensaje: string; detalles?: DetalleError[] };
}

export class ApiError extends Error {
  readonly status: number;
  readonly codigo: string;
  /** Solo presente en `importacion_invalida`. */
  readonly detalles?: DetalleError[];

  constructor(status: number, codigo: string, mensaje: string, detalles?: DetalleError[]) {
    super(mensaje);
    this.name = 'ApiError';
    this.status = status;
    this.codigo = codigo;
    this.detalles = detalles;
  }
}

async function interpretar<T>(res: Response): Promise<T> {
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
      err?.detalles,
    );
  }

  return body as T;
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
  return interpretar<T>(res);
}

/**
 * POST multipart (016). No fija `Content-Type`: el navegador agrega el `boundary`.
 */
export async function apiFormData<T>(path: string, datos: FormData): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    credentials: 'include',
    body: datos,
  });
  return interpretar<T>(res);
}
