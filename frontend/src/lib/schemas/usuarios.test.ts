import { describe, expect, it } from 'vitest';
import {
	ActualizarUsuarioSchema,
	CrearUsuarioSchema,
	ListaUsuariosSchema,
	UsuarioSchema
} from './usuarios';

const usuario = {
	id: 3,
	nombre: 'Ana Gómez',
	email: 'ana@test.local',
	rol: 'GERENTE',
	familia_rol: 'GESTION',
	activo: true,
	password_temporal: false,
	creado_en: '2026-10-25T12:00:00Z'
};

describe('UsuarioSchema', () => {
	it('parsea el item de GET /usuarios', () => {
		const r = UsuarioSchema.safeParse(usuario);
		expect(r.success).toBe(true);
	});

	it('rechaza un rol fuera del catálogo', () => {
		const r = UsuarioSchema.safeParse({ ...usuario, rol: 'SUPERVISOR' });
		expect(r.success).toBe(false);
	});

	it('exige el flag de contraseña temporal', () => {
		const { password_temporal: _omitido, ...sinFlag } = usuario;
		const r = UsuarioSchema.safeParse(sinFlag);
		expect(r.success).toBe(false);
	});
});

describe('ListaUsuariosSchema', () => {
	it('parsea la página completa', () => {
		const r = ListaUsuariosSchema.safeParse({
			items: [usuario],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(r.success).toBe(true);
		if (r.success) expect(r.data.items[0]?.email).toBe('ana@test.local');
	});

	it('rechaza items que no son usuarios', () => {
		const r = ListaUsuariosSchema.safeParse({
			items: [{ id: 1 }],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(r.success).toBe(false);
	});
});

describe('CrearUsuarioSchema', () => {
	it('acepta un alta válida y recorta el nombre', () => {
		const r = CrearUsuarioSchema.safeParse({
			nombre: '  Nueva Persona  ',
			email: 'nueva@test.local',
			rol: 'OPERARIO'
		});
		expect(r.success).toBe(true);
		if (r.success) expect(r.data.nombre).toBe('Nueva Persona');
	});

	it.each([
		['email mal formado', { nombre: 'X', email: 'no-es-email', rol: 'OPERARIO' }],
		['nombre vacío', { nombre: '   ', email: 'x@test.local', rol: 'OPERARIO' }],
		['rol inválido', { nombre: 'X', email: 'x@test.local', rol: 'SUPERVISOR' }]
	])('rechaza %s', (_caso, datos) => {
		const r = CrearUsuarioSchema.safeParse(datos);
		expect(r.success).toBe(false);
	});
});

describe('ActualizarUsuarioSchema', () => {
	it('acepta cambios parciales', () => {
		const r = ActualizarUsuarioSchema.safeParse({ rol: 'CAJERO' });
		expect(r.success).toBe(true);
	});

	it('rechaza un email mal formado aunque sea parcial', () => {
		const r = ActualizarUsuarioSchema.safeParse({ email: 'roto' });
		expect(r.success).toBe(false);
	});
});
