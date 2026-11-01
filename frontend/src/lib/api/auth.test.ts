import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './client';
import { cambiarPassword, login, logout, obtenerSesion, urlInicioSso, verificarCorreo } from './auth';

describe('cliente de auth', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('login lanza ApiError 401 credenciales_invalidas', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				new Response(
					JSON.stringify({
						error: { codigo: 'credenciales_invalidas', mensaje: 'Credenciales inválidas' }
					}),
					{ status: 401, headers: { 'Content-Type': 'application/json' } }
				)
			)
		);
		await expect(
			login({ email: 'a@b.com', password: 'Password1' })
		).rejects.toMatchObject({
			name: 'ApiError',
			status: 401,
			codigo: 'credenciales_invalidas'
		} satisfies Partial<ApiError>);
	});

	it('login 204 resuelve sin cuerpo', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 204 })));
		await expect(
			login({ email: 'a@b.com', password: 'Password1' })
		).resolves.toBeUndefined();
	});

	it('logout lanza POST /auth/logout', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);
		await logout();
		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/auth/logout');
		expect(init.method).toBe('POST');
	});

	it('obtenerSesion devuelve el usuario parseado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				new Response(
					JSON.stringify({
						usuario: {
							id: 7,
							nombre: 'Ana',
							email: 'ana@test.local',
							rol: 'GERENTE',
							familia_rol: 'GESTION',
							debe_cambiar_password: true
						}
					}),
					{ status: 200, headers: { 'Content-Type': 'application/json' } }
				)
			)
		);
		const usuario = await obtenerSesion();
		expect(usuario.rol).toBe('GERENTE');
		expect(usuario.debe_cambiar_password).toBe(true);
	});

	it('obtenerSesion propaga el 401 como ApiError', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				new Response(
					JSON.stringify({ error: { codigo: 'no_autenticado', mensaje: 'Necesitás iniciar sesión para continuar' } }),
					{ status: 401, headers: { 'Content-Type': 'application/json' } }
				)
			)
		);
		await expect(obtenerSesion()).rejects.toMatchObject({ status: 401 });
	});

	it('cambiarPassword envía el body con ambas contraseñas', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);
		await cambiarPassword({ password_actual: 'Temporal1', password_nueva: 'Nueva1234' });
		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/auth/cambiar-password');
		expect(JSON.parse(init.body as string)).toEqual({
			password_actual: 'Temporal1',
			password_nueva: 'Nueva1234'
		});
	});

	// SSO: los <a> de navegación NO pasan por fetch ni por el dev server de
	// Vite (que responde 404 en /api), así que la URL debe ser absoluta al
	// backend (regresión del reporte "SSO Google no funciona").
	it('urlInicioSso devuelve una URL absoluta al backend', () => {
		const url = urlInicioSso('google');
		expect(url).toMatch(/^https?:\/\//);
		expect(url).toContain('/api/auth/sso/google/iniciar');
		expect(urlInicioSso('microsoft')).toContain('/api/auth/sso/microsoft/iniciar');
	});

	it('verificarCorreo hace GET con el token codificado', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 200 }));
		vi.stubGlobal('fetch', fetchMock);
		await verificarCorreo('tok 123/abc');
		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/auth/verificar-correo?token=tok%20123%2Fabc');
		expect(init.method ?? 'GET').toBe('GET');
	});
});
