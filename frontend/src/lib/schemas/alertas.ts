import { z } from 'zod';

/** Producto activo por debajo de su mínimo (011). */
export const AlertaItemSchema = z.object({
	producto_id: z.number().int(),
	nombre: z.string(),
	sku: z.string(),
	stock_actual: z.number().int(),
	stock_minimo: z.number().int(),
	faltante: z.number().int()
});
export type AlertaItem = z.infer<typeof AlertaItemSchema>;

export const ListaAlertasSchema = z.object({
	items: z.array(AlertaItemSchema),
	total: z.number().int(),
	pagina: z.number().int(),
	por_pagina: z.number().int()
});
export type ListaAlertas = z.infer<typeof ListaAlertasSchema>;

/** Resultado de `POST /api/alertas/enviar`. */
export const RespuestaCicloSchema = z.object({
	fecha: z.string(),
	productos: z.number().int(),
	destinatarios: z.number().int(),
	enviados: z.number().int(),
	fallidos: z.number().int()
});
export type RespuestaCiclo = z.infer<typeof RespuestaCicloSchema>;
