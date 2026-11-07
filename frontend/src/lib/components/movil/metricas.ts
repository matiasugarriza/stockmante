import type { NombreIcono } from '../ui/iconos';

/** Métrica del tablero móvil (spec 019, data-model §5). */
export interface MetricaTablero {
	id: string;
	etiqueta: string;
	valor: string;
	contexto?: string;
	icono?: NombreIcono;
	variante?: 'neutro' | 'aviso' | 'peligro' | 'exito' | 'info';
	cargando?: boolean;
}
