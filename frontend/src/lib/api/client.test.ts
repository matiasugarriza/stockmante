import { afterEach, describe, expect, it, vi } from 'vitest';
import { api, ApiError } from './client';

describe('api client', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('lanza ApiError con codigo y mensaje desde el body de error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(
          JSON.stringify({
            error: {
              codigo: 'credenciales_invalidas',
              mensaje: 'Credenciales inválidas',
            },
          }),
          { status: 401, headers: { 'Content-Type': 'application/json' } },
        ),
      ),
    );

    await expect(api('/api/auth/login', { method: 'POST' })).rejects.toMatchObject({
      name: 'ApiError',
      status: 401,
      codigo: 'credenciales_invalidas',
      message: 'Credenciales inválidas',
    } satisfies Partial<ApiError>);
  });

  it('devuelve undefined en 204', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response(null, { status: 204 })),
    );
    await expect(api('/api/auth/logout', { method: 'POST' })).resolves.toBeUndefined();
  });

  it('usa credentials include para cookies de sesión', async () => {
    const fetchMock = vi.fn(
      async () => new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    vi.stubGlobal('fetch', fetchMock);
    await api('/api/auth/sesion');
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toContain('/api/auth/sesion');
    expect(init.credentials).toBe('include');
  });
});
