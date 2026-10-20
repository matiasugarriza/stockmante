import { describe, expect, it } from 'vitest';
import {
	CambioPasswordSchema,
	LoginSchema,
	SesionResponseSchema
} from './auth';

describe('LoginSchema', () => {
	it('acepta credenciales válidas', () => {
		const r = LoginSchema.safeParse({
			email: 'admin@stockmante.local',
			password: 'Password1'
		});
		expect(r.success).toBe(true);
	});

	it('rechaza email mal formado', () => {
		const r = LoginSchema.safeParse({ email: 'no-es-email', password: 'x' });
		expect(r.success).toBe(false);
	});

	it('rechaza password vacía', () => {
		const r = LoginSchema.safeParse({ email: 'a@b.com', password: '' });
		expect(r.success).toBe(false);
	});
});

describe('CambioPasswordSchema', () => {
	it('acepta password que cumple la política', () => {
		const r = CambioPasswordSchema.safeParse({
			password_actual: 'Temporal1',
			password_nueva: 'Nueva1234'
		});
		expect(r.success).toBe(true);
	});

	it.each([
		['corta', 'corta1A'],
		['sin mayúscula', 'sinmayuscula1'],
		['sin dígito', 'SINNUMEROSAQUI']
	])('rechaza password %s', (_nombre, mala) => {
		const r = CambioPasswordSchema.safeParse({
			password_actual: 'x',
			password_nueva: mala
		});
		expect(r.success).toBe(false);
	});
});

describe('SesionResponseSchema', () => {
	const usuario = {
		id: 1,
		nombre: 'Admin',
		email: 'admin@stockmante.local',
		rol: 'ADMIN',
		familia_rol: 'GESTION',
		debe_cambiar_password: false
	};

	it('parsea la respuesta de GET /auth/sesion', () => {
		const r = SesionResponseSchema.safeParse({ usuario });
		expect(r.success).toBe(true);
	});

	it('rechaza rol fuera del catálogo', () => {
		const r = SesionResponseSchema.safeParse({
			usuario: { ...usuario, rol: 'SUPERVISOR' }
		});
		expect(r.success).toBe(false);
	});
});
