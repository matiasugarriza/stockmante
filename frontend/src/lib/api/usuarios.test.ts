import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApiError } from './client';
import {
	actualizarUsuario,
	crearUsuario,
	desactivarUsuario,
	listarUsuarios,
	reactivarUsuario,
	regenerarPasswordTemporal
} from './usuarios';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

const usuario = {
	id: 7,
	nombre: 'Ana',
	email: 'ana@test.local',
	rol: 'GERENTE',
	familia_rol: 'GESTION',
	activo: true,
	password_temporal: false,
	creado_en: '2026-10-25T12:00:00Z'
};

describe('cliente de usuarios', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listarUsuarios arma el query y parsea la página', async () => {
		const fetchMock = vi.fn(async () =>
			jsonResponse({ items: [usuario], total: 1, pagina: 2, por_pagina: 50 })
		);
		vi.stubGlobal('fetch', fetchMock);

		const lista = await listarUsuarios({ activo: 'true', pagina: 2, porPagina: 50 });

		const [url] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/usuarios?activo=true&pagina=2&por_pagina=50');
		expect(lista.total).toBe(1);
		expect(lista.items[0]?.email).toBe('ana@test.local');
	});

	it('crearUsuario hace POST y devuelve el usuario creado', async () => {
		const fetchMock = vi.fn(async () => jsonResponse(usuario, 201));
		vi.stubGlobal('fetch', fetchMock);

		const creado = await crearUsuario({
			nombre: 'Ana',
			email: 'ana@test.local',
			rol: 'GERENTE'
		});

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/usuarios');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({
			nombre: 'Ana',
			email: 'ana@test.local',
			rol: 'GERENTE'
		});
		expect(creado.id).toBe(7);
	});

	it('crearUsuario propaga el 409 email_duplicado', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				jsonResponse(
					{ error: { codigo: 'email_duplicado', mensaje: 'Ya existe un usuario con ese email' } },
					409
				)
			)
		);

		await expect(
			crearUsuario({ nombre: 'Ana', email: 'ana@test.local', rol: 'GERENTE' })
		).rejects.toMatchObject({
			name: 'ApiError',
			status: 409,
			codigo: 'email_duplicado'
		} satisfies Partial<ApiError>);
	});

	it('actualizarUsuario hace PUT al id con cambios parciales', async () => {
		const fetchMock = vi.fn(async () => jsonResponse({ ...usuario, rol: 'CAJERO' }));
		vi.stubGlobal('fetch', fetchMock);

		const actualizado = await actualizarUsuario(7, { rol: 'CAJERO' });

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/usuarios/7');
		expect(init.method).toBe('PUT');
		expect(actualizado.rol).toBe('CAJERO');
	});

	it.each([
		['desactivarUsuario', desactivarUsuario, '/api/usuarios/7/desactivar'],
		['reactivarUsuario', reactivarUsuario, '/api/usuarios/7/reactivar'],
		['regenerarPasswordTemporal', regenerarPasswordTemporal, '/api/usuarios/7/password-temporal']
	])('%s hace POST y resuelve con 204', async (_nombre, accion, path) => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);

		await expect(accion(7)).resolves.toBeUndefined();

		const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain(path);
		expect(init.method).toBe('POST');
	});
});
