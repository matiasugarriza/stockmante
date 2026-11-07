/**
 * SC-025 / FR-069: matriz tools × familia de rol. Ninguna combinación escribe contra la
 * API y ninguna propuesta excede los permisos de la familia del usuario.
 */
import { describe, expect, it } from 'vitest';
import { ClienteApi } from './api.ts';
import { crearTools, type RegistroConsulta } from './tools.ts';
import type { FamiliaRol, UsuarioSesion } from './tipos.ts';
import { crearFetchFalso, rutasCatalogo } from './pruebas/api-falsa.ts';

const ENTRADAS: Record<string, Record<string, unknown>> = {
	consultar_stock: { producto: 'yerba' },
	buscar_producto: { texto: 'yerba' },
	listar_depositos: {},
	resumen_ventas: { desde: '2026-10-01' },
	reporte_agregado: { reporte: 'ventas' },
	proponer_movimiento: { tipo: 'MERMA', producto: 'YER-SUA-1K', deposito: 'Principal', cantidad: 1 }
};

const TIPOS = ['ENTRADA', 'SALIDA', 'MERMA', 'AJUSTE'] as const;
const FAMILIAS: FamiliaRol[] = ['GESTION', 'OPERACION'];

function armar(familia: FamiliaRol) {
	const usuario: UsuarioSesion = { id: 1, nombre: 'Test', familia_rol: familia };
	const { fetch, llamadas } = crearFetchFalso({
		...rutasCatalogo(),
		'GET /api/ventas': { status: 200, body: { items: [] } },
		'GET /api/reportes/ventas': { status: 200, body: { total: 0 } }
	});
	const registro: RegistroConsulta = { tools: [], propuesta: null };
	const tools = crearTools(new ClienteApi('http://api.test', 'stockmante_sesion=x', fetch), usuario, registro);
	return { tools, registro, llamadas };
}

describe.each(FAMILIAS)('familia %s', (familia) => {
	it.each(Object.keys(ENTRADAS))('la tool %s solo hace GET', async (nombre) => {
		const { tools, llamadas } = armar(familia);
		const tool = tools.find((t) => t.name === nombre);
		expect(tool).toBeDefined();
		await tool?.invoke(ENTRADAS[nombre]);
		expect(llamadas.length).toBeGreaterThan(0);
		expect(llamadas.every((l) => l.metodo === 'GET')).toBe(true);
	});

	it.each(TIPOS)('propuesta de %s respeta la familia', async (tipo) => {
		const { tools, registro } = armar(familia);
		const tool = tools.find((t) => t.name === 'proponer_movimiento');
		await tool?.invoke({ tipo, producto: 'YER-SUA-1K', deposito: 'Principal', cantidad: tipo === 'AJUSTE' ? -1 : 1 });
		const permitida = tipo !== 'AJUSTE' || familia === 'GESTION';
		expect(registro.propuesta !== null).toBe(permitida);
	});
});
