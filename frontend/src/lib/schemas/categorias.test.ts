import { describe, expect, it } from 'vitest';
import {
	ActualizarCategoriaSchema,
	CategoriaSchema,
	CrearCategoriaSchema,
	ListaCategoriasSchema
} from './categorias';

const categoria = {
	id: 4,
	nombre: 'Almacén',
	descripcion: 'Productos secos',
	activo: true
};

describe('CategoriaSchema', () => {
	it('parsea el item de GET /categorias', () => {
		expect(CategoriaSchema.safeParse(categoria).success).toBe(true);
	});

	it('acepta descripción nula', () => {
		expect(CategoriaSchema.safeParse({ ...categoria, descripcion: null }).success).toBe(true);
	});

	it('rechaza un item sin estado', () => {
		const { activo: _omitido, ...sinEstado } = categoria;
		expect(CategoriaSchema.safeParse(sinEstado).success).toBe(false);
	});
});

describe('ListaCategoriasSchema', () => {
	it('parsea una lista plana', () => {
		const r = ListaCategoriasSchema.safeParse([categoria, { ...categoria, id: 5 }]);
		expect(r.success).toBe(true);
		if (r.success) expect(r.data).toHaveLength(2);
	});

	it('rechaza items incompletos', () => {
		expect(ListaCategoriasSchema.safeParse([{ id: 1 }]).success).toBe(false);
	});
});

describe('CrearCategoriaSchema', () => {
	it('recorta el nombre y la descripción', () => {
		const r = CrearCategoriaSchema.safeParse({
			nombre: '  Bebidas  ',
			descripcion: '  Con alcohol  '
		});
		expect(r.success).toBe(true);
		if (r.success) {
			expect(r.data.nombre).toBe('Bebidas');
			expect(r.data.descripcion).toBe('Con alcohol');
		}
	});

	it('acepta descripción ausente, nula o vacía', () => {
		for (const descripcion of [undefined, null, '']) {
			expect(CrearCategoriaSchema.safeParse({ nombre: 'Bebidas', descripcion }).success).toBe(
				true
			);
		}
	});

	it.each([
		['nombre vacío', { nombre: '   ' }],
		['nombre de 81', { nombre: 'a'.repeat(81) }],
		['descripción de 256', { nombre: 'Bebidas', descripcion: 'd'.repeat(256) }]
	])('rechaza %s', (_caso, datos) => {
		expect(CrearCategoriaSchema.safeParse(datos).success).toBe(false);
	});
});

describe('ActualizarCategoriaSchema', () => {
	it('acepta cambios parciales', () => {
		const r = ActualizarCategoriaSchema.safeParse({ descripcion: '' });
		expect(r.success).toBe(true);
	});

	it('rechaza un cuerpo vacío', () => {
		expect(ActualizarCategoriaSchema.safeParse({}).success).toBe(false);
	});

	it('rechaza `activo`: el estado se cambia por acciones', () => {
		expect(ActualizarCategoriaSchema.safeParse({ activo: false }).success).toBe(false);
	});
});
