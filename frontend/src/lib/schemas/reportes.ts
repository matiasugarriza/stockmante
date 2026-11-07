import { z } from 'zod';

/** Reporte de stock valorizado (014); `total` es el valor monetario del inventario. */
export const StockValorizadoItemSchema = z.object({
	producto_id: z.number().int(),
	nombre: z.string(),
	sku: z.string(),
	cantidad: z.number().int(),
	costo_unitario: z.number(),
	valor: z.number()
});
export type StockValorizadoItem = z.infer<typeof StockValorizadoItemSchema>;

export const StockValorizadoSchema = z.object({
	items: z.array(StockValorizadoItemSchema),
	total: z.number()
});
export type StockValorizado = z.infer<typeof StockValorizadoSchema>;

/** Kardex (014, FR-056): saldo inicial, líneas con saldo corrido y saldo final. */
export const KardexLineaSchema = z.object({
	movimiento_id: z.number().int(),
	creado_en: z.string(),
	tipo: z.string(),
	motivo: z.string(),
	deposito_id: z.number().int(),
	deposito_nombre: z.string(),
	cantidad: z.number().int(),
	saldo: z.number().int()
});
export type KardexLinea = z.infer<typeof KardexLineaSchema>;

export const KardexSchema = z.object({
	producto_id: z.number().int(),
	producto_nombre: z.string(),
	deposito_id: z.number().int().nullable(),
	desde: z.string().nullable(),
	hasta: z.string().nullable(),
	saldo_inicial: z.number().int(),
	lineas: z.array(KardexLineaSchema),
	saldo_final: z.number().int()
});
export type Kardex = z.infer<typeof KardexSchema>;

/** Rotación (014, FR-058). */
export const RotacionItemSchema = z.object({
	producto_id: z.number().int(),
	nombre: z.string(),
	unidades_salidas: z.number().int(),
	stock_inicial: z.number().int(),
	stock_final: z.number().int(),
	stock_promedio: z.number(),
	indice_rotacion: z.number()
});
export type RotacionItem = z.infer<typeof RotacionItemSchema>;

export const RotacionSchema = z.object({
	desde: z.string(),
	hasta: z.string(),
	items: z.array(RotacionItemSchema)
});
export type Rotacion = z.infer<typeof RotacionSchema>;

/** Ventas del período (014, FR-058): solo confirmadas, por día local y por producto. */
export const VentasReporteSchema = z.object({
	desde: z.string(),
	hasta: z.string(),
	cantidad_ventas: z.number().int(),
	total: z.number(),
	por_dia: z.array(
		z.object({ dia: z.string(), cantidad_ventas: z.number().int(), total: z.number() })
	),
	por_producto: z.array(
		z.object({
			producto_id: z.number().int(),
			nombre: z.string(),
			unidades: z.number().int(),
			total: z.number()
		})
	)
});
export type VentasReporte = z.infer<typeof VentasReporteSchema>;
