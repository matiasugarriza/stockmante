/**
 * Matemática de gráficos SVG (spec 018, FR-090/FR-093).
 * Funciones puras, sin DOM: los componentes solo dibujan el resultado.
 */

export interface DatoGrafico {
	etiqueta: string;
	valor: number;
	color?: string;
}

export interface BarraCalculada extends DatoGrafico {
	x: number;
	y: number;
	ancho: number;
	alto: number;
	porcentaje: number;
}

export interface EscalaBarras {
	barras: BarraCalculada[];
	max: number;
	ancho: number;
	alto: number;
}

export interface OpcionesBarras {
	ancho?: number;
	alto?: number;
	separacion?: number;
}

function valorSeguro(valor: number): number {
	return typeof valor === 'number' && Number.isFinite(valor) ? Math.max(0, valor) : 0;
}

export function porcentaje(valor: number, total: number): number {
	if (!Number.isFinite(valor) || !Number.isFinite(total) || total <= 0) return 0;
	return Math.max(0, Math.min(100, (valor / total) * 100));
}

export function escalaBarras(datos: DatoGrafico[], opciones: OpcionesBarras = {}): EscalaBarras {
	const ancho = opciones.ancho ?? 600;
	const alto = opciones.alto ?? 220;
	const separacion = opciones.separacion ?? 16;
	const normalizados = datos.map((dato) => ({ ...dato, valor: valorSeguro(dato.valor) }));
	const max = Math.max(1, ...normalizados.map((dato) => dato.valor));
	const anchoBarra =
		normalizados.length > 0
			? Math.max(0, (ancho - separacion * (normalizados.length - 1)) / normalizados.length)
			: 0;

	const barras = normalizados.map<BarraCalculada>((dato, indice) => {
		const proporcion = dato.valor / max;
		return {
			...dato,
			x: indice * (anchoBarra + separacion),
			y: alto - proporcion * alto,
			ancho: anchoBarra,
			alto: proporcion * alto,
			porcentaje: proporcion * 100
		};
	});

	return { barras, max, ancho, alto };
}

export interface SegmentoDona extends DatoGrafico {
	porcentaje: number;
	inicio: number;
	fin: number;
	d: string;
}

export interface OpcionesDona {
	cx?: number;
	cy?: number;
	radioExterior?: number;
	radioInterior?: number;
}

function punto(cx: number, cy: number, radio: number, angulo: number): { x: number; y: number } {
	return { x: cx + radio * Math.cos(angulo), y: cy + radio * Math.sin(angulo) };
}

function describirArco(
	cx: number,
	cy: number,
	radioExterior: number,
	radioInterior: number,
	inicio: number,
	fin: number
): string {
	const p1 = punto(cx, cy, radioExterior, inicio);
	const p2 = punto(cx, cy, radioExterior, fin);
	const p3 = punto(cx, cy, radioInterior, fin);
	const p4 = punto(cx, cy, radioInterior, inicio);
	const grande = fin - inicio > Math.PI ? 1 : 0;
	return [
		`M ${p1.x.toFixed(3)} ${p1.y.toFixed(3)}`,
		`A ${radioExterior} ${radioExterior} 0 ${grande} 1 ${p2.x.toFixed(3)} ${p2.y.toFixed(3)}`,
		`L ${p3.x.toFixed(3)} ${p3.y.toFixed(3)}`,
		`A ${radioInterior} ${radioInterior} 0 ${grande} 0 ${p4.x.toFixed(3)} ${p4.y.toFixed(3)}`,
		'Z'
	].join(' ');
}

export function arcoDona(datos: DatoGrafico[], opciones: OpcionesDona = {}): SegmentoDona[] {
	const cx = opciones.cx ?? 100;
	const cy = opciones.cy ?? 100;
	const radioExterior = opciones.radioExterior ?? 88;
	const radioInterior = opciones.radioInterior ?? 56;
	const normalizados = datos.map((dato) => ({ ...dato, valor: valorSeguro(dato.valor) }));
	const total = normalizados.reduce((suma, dato) => suma + dato.valor, 0);
	if (total <= 0) return [];

	const dosPi = Math.PI * 2;
	const inicioBase = -Math.PI / 2;
	let cursor = inicioBase;

	return normalizados.map<SegmentoDona>((dato, indice) => {
		const proporcion = dato.valor / total;
		const inicio = cursor;
		const finBruto = inicio + proporcion * dosPi;
		// Un único segmento que ocupa toda la circunferencia no puede dibujarse con
		// un solo arco (inicio === fin); se recorta una milésima.
		const fin = Math.min(finBruto, inicioBase + dosPi - (indice === normalizados.length - 1 ? 0.0001 : 0));
		cursor = finBruto;
		return {
			...dato,
			porcentaje: proporcion * 100,
			inicio,
			fin,
			d: describirArco(cx, cy, radioExterior, radioInterior, inicio, fin)
		};
	});
}
