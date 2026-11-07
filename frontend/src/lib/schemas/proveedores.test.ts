import { describe, expect, it } from 'vitest';
import {
	CrearProveedorSchema,
	ListaProveedoresSchema,
	ProveedorSchema
} from './proveedores';

const proveedor = {
	id: 7,
	nombre: 'Distribuidora Norte',
	cuit: '20123456789',
	email: null,
	telefono: null,
	direccion: null,
	activo: true
};

describe('schemas de proveedores', () => {
	it('parsea un proveedor con campos opcionales nulos', () => {
		const parseado = ProveedorSchema.parse(proveedor);
		expect(parseado.nombre).toBe('Distribuidora Norte');
		expect(parseado.email).toBeNull();
	});

	it('parsea la lista paginada', () => {
		const lista = ListaProveedoresSchema.parse({
			items: [proveedor],
			total: 1,
			pagina: 1,
			por_pagina: 20
		});
		expect(lista.items).toHaveLength(1);
		expect(lista.total).toBe(1);
	});

	it('rechaza un proveedor con activo no booleano', () => {
		expect(() => ProveedorSchema.parse({ ...proveedor, activo: 'Sí' })).toThrow();
	});

	it('normaliza el alta: recorta el nombre y acepta CUIT vacío', () => {
		const datos = CrearProveedorSchema.parse({
			nombre: '  Distribuidora Norte  ',
			cuit: '',
			email: '',
			telefono: '',
			direccion: ''
		});
		expect(datos.nombre).toBe('Distribuidora Norte');
		expect(datos.cuit).toBe('');
	});

	it('rechaza nombre vacío y CUIT con longitud incorrecta', () => {
		expect(() => CrearProveedorSchema.parse({ nombre: '   ' })).toThrow();
		expect(() => CrearProveedorSchema.parse({ nombre: 'Proveedor', cuit: '123' })).toThrow(
			/11 dígitos/
		);
	});
});