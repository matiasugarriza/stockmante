import { describe, expect, it } from 'vitest';
import { ClienteApi } from './api.ts';
import { crearTools, type RegistroConsulta } from './tools.ts';
import type { UsuarioSesion } from './tipos.ts';
import { crearFetchFalso, producto, rutasCatalogo, type Rutas } from './pruebas/api-falsa.ts';

const COOKIE = 'stockmante_sesion=11111111-1111-1111-1111-111111111111';
const OPERARIO: UsuarioSesion = { id: 10, nombre: 'Ana', familia_rol: 'OPERACION' };
const GERENTE: UsuarioSesion = { id: 20, nombre: 'Beto', familia_rol: 'GESTION' };

function armar(usuario: UsuarioSesion, rutas: Rutas = rutasCatalogo()) {
	const { fetch, llamadas } = crearFetchFalso(rutas);
	const api = new ClienteApi('http://api.test', COOKIE, fetch);
	const registro: RegistroConsulta = { tools: [], propuesta: null };
	const tools = crearTools(api, usuario, registro);
	const tool = (nombre: string) => {
		const encontrada = tools.find((t) => t.name === nombre);
		if (!encontrada) throw new Error(`no existe la tool ${nombre}`);
		return encontrada;
	};
	return { tools, tool, registro, llamadas };
}

describe('tools de lectura (FR-066)', () => {
	it('expone exactamente la lista cerrada de tools', () => {
		const { tools } = armar(OPERARIO);
		expect(tools.map((t) => t.name).sort()).toEqual(
			[
				'buscar_producto',
				'consultar_stock',
				'listar_depositos',
				'proponer_movimiento',
				'reporte_agregado',
				'resumen_ventas'
			].sort()
		);
	});

	it('consultar_stock usa la cookie de la sesión y devuelve el desglose por depósito (17.1)', async () => {
		const { tool, llamadas, registro } = armar(GERENTE);
		const salida = String(await tool('consultar_stock').invoke({ producto: 'yerba suave 1 kg' }));
		const datos = JSON.parse(salida);
		expect(datos.items[0].desglose[0]).toMatchObject({ deposito_nombre: 'Principal', cantidad: 12 });
		expect(datos.items[0].stock_minimo).toBe(5);
		expect(llamadas.every((l) => l.cookie === COOKIE)).toBe(true);
		expect(registro.tools).toContain('consultar_stock');
	});

	it('una sesión vencida se informa y no se reintenta con otra credencial (17.6)', async () => {
		const rutas: Rutas = {
			'GET /api/productos/buscar': {
				status: 401,
				body: { error: { codigo: 'sesion_expirada', mensaje: 'Tu sesión expiró. Iniciá sesión nuevamente' } }
			}
		};
		const { tool, llamadas } = armar(OPERARIO, rutas);
		const salida = String(await tool('buscar_producto').invoke({ texto: 'yerba' }));
		expect(salida).toContain('Tu sesión expiró');
		expect(llamadas).toHaveLength(1);
	});

	it('un 403 de la API se informa con su mensaje real', async () => {
		const rutas: Rutas = {
			'GET /api/depositos': {
				status: 403,
				body: { error: { codigo: 'sin_permisos', mensaje: 'No tenés permiso para realizar esta acción' } }
			}
		};
		const { tool } = armar(OPERARIO, rutas);
		const salida = String(await tool('listar_depositos').invoke({}));
		expect(salida).toContain('No tenés permiso para realizar esta acción');
	});

	it('reporte_agregado solo consulta reportes conocidos', async () => {
		const rutas: Rutas = { 'GET /api/reportes/rotacion': { status: 200, body: { items: [] } } };
		const { tool, llamadas } = armar(GERENTE, rutas);
		await tool('reporte_agregado').invoke({ reporte: 'rotacion', desde: '2026-10-01' });
		expect(llamadas[0].ruta).toBe('/api/reportes/rotacion?desde=2026-10-01');
		await expect(tool('reporte_agregado').invoke({ reporte: '../usuarios' })).rejects.toThrow();
	});
});

describe('proponer_movimiento (FR-067, 17.2, 17.3, 17.10)', () => {
	it('arma una MERMA con delta negativo y no escribe nada', async () => {
		const { tool, registro, llamadas } = armar(OPERARIO);
		const salida = String(
			await tool('proponer_movimiento').invoke({
				tipo: 'MERMA',
				producto: 'Yerba Suave 1 kg',
				deposito: 'principal',
				cantidad: 2,
				observacion: 'vencimiento'
			})
		);
		expect(registro.propuesta).toEqual({
			tipo: 'MERMA',
			deposito_id: 1,
			deposito_nombre: 'Principal',
			producto_id: 1,
			producto_nombre: 'Yerba Suave 1 kg',
			sku: 'YER-SUA-1K',
			cantidad_delta: -2,
			motivo: 'MERMA',
			observacion: 'vencimiento'
		});
		expect(salida).toContain('confirm');
		expect(llamadas.every((l) => l.metodo === 'GET')).toBe(true);
	});

	it('con varias coincidencias no propone y lista candidatos con nombre y SKU (17.10)', async () => {
		const { tool, registro } = armar(OPERARIO);
		const salida = String(
			await tool('proponer_movimiento').invoke({ tipo: 'MERMA', producto: 'yerba', deposito: 'Principal', cantidad: 2 })
		);
		expect(registro.propuesta).toBeNull();
		expect(salida).toContain('Yerba Suave 1 kg (SKU YER-SUA-1K)');
		expect(salida).toContain('Yerba Suave 500 g (SKU YER-SUA-500)');
		expect(salida).toContain('Yerba Compuesta 1 kg (SKU YER-COM-1K)');
	});

	it('con más de 5 coincidencias muestra 5 y pide precisar', async () => {
		const muchos = Array.from({ length: 7 }, (_, i) => producto(100 + i, `Galletita ${i}`, `GAL-${i}00`));
		const rutas: Rutas = { ...rutasCatalogo(), 'GET /api/productos/buscar': { status: 200, body: { items: muchos } } };
		const { tool, registro } = armar(OPERARIO, rutas);
		const salida = String(
			await tool('proponer_movimiento').invoke({ tipo: 'ENTRADA', producto: 'galletita', deposito: 'Principal', cantidad: 5 })
		);
		expect(registro.propuesta).toBeNull();
		expect(salida).toContain('Galletita 4');
		expect(salida).not.toContain('Galletita 5');
		expect(salida).toContain('precis');
	});

	it('un SKU exacto resuelve la ambigüedad', async () => {
		const { tool, registro } = armar(OPERARIO);
		await tool('proponer_movimiento').invoke({ tipo: 'SALIDA', producto: 'YER-SUA-500', deposito: 'Sucursal', cantidad: 1 });
		expect(registro.propuesta).toMatchObject({ producto_id: 2, deposito_id: 2, cantidad_delta: -1, motivo: 'VENTA' });
	});

	it('un operario no recibe propuestas de AJUSTE (17.3)', async () => {
		const { tool, registro } = armar(OPERARIO);
		const salida = String(
			await tool('proponer_movimiento').invoke({ tipo: 'AJUSTE', producto: 'YER-SUA-1K', deposito: 'Principal', cantidad: -3 })
		);
		expect(registro.propuesta).toBeNull();
		expect(salida).toContain('gerente');
	});

	it('un gerente sí recibe la propuesta de AJUSTE con el signo pedido', async () => {
		const { tool, registro } = armar(GERENTE);
		await tool('proponer_movimiento').invoke({ tipo: 'AJUSTE', producto: 'YER-SUA-1K', deposito: 'Principal', cantidad: -3 });
		expect(registro.propuesta).toMatchObject({ tipo: 'AJUSTE', cantidad_delta: -3, motivo: 'AJUSTE' });
	});

	it('un depósito sin stock del producto se resuelve con el listado si el usuario es gerente', async () => {
		const { tool, registro } = armar(GERENTE);
		await tool('proponer_movimiento').invoke({ tipo: 'ENTRADA', producto: 'YER-SUA-1K', deposito: 'Depósito Norte', cantidad: 4 });
		expect(registro.propuesta).toMatchObject({ deposito_id: 3, cantidad_delta: 4, motivo: 'COMPRA' });
	});

	it('si no identifica el depósito no propone y explica qué hacer', async () => {
		const { tool, registro } = armar(OPERARIO, {
			...rutasCatalogo(),
			'GET /api/depositos': { status: 403, body: { error: { codigo: 'sin_permisos', mensaje: 'No tenés permiso' } } }
		});
		const salida = String(
			await tool('proponer_movimiento').invoke({ tipo: 'ENTRADA', producto: 'YER-SUA-1K', deposito: 'Norte', cantidad: 4 })
		);
		expect(registro.propuesta).toBeNull();
		expect(salida).toContain('depósito');
	});

	it('rechaza cantidades inválidas', async () => {
		const { tool, registro } = armar(GERENTE);
		const salida = String(
			await tool('proponer_movimiento').invoke({ tipo: 'MERMA', producto: 'YER-SUA-1K', deposito: 'Principal', cantidad: 0 })
		);
		expect(registro.propuesta).toBeNull();
		expect(salida).toContain('cantidad');
	});

	it('motivo OTRO exige observación', async () => {
		const { tool, registro } = armar(GERENTE);
		const salida = String(
			await tool('proponer_movimiento').invoke({
				tipo: 'SALIDA',
				producto: 'YER-SUA-1K',
				deposito: 'Principal',
				cantidad: 1,
				motivo: 'OTRO'
			})
		);
		expect(registro.propuesta).toBeNull();
		expect(salida).toContain('observación');
	});
});
