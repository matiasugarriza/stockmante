/**
 * Contraste WCAG (spec 019, FR-103/SC-042).
 *
 * Funciones puras sobre colores hex: el guard de tokens las usa para verificar la paleta
 * móvil y evitar que un cambio de color deje un par por debajo de su mínimo.
 */

export interface ColorRgb {
	r: number;
	g: number;
	b: number;
}

/** Acepta `#rrggbb`, `#rgb` y el hex sin `#`. Devuelve null si no es válido. */
export function parsearHex(valor: string): ColorRgb | null {
	const limpio = valor.trim().replace(/^#/, '');
	if (!/^[0-9a-fA-F]{3}$|^[0-9a-fA-F]{6}$/.test(limpio)) return null;
	const completo =
		limpio.length === 3
			? limpio
					.split('')
					.map((caracter) => caracter + caracter)
					.join('')
			: limpio;
	return {
		r: Number.parseInt(completo.slice(0, 2), 16),
		g: Number.parseInt(completo.slice(2, 4), 16),
		b: Number.parseInt(completo.slice(4, 6), 16)
	};
}

function canalLineal(canal: number): number {
	const normalizado = canal / 255;
	return normalizado <= 0.03928
		? normalizado / 12.92
		: ((normalizado + 0.055) / 1.055) ** 2.4;
}

/** Luminancia relativa según WCAG 2.x. Lanza si el color no es válido. */
export function luminanciaRelativa(valor: string): number {
	const rgb = parsearHex(valor);
	if (!rgb) throw new Error(`Color inválido: ${valor}`);
	return (
		0.2126 * canalLineal(rgb.r) + 0.7152 * canalLineal(rgb.g) + 0.0722 * canalLineal(rgb.b)
	);
}

/** Ratio de contraste entre dos colores (1 a 21). */
export function ratioContraste(primero: string, segundo: string): number {
	const luminanciaA = luminanciaRelativa(primero);
	const luminanciaB = luminanciaRelativa(segundo);
	const claro = Math.max(luminanciaA, luminanciaB);
	const oscuro = Math.min(luminanciaA, luminanciaB);
	return (claro + 0.05) / (oscuro + 0.05);
}

/** `true` si el ratio alcanza el mínimo exigido (7, 4,5 o 3). */
export function cumpleMinimo(ratio: number, minimo: number): boolean {
	return ratio >= minimo;
}
