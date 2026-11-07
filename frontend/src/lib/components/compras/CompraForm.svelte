<script lang="ts">
	import { CrearCompraSchema, type CompraDetalle, type CrearCompraInput } from '#lib/schemas/compras';
	import type { Deposito } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import type { Proveedor } from '#lib/schemas/proveedores';

	interface Props {
		compra: CompraDetalle | null;
		proveedores: Proveedor[];
		depositos: Deposito[];
		productos: Producto[];
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: CrearCompraInput) => void;
		onCerrar: () => void;
	}

	let { compra, proveedores, depositos, productos, guardando, error, onGuardar, onCerrar }: Props =
		$props();

	interface LineaForm {
		productoId: string;
		cantidad: string;
		costo: string;
	}

	// El padre monta el formulario al abrirlo, con `compra` ya resuelta.
	// svelte-ignore state_referenced_locally
	let proveedorId = $state(compra ? String(compra.compra.proveedor_id) : '');
	// svelte-ignore state_referenced_locally
	let depositoId = $state(compra ? String(compra.compra.deposito_id) : '');
	// svelte-ignore state_referenced_locally
	let numero = $state(compra?.compra.numero ?? '');
	// svelte-ignore state_referenced_locally
	let observacion = $state(compra?.compra.observacion ?? '');
	// svelte-ignore state_referenced_locally
	let lineas = $state<LineaForm[]>(
		compra && compra.lineas.length > 0
			? compra.lineas.map((linea) => ({
					productoId: String(linea.producto_id),
					cantidad: String(linea.cantidad),
					costo: String(linea.costo_unitario)
				}))
			: [{ productoId: '', cantidad: '1', costo: '' }]
	);
	let errorValidacion = $state<string | null>(null);

	const total = $derived(
		lineas.reduce((acumulado, linea) => {
			const cantidad = Number(linea.cantidad.replace(',', '.'));
			const costo = Number(linea.costo.replace(',', '.'));
			if (!Number.isFinite(cantidad) || !Number.isFinite(costo)) return acumulado;
			return acumulado + cantidad * costo;
		}, 0)
	);

	function agregarLinea() {
		lineas.push({ productoId: '', cantidad: '1', costo: '' });
	}

	function quitarLinea(indice: number) {
		if (lineas.length === 1) return;
		lineas.splice(indice, 1);
	}

	function elegirProducto(indice: number, productoId: string) {
		lineas[indice].productoId = productoId;
		const producto = productos.find((p) => String(p.id) === productoId);
		if (producto && !lineas[indice].costo.trim()) {
			lineas[indice].costo = String(producto.costo);
		}
	}

	function enviar() {
		errorValidacion = null;
		// El backend rechaza líneas repetidas del mismo producto (unique en
		// movimiento_lineas); lo validamos antes para no mostrar un 500.
		const productosElegidos = lineas
			.map((linea) => linea.productoId)
			.filter((productoId) => productoId !== '');
		if (new Set(productosElegidos).size !== productosElegidos.length) {
			errorValidacion = 'No repitas el mismo producto en dos líneas.';
			return;
		}
		const datos = CrearCompraSchema.safeParse({
			proveedor_id: Number(proveedorId),
			deposito_id: Number(depositoId),
			numero,
			observacion,
			lineas: lineas.map((linea) => ({
				producto_id: Number(linea.productoId),
				cantidad: Number(linea.cantidad.replace(',', '.')),
				costo_unitario: Number(linea.costo.replace(',', '.'))
			}))
		});
		if (!datos.success) {
			const primero = datos.error.issues[0];
			errorValidacion = primero ? primero.message : 'Revisá los datos';
			return;
		}
		onGuardar(datos.data);
	}
</script>

<div class="fixed inset-0 z-10 flex items-center justify-center bg-slate-900/40 p-4">
	<div
		class="max-h-full w-full max-w-3xl overflow-y-auto rounded-xl border border-slate-200 bg-white p-5 shadow-lg"
	>
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h2 class="text-lg font-semibold">
					{compra ? 'Editar compra en borrador' : 'Nueva compra'}
				</h2>
				<p class="text-xs text-slate-500">
					La compra nace en BORRADOR. Confirmar la recepción recién ahí mueve el stock.
				</p>
			</div>
			<button
				type="button"
				class="rounded px-2 py-1 text-sm text-slate-500 hover:bg-slate-100"
				onclick={onCerrar}
				disabled={guardando}
			>
				Cerrar
			</button>
		</div>

		<form
			class="flex flex-col gap-3"
			onsubmit={(e) => {
				e.preventDefault();
				enviar();
			}}
		>
			<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Proveedor</span>
					<select
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						bind:value={proveedorId}
						required
					>
						<option value="" disabled>Elegí un proveedor activo</option>
						{#each proveedores as proveedor (proveedor.id)}
							<option value={String(proveedor.id)}>{proveedor.nombre}</option>
						{/each}
					</select>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Depósito de destino</span>
					<select
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						bind:value={depositoId}
						required
					>
						<option value="" disabled>Elegí un depósito activo</option>
						{#each depositos as deposito (deposito.id)}
							<option value={String(deposito.id)}>
								{deposito.nombre} ({deposito.identificador})
							</option>
						{/each}
					</select>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Número <span class="text-slate-400">(opcional)</span></span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						bind:value={numero}
						maxlength="40"
						placeholder="OC-0001"
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Observación <span class="text-slate-400">(opcional)</span></span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						bind:value={observacion}
						maxlength="500"
					/>
				</label>
			</div>

			<div class="rounded border border-slate-200">
				<div class="flex items-center justify-between border-b border-slate-200 bg-slate-50 px-3 py-2">
					<span class="text-sm font-medium">Líneas</span>
					<button
						type="button"
						class="rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100"
						onclick={agregarLinea}
					>
						Agregar línea
					</button>
				</div>
				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead>
							<tr class="text-slate-500">
								<th class="p-2 text-left">Producto</th>
								<th class="p-2 text-left">Cantidad</th>
								<th class="p-2 text-left">Costo unitario</th>
								<th class="p-2 text-right">Subtotal</th>
								<th class="p-2"></th>
							</tr>
						</thead>
						<tbody>
							{#each lineas as linea, indice (indice)}
								{@const producto = productos.find((p) => String(p.id) === linea.productoId)}
								{@const subtotal =
									Number(linea.cantidad.replace(',', '.')) *
									Number(linea.costo.replace(',', '.'))}
								<tr class="border-t border-slate-100">
									<td class="p-2">
										<select
											class="w-full rounded border border-slate-300 px-2 py-1"
											value={linea.productoId}
											onchange={(e) => elegirProducto(indice, e.currentTarget.value)}
											required
										>
											<option value="" disabled>Elegí un producto</option>
											{#each productos as opcion (opcion.id)}
												<option value={String(opcion.id)}>{opcion.nombre} ({opcion.sku})</option>
											{/each}
										</select>
									</td>
									<td class="p-2">
										<input
											class="w-24 rounded border border-slate-300 px-2 py-1"
											bind:value={linea.cantidad}
											inputmode="numeric"
											required
										/>
									</td>
									<td class="p-2">
										<input
											class="w-28 rounded border border-slate-300 px-2 py-1"
											bind:value={linea.costo}
											inputmode="decimal"
											placeholder={producto ? String(producto.costo) : '0.00'}
											required
										/>
									</td>
									<td class="p-2 text-right">
										{Number.isFinite(subtotal) ? subtotal.toFixed(2) : '—'}
									</td>
									<td class="p-2 text-right">
										<button
											type="button"
											class="rounded px-2 py-1 text-xs text-red-600 hover:bg-red-50 disabled:opacity-40"
											onclick={() => quitarLinea(indice)}
											disabled={lineas.length === 1}
										>
											Quitar
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>

			<div class="flex items-center justify-end gap-3 text-sm">
				<span class="text-slate-500">Total estimado</span>
				<span class="text-lg font-semibold">{total.toFixed(2)}</span>
			</div>

			{#if error || errorValidacion}
				<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700">{error ?? errorValidacion}</p>
			{/if}

			<div class="flex justify-end gap-2">
				<button
					type="button"
					class="rounded border border-slate-300 px-3 py-2 text-sm hover:bg-slate-100"
					onclick={onCerrar}
					disabled={guardando}
				>
					Cancelar
				</button>
				<button
					type="submit"
					class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-60"
					disabled={guardando}
				>
					{guardando ? 'Guardando...' : compra ? 'Guardar cambios' : 'Crear compra'}
				</button>
			</div>
		</form>
	</div>
</div>