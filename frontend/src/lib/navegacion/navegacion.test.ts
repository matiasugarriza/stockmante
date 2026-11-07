import { describe, expect, it } from 'vitest';
import {
	destinosBarraInferior,
	esRutaActiva,
	gruposVisibles,
	ITEMS_NAVEGACION,
	tituloDeRuta
} from './navegacion';

const GESTION = { familiaRol: 'GESTION' as const, alertasStock: 0 };
const OPERACION = { familiaRol: 'OPERACION' as const, alertasStock: 0 };

function etiquetas(ruta: string, contexto: typeof GESTION | typeof OPERACION): string[] {
	return gruposVisibles(ruta, contexto).flatMap((grupo) => grupo.items.map((item) => item.etiqueta));
}

describe('esRutaActiva', () => {
	it('solo activa Inicio en la raíz exacta', () => {
		expect(esRutaActiva('/', '/')).toBe(true);
		expect(esRutaActiva('/caja', '/')).toBe(false);
	});

	it('activa por prefijo en el resto de las rutas', () => {
		expect(esRutaActiva('/admin/productos', '/admin/productos')).toBe(true);
		expect(esRutaActiva('/admin/productos/12', '/admin/productos')).toBe(true);
		expect(esRutaActiva('/admin/productos-otra', '/admin/productos')).toBe(false);
	});

	it('tolera barra final y rutas desconocidas', () => {
		expect(esRutaActiva('/caja/', '/caja')).toBe(true);
		expect(esRutaActiva('/no-existe', '/caja')).toBe(false);
	});
});

describe('gruposVisibles', () => {
	it('GESTION ve Operación y Administración completas', () => {
		const grupos = gruposVisibles('/admin/productos', GESTION);
		expect(grupos.map((g) => g.etiqueta)).toEqual(['Operación', 'Administración']);
		expect(grupos[0]?.items).toHaveLength(6);
		// 013 agrega Ventas y Clientes; 014, Reportes; 016, Importar.
		expect(grupos[1]?.items).toHaveLength(12);
	});

	it('OPERACION no ve Conteos ni Administración', () => {
		const grupos = gruposVisibles('/caja', OPERACION);
		expect(grupos.map((g) => g.etiqueta)).toEqual(['Operación']);
		const items = grupos.flatMap((g) => g.items.map((i) => i.etiqueta));
		expect(items).toEqual(['Inicio', 'Caja', 'Movimientos', 'Transferencias', 'Asistente']);
		expect(items).not.toContain('Conteos');
		expect(items).not.toContain('Productos');
	});

	it('marca exactamente un ítem activo', () => {
		const activos = gruposVisibles('/admin/stock', GESTION)
			.flatMap((g) => g.items)
			.filter((i) => i.activo);
		expect(activos.map((i) => i.etiqueta)).toEqual(['Stock']);
	});

	it('sin ruta conocida no hay activo', () => {
		const activos = gruposVisibles('/ruta-inexistente', GESTION)
			.flatMap((g) => g.items)
			.filter((i) => i.activo);
		expect(activos).toHaveLength(0);
	});

	it('el contador de Stock solo aparece a GESTION y con alertas', () => {
		const conAlertas = gruposVisibles('/admin/stock', { ...GESTION, alertasStock: 3 })
			.flatMap((g) => g.items)
			.find((i) => i.etiqueta === 'Stock');
		expect(conAlertas?.contadorVisible).toBe(3);

		const sinAlertas = gruposVisibles('/admin/stock', GESTION)
			.flatMap((g) => g.items)
			.find((i) => i.etiqueta === 'Stock');
		expect(sinAlertas?.contadorVisible).toBeNull();
	});

	it('mantiene el orden declarado y no duplica ítems', () => {
		const items = gruposVisibles('/', GESTION).flatMap((g) => g.items);
		expect(items.map((i) => i.ruta)).toEqual(ITEMS_NAVEGACION.map((i) => i.ruta));
		expect(new Set(items.map((i) => i.ruta)).size).toBe(ITEMS_NAVEGACION.length);
	});
});

describe('tituloDeRuta', () => {
	it('resuelve la etiqueta del ítem activo', () => {
		expect(tituloDeRuta('/')).toBe('Inicio');
		expect(tituloDeRuta('/admin/productos/12')).toBe('Productos');
	});

	it('cae a Stockmante en rutas fuera de la navegación', () => {
		expect(tituloDeRuta('/no-existe')).toBe('Stockmante');
	});
});

describe('destinosBarraInferior', () => {
	it('GESTION ve 4 destinos + Más y el cuarto es Stock con contador', () => {
		const destinos = destinosBarraInferior('/admin/auditoria', {
			familiaRol: 'GESTION',
			alertasStock: 3
		});
		expect(destinos.map((d) => d.etiqueta)).toEqual([
			'Inicio',
			'Caja',
			'Movimientos',
			'Stock',
			'Más'
		]);
		expect(destinos[3]?.contadorVisible).toBe(3);
		expect(destinos[4]?.activo).toBe(true);
		expect(destinos[0]?.activo).toBe(false);
	});

	it('OPERACION usa Transferencias como cuarto destino y no ve Stock', () => {
		const destinos = destinosBarraInferior('/caja', OPERACION);
		expect(destinos.map((d) => d.etiqueta)).toEqual([
			'Inicio',
			'Caja',
			'Movimientos',
			'Transferencias',
			'Más'
		]);
		expect(destinos.some((d) => d.etiqueta === 'Stock')).toBe(false);
	});

	it('marca el destino de la barra y no Más cuando la ruta está en la barra', () => {
		const destinos = destinosBarraInferior('/caja', OPERACION);
		expect(destinos[1]?.activo).toBe(true);
		expect(destinos[4]?.activo).toBe(false);
	});

	it('marca el cuarto destino cuando la ruta es Stock', () => {
		const destinos = destinosBarraInferior('/admin/stock', GESTION);
		expect(destinos[3]?.activo).toBe(true);
		expect(destinos[4]?.activo).toBe(false);
	});

	it('siempre devuelve cinco destinos con etiqueta y ruta', () => {
		const destinos = destinosBarraInferior('/', GESTION);
		expect(destinos).toHaveLength(5);
		expect(destinos.every((d) => d.etiqueta && d.ruta && d.icono)).toBe(true);
		expect(destinos[4]?.ruta).toBe('#panel');
	});
});
