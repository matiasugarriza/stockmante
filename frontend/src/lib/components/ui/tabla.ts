export type Alineacion = 'izquierda' | 'derecha' | 'centro';

export interface Columna<T> {
	clave: string;
	encabezado: string;
	alineacion?: Alineacion;
	clase?: string;
}
