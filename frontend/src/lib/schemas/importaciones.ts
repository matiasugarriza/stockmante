import { z } from 'zod';

/** Error por celda de la importación CSV (016). `fila` 0 = archivo, 1 = encabezado. */
export const DetalleImportacionSchema = z.object({
	fila: z.number().int(),
	columna: z.string(),
	mensaje: z.string()
});
export type DetalleImportacion = z.infer<typeof DetalleImportacionSchema>;

export const ResumenImportacionSchema = z.object({
	filas_leidas: z.number().int(),
	validas: z.number().int(),
	con_error: z.number().int(),
	errores: z.array(DetalleImportacionSchema),
	creados: z.number().int(),
	aplicado: z.boolean()
});
export type ResumenImportacion = z.infer<typeof ResumenImportacionSchema>;

/** Sobre de error de `importacion_invalida`. */
export const ErrorImportacionSchema = z.object({
	error: z.object({
		codigo: z.string(),
		mensaje: z.string(),
		detalles: z.array(DetalleImportacionSchema).optional()
	})
});
export type ErrorImportacion = z.infer<typeof ErrorImportacionSchema>;
