/**
 * Tools del asistente (FR-066/FR-067). Todas leen la API Rust con la sesión del usuario;
 * ninguna escribe. `proponer_movimiento` solo arma una propuesta que el usuario confirma
 * desde el frontend, y resuelve producto y depósito con reglas deterministas en vez de
 * confiar en el modelo (17.3, 17.10).
 */
import { tool, type StructuredToolInterface } from '@langchain/core/tools';
import { z } from 'zod';
import { mensajeDeError, type ClienteApi } from './api.ts';
import { MOTIVOS, TIPOS_MOVIMIENTO, type AccionPropuesta, type Motivo, type TipoMovimiento, type UsuarioSesion } from './tipos.ts';

export interface RegistroConsulta {
	/** Tools invocadas, sin repetir, en orden (FR-078). */
	tools: string[];
	/** Última propuesta válida de la consulta; nunca más de una. */
	propuesta: AccionPropuesta | null;
}

export const MAX_CANDIDATOS = 5;
export const REPORTES = ['stock-valorizado', 'rotacion', 'ventas'] as const;

const MOTIVO_POR_TIPO: Record<TipoMovimiento, Motivo> = {
	ENTRADA: 'COMPRA',
	SALIDA: 'VENTA',
	MERMA: 'MERMA',
	AJUSTE: 'AJUSTE'
};

/** Matriz tipo ↔ motivo del alta manual (007, ampliación 2026-11-07). */
export const MOTIVOS_POR_TIPO: Record<TipoMovimiento, readonly Motivo[]> = {
	ENTRADA: ['COMPRA', 'DEVOLUCION', 'OTRO'],
	SALIDA: ['VENTA', 'DEVOLUCION', 'OTRO'],
	MERMA: ['MERMA', 'OTRO'],
	AJUSTE: ['AJUSTE', 'OTRO']
};

interface ProductoApi {
	id: number;
	nombre: string;
	sku: string;
}

interface DepositoCandidato {
	id: number;
	nombre: string;
	identificador?: string;
}

const json = (valor: unknown) => JSON.stringify(valor);

function normalizar(texto: string): string {
	return texto
		.normalize('NFD')
		.replace(/[̀-ͯ]/g, '')
		.trim()
		.toLowerCase();
}

function listarCandidatos(productos: ProductoApi[]): string {
	return productos
		.slice(0, MAX_CANDIDATOS)
		.map((p) => `- ${p.nombre} (SKU ${p.sku})`)
		.join('\n');
}

type Resolucion<T> = { valor: T } | { mensaje: string };

async function resolverProducto(api: ClienteApi, texto: string): Promise<Resolucion<ProductoApi>> {
	const res = await api.get('/api/productos/buscar', { q: texto, limite: MAX_CANDIDATOS + 1 });
	if (!res.ok) return { mensaje: mensajeDeError(res) };
	const items = ((res.body as { items?: ProductoApi[] } | null)?.items ?? []) as ProductoApi[];
	const buscado = normalizar(texto);
	const exacto = items.find((p) => normalizar(p.sku) === buscado || normalizar(p.nombre) === buscado);
	if (exacto) return { valor: exacto };
	if (items.length === 1) return { valor: items[0] };
	if (items.length === 0) {
		return {
			mensaje: `No encontré productos que coincidan con «${texto}». Pedile al usuario el nombre exacto o el SKU.`
		};
	}
	const exceso =
		items.length > MAX_CANDIDATOS
			? `\nHay más de ${MAX_CANDIDATOS} coincidencias: pedile que precise el nombre o el SKU.`
			: '';
	return {
		mensaje:
			`«${texto}» coincide con varios productos. No propongas nada: preguntale al usuario cuál es.\n` +
			listarCandidatos(items) +
			exceso
	};
}

function coincidirDeposito(candidatos: DepositoCandidato[], texto: string): Resolucion<DepositoCandidato> | null {
	const buscado = normalizar(texto);
	const exacto = candidatos.find(
		(d) => normalizar(d.nombre) === buscado || (d.identificador && normalizar(d.identificador) === buscado)
	);
	if (exacto) return { valor: exacto };
	const parciales = candidatos.filter((d) => normalizar(d.nombre).includes(buscado));
	if (parciales.length === 1) return { valor: parciales[0] };
	if (parciales.length > 1) {
		return {
			mensaje: `«${texto}» coincide con varios depósitos (${parciales.map((d) => d.nombre).join(', ')}). Preguntale al usuario cuál es.`
		};
	}
	return null;
}

/**
 * Primero el desglose de stock del producto (abierto a cualquier sesión); si no aparece,
 * el listado de depósitos, que la API reserva a GESTION.
 */
async function resolverDeposito(
	api: ClienteApi,
	productoId: number,
	texto: string
): Promise<Resolucion<DepositoCandidato>> {
	const stock = await api.get(`/api/stock/${productoId}`);
	if (stock.ok) {
		const desglose = ((stock.body as { desglose?: Array<{ deposito_id: number; deposito_nombre: string }> } | null)
			?.desglose ?? []);
		const unicos = new Map<number, DepositoCandidato>();
		for (const fila of desglose) unicos.set(fila.deposito_id, { id: fila.deposito_id, nombre: fila.deposito_nombre });
		const encontrado = coincidirDeposito([...unicos.values()], texto);
		if (encontrado) return encontrado;
	}
	const depositos = await api.get('/api/depositos');
	if (depositos.ok && Array.isArray(depositos.body)) {
		const encontrado = coincidirDeposito(depositos.body as DepositoCandidato[], texto);
		if (encontrado) return encontrado;
	}
	return {
		mensaje:
			`No pude identificar el depósito «${texto}». Pedile al usuario el nombre exacto de un depósito ` +
			'donde el producto tenga stock, o que un gerente registre el movimiento.'
	};
}

function calcularDelta(tipo: TipoMovimiento, cantidad: number): Resolucion<number> {
	if (tipo === 'AJUSTE') {
		return cantidad === 0
			? { mensaje: 'La cantidad del AJUSTE no puede ser cero: indicá cuánto sumar (positivo) o restar (negativo).' }
			: { valor: cantidad };
	}
	if (cantidad <= 0) {
		return { mensaje: 'La cantidad debe ser un entero mayor que cero. Pedile al usuario cuántas unidades son.' };
	}
	return { valor: tipo === 'ENTRADA' ? cantidad : -cantidad };
}

export function crearTools(
	api: ClienteApi,
	usuario: UsuarioSesion,
	registro: RegistroConsulta
): StructuredToolInterface[] {
	const usar = (nombre: string) => {
		if (!registro.tools.includes(nombre)) registro.tools.push(nombre);
	};
	const leer = async (nombre: string, ruta: string, params?: Record<string, string | number | boolean | undefined>) => {
		usar(nombre);
		const res = await api.get(ruta, params);
		return res.ok ? json(res.body) : mensajeDeError(res);
	};

	const consultarStock = tool(
		async ({ producto, bajo_minimo }) => {
			usar('consultar_stock');
			const lista = await api.get('/api/stock', { q: producto, bajo_minimo, por_pagina: MAX_CANDIDATOS });
			if (!lista.ok) return mensajeDeError(lista);
			if (!producto) return json(lista.body);
			const items = ((lista.body as { items?: Array<{ producto_id: number }> } | null)?.items ?? []);
			const detalles = [];
			for (const item of items) {
				const detalle = await api.get(`/api/stock/${item.producto_id}`);
				if (detalle.ok) detalles.push(detalle.body);
			}
			return json({ items: detalles, total: (lista.body as { total?: number }).total ?? detalles.length });
		},
		{
			name: 'consultar_stock',
			description:
				'Stock de productos con su desglose por depósito y el stock mínimo. Usala cuando pregunten cuánto hay de algo o qué está bajo el mínimo.',
			schema: z.object({
				producto: z.string().min(1).optional().describe('Nombre o SKU del producto'),
				bajo_minimo: z.boolean().optional().describe('true para listar solo productos bajo el mínimo')
			})
		}
	);

	const buscarProducto = tool(
		async ({ texto }) => leer('buscar_producto', '/api/productos/buscar', { q: texto, limite: 10 }),
		{
			name: 'buscar_producto',
			description: 'Busca productos del catálogo por nombre, SKU o código de barras.',
			schema: z.object({ texto: z.string().min(1).describe('Texto a buscar') })
		}
	);

	const listarDepositos = tool(async () => leer('listar_depositos', '/api/depositos'), {
		name: 'listar_depositos',
		description: 'Lista los depósitos activos (la API lo permite solo a gerentes).',
		schema: z.object({})
	});

	const resumenVentas = tool(
		async ({ desde, hasta, deposito_id }) =>
			leer('resumen_ventas', '/api/ventas', { desde, hasta, deposito_id, por_pagina: 20 }),
		{
			name: 'resumen_ventas',
			description: 'Ventas registradas en un rango de fechas (ISO 8601), opcionalmente por depósito.',
			schema: z.object({
				desde: z.string().optional(),
				hasta: z.string().optional(),
				deposito_id: z.number().int().positive().optional()
			})
		}
	);

	const reporteAgregado = tool(
		async ({ reporte, desde, hasta }) => leer('reporte_agregado', `/api/reportes/${reporte}`, { desde, hasta }),
		{
			name: 'reporte_agregado',
			description: 'Reportes agregados: stock-valorizado, rotacion o ventas, con rango de fechas opcional (ISO 8601).',
			schema: z.object({
				reporte: z.enum(REPORTES),
				desde: z.string().optional(),
				hasta: z.string().optional()
			})
		}
	);

	const proponerMovimiento = tool(
		async ({ tipo, producto, deposito, cantidad, motivo, observacion }) => {
			usar('proponer_movimiento');
			if (tipo === 'AJUSTE' && usuario.familia_rol !== 'GESTION') {
				return 'El AJUSTE requiere un gerente (familia GESTION). No lo propongas: explicale al usuario que se lo pida a un gerente.';
			}
			const delta = calcularDelta(tipo, cantidad);
			if ('mensaje' in delta) return delta.mensaje;
			const motivoFinal = motivo ?? MOTIVO_POR_TIPO[tipo];
			if (!MOTIVOS_POR_TIPO[tipo].includes(motivoFinal)) {
				return (
					`El motivo ${motivoFinal} no corresponde a una ${tipo}. ` +
					`Motivos válidos: ${MOTIVOS_POR_TIPO[tipo].join(', ')}. Pedile al usuario que elija uno.`
				);
			}
			const observacionFinal = observacion?.trim() ? observacion.trim() : null;
			if (motivoFinal === 'OTRO' && !observacionFinal) {
				return 'Con motivo OTRO hace falta una observación. Pedile al usuario que la indique.';
			}
			const productoResuelto = await resolverProducto(api, producto);
			if ('mensaje' in productoResuelto) return productoResuelto.mensaje;
			const depositoResuelto = await resolverDeposito(api, productoResuelto.valor.id, deposito);
			if ('mensaje' in depositoResuelto) return depositoResuelto.mensaje;

			const propuesta: AccionPropuesta = {
				tipo,
				deposito_id: depositoResuelto.valor.id,
				deposito_nombre: depositoResuelto.valor.nombre,
				producto_id: productoResuelto.valor.id,
				producto_nombre: productoResuelto.valor.nombre,
				sku: productoResuelto.valor.sku,
				cantidad_delta: delta.valor,
				motivo: motivoFinal,
				observacion: observacionFinal
			};
			registro.propuesta = propuesta;
			return (
				`Propuesta lista: ${tipo} de ${Math.abs(delta.valor)} unidades de ${propuesta.producto_nombre} ` +
				`(SKU ${propuesta.sku}) en ${propuesta.deposito_nombre}, motivo ${motivoFinal}. ` +
				'NO está ejecutada: el usuario tiene que confirmarla en la pantalla.'
			);
		},
		{
			name: 'proponer_movimiento',
			description:
				'Arma una propuesta de ENTRADA, SALIDA, MERMA o AJUSTE para que el usuario la confirme. No ejecuta nada. ' +
				'Usala solo cuando el usuario pida registrar un movimiento.',
			schema: z.object({
				tipo: z.enum(TIPOS_MOVIMIENTO),
				producto: z.string().min(1).describe('Nombre o SKU del producto tal como lo dijo el usuario'),
				deposito: z.string().min(1).describe('Nombre del depósito'),
				cantidad: z
					.number()
					.int()
					.describe('Unidades. Positiva para ENTRADA/SALIDA/MERMA; en AJUSTE, con signo (+ suma, − resta)'),
				motivo: z.enum(MOTIVOS).optional(),
				observacion: z.string().max(500).optional().describe('Detalle, por ejemplo "vencimiento"')
			})
		}
	);

	return [consultarStock, buscarProducto, listarDepositos, resumenVentas, reporteAgregado, proponerMovimiento];
}
