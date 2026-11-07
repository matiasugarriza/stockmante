/**
 * Carrito del punto de venta (013): lógica pura, sin I/O. El precio que se muestra es el del
 * catálogo al momento de agregar; el backend lo vuelve a tomar al confirmar (FR-050).
 */

export interface ProductoCarrito {
	producto_id: number;
	nombre: string;
	precio: number;
}

export interface ItemCarrito extends ProductoCarrito {
	cantidad: number;
}

/** Agrega `cantidad` unidades; si el producto ya está, fusiona la línea (13.9). */
export function agregarAlCarrito(
	carrito: readonly ItemCarrito[],
	producto: ProductoCarrito,
	cantidad = 1
): ItemCarrito[] {
	const existe = carrito.some((i) => i.producto_id === producto.producto_id);
	if (existe) {
		return carrito.map((i) =>
			i.producto_id === producto.producto_id ? { ...i, cantidad: i.cantidad + cantidad } : i
		);
	}
	return [...carrito, { ...producto, cantidad }];
}

/** Fija la cantidad; 0 o menos quita la línea. */
export function cambiarCantidad(
	carrito: readonly ItemCarrito[],
	productoId: number,
	cantidad: number
): ItemCarrito[] {
	if (!Number.isFinite(cantidad) || cantidad <= 0) return quitarDelCarrito(carrito, productoId);
	return carrito.map((i) =>
		i.producto_id === productoId ? { ...i, cantidad: Math.trunc(cantidad) } : i
	);
}

export function quitarDelCarrito(carrito: readonly ItemCarrito[], productoId: number): ItemCarrito[] {
	return carrito.filter((i) => i.producto_id !== productoId);
}

export function totalesCarrito(
	carrito: readonly ItemCarrito[],
	descuento: number
): { subtotal: number; descuento: number; total: number } {
	const subtotal = carrito.reduce((t, i) => t + i.precio * i.cantidad, 0);
	return { subtotal, descuento, total: subtotal - descuento };
}

/** Mensaje de error del descuento, o `null` si es válido (mismos textos que la API). */
export function errorDescuento(descuento: number, subtotal: number): string | null {
	if (Number.isNaN(descuento)) return 'Ingresá un descuento numérico.';
	if (descuento < 0) return 'El descuento no puede ser negativo.';
	if (descuento > subtotal) return 'El descuento no puede superar el subtotal.';
	return null;
}

export function lineasParaVenta(
	carrito: readonly ItemCarrito[]
): Array<{ producto_id: number; cantidad: number }> {
	return carrito.map((i) => ({ producto_id: i.producto_id, cantidad: i.cantidad }));
}
