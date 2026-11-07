/**
 * Cliente de importación masiva (016). Solo GESTION. `validar` no escribe; `aplicar` es
 * todo o nada y, con errores, lanza `ApiError` con `detalles`.
 */
import { apiFormData } from './client';
import { ResumenImportacionSchema, type ResumenImportacion } from '../schemas/importaciones';

export type ModoImportacion = 'validar' | 'aplicar';

export async function importarProductos(
	archivo: File,
	modo: ModoImportacion
): Promise<ResumenImportacion> {
	const datos = new FormData();
	datos.append('archivo', archivo);
	const body = await apiFormData<unknown>(`/api/importaciones/productos?modo=${modo}`, datos);
	return ResumenImportacionSchema.parse(body);
}
