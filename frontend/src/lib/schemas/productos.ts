import { z } from 'zod';

export const UnidadMedidaSchema = z.enum(['UNIDAD', 'KG', 'METRO', 'CAJA']);
export type UnidadMedida = z.infer<typeof UnidadMedidaSchema>;

export const CategoriaBreveSchema = z.object({
	id: z.number().int(),
	nombre: z.string()
});
export type CategoriaBreve = z.infer<typeof CategoriaBreveSchema>;

/** Item de GET /productos (contracts/api.md). */
export const ProductoSchema = z.object({
	id: z.number().int(),
	nombre: z.string(),
	sku: z.string(),
	codigo_barras: z.string().nullable(),
	categoria: CategoriaBreveSchema,
	unidad_medida: UnidadMedidaSchema,
	costo: z.number(),
	precio_venta: z.number(),
	stock_minimo: z.number().int(),
	activo: z.boolean()
});
export type Producto = z.infer<typeof ProductoSchema>;

export const ListaProductosSchema = z.object({
	items: z.array(ProductoSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaProductos = z.infer<typeof ListaProductosSchema>;

/** Búsqueda de caja (004): lista acotada, sin paginación. */
export const ListaBusquedaSchema = z.object({
	items: z.array(ProductoSchema)
});
export type ListaBusqueda = z.infer<typeof ListaBusquedaSchema>;

const MontoSchema = z
	.number()
	.nonnegative('El monto no puede ser negativo')
	.multipleOf(0.01, 'Máximo 2 decimales')
	.lt(10_000_000_000, 'El monto debe ser menor a 10.000.000.000');

const CodigoBarrasSchema = z
	.union([
		z.literal(''),
		z
			.string()
			.regex(
				/^[0-9A-Za-z]{8,64}$/,
				'El código de barras debe tener entre 8 y 64 caracteres alfanuméricos'
			)
	])
	.nullish();

/** Alta: espejo de contracts/validation.md; el backend sigue siendo la autoridad. */
export const CrearProductoSchema = z.object({
	nombre: z.string().trim().min(1, 'Ingresá el nombre').max(160, 'Máximo 160 caracteres'),
	sku: z
		.string()
		.trim()
		.toUpperCase()
		.regex(
			/^[A-Z0-9][A-Z0-9-]{2,29}$/,
			'El SKU debe tener entre 3 y 30 caracteres en mayúsculas, dígitos y guiones'
		),
	codigo_barras: CodigoBarrasSchema,
	categoria_id: z.number().int().positive('Elegí una categoría'),
	unidad_medida: UnidadMedidaSchema,
	costo: MontoSchema,
	precio_venta: MontoSchema,
	stock_minimo: z
		.number()
		.int('El stock mínimo debe ser un entero')
		.nonnegative('El stock mínimo no puede ser negativo')
});
export type CrearProductoInput = z.infer<typeof CrearProductoSchema>;

/** Edición parcial: al menos un campo y nunca `activo` (se cambia por acciones). */
export const ActualizarProductoSchema = CrearProductoSchema.partial().refine(
	(datos) => Object.keys(datos).length > 0,
	{ message: 'Indicá al menos un campo para actualizar.' }
);
export type ActualizarProductoInput = z.infer<typeof ActualizarProductoSchema>;
