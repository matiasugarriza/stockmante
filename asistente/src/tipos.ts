/** Contrato compartido con el backend Rust (`services/asistente.rs`, spec 017). */

export type FamiliaRol = 'GESTION' | 'OPERACION';

/** Lo informa el backend autenticado; nunca sale del texto del usuario (17.5). */
export interface UsuarioSesion {
	id: number;
	nombre: string;
	familia_rol: FamiliaRol;
}

export interface MensajeHistorial {
	rol: 'usuario' | 'asistente';
	texto: string;
}

export const TIPOS_MOVIMIENTO = ['ENTRADA', 'SALIDA', 'MERMA', 'AJUSTE'] as const;
export type TipoMovimiento = (typeof TIPOS_MOVIMIENTO)[number];

/** Mismos valores que valida `models/movimiento.rs::validar_motivo`. */
export const MOTIVOS = [
	'COMPRA',
	'VENTA',
	'MERMA',
	'AJUSTE',
	'DEVOLUCION',
	'TRANSFERENCIA',
	'INVENTARIO',
	'IMPORTACION',
	'OTRO'
] as const;
export type Motivo = (typeof MOTIVOS)[number];

/** Igual a `AccionPropuesta` del backend: lo que el frontend manda a `POST /api/movimientos`. */
export interface AccionPropuesta {
	tipo: TipoMovimiento;
	deposito_id: number;
	deposito_nombre: string;
	producto_id: number;
	producto_nombre: string;
	sku: string;
	cantidad_delta: number;
	motivo: Motivo;
	observacion: string | null;
}

export interface PedidoConsulta {
	mensaje: string;
	historial: MensajeHistorial[];
	usuario: UsuarioSesion;
}

export interface RespuestaConsulta {
	respuesta: string;
	accion_propuesta: AccionPropuesta | null;
	/** Nombres de las tools invocadas; el backend las usa solo para el log (FR-078). */
	tools: string[];
}
