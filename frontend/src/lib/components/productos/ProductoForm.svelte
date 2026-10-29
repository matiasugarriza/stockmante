<script lang="ts">
	import {
		CrearProductoSchema,
		type CrearProductoInput,
		type Producto,
		type UnidadMedida
	} from '#lib/schemas/productos';
	import type { Categoria } from '#lib/schemas/categorias';

	interface Props {
		producto: Producto | null;
		categorias: Categoria[];
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: CrearProductoInput) => void;
		onCerrar: () => void;
	}

	let { producto, categorias, guardando, error, onGuardar, onCerrar }: Props = $props();

	// El padre monta el formulario al abrirlo, con `producto` ya definido.
	// svelte-ignore state_referenced_locally
	let nombre = $state(producto?.nombre ?? '');
	// svelte-ignore state_referenced_locally
	let sku = $state(producto?.sku ?? '');
	// svelte-ignore state_referenced_locally
	let codigoBarras = $state(producto?.codigo_barras ?? '');
	// svelte-ignore state_referenced_locally
	let categoriaId = $state(producto ? String(producto.categoria.id) : '');
	// svelte-ignore state_referenced_locally
	let unidad = $state<UnidadMedida>(producto?.unidad_medida ?? 'UNIDAD');
	// svelte-ignore state_referenced_locally
	let costo = $state(producto ? String(producto.costo) : '');
	// svelte-ignore state_referenced_locally
	let precio = $state(producto ? String(producto.precio_venta) : '');
	// svelte-ignore state_referenced_locally
	let stockMinimo = $state(producto ? String(producto.stock_minimo) : '0');
	let errorValidacion = $state<string | null>(null);

	// Al editar se ofrece la categoría actual aunque esté inactiva: el backend
	// permite conservarla, pero no asignar una distinta inactiva.
	const opciones = $derived(
		categorias.filter((c) => c.activo || c.id === producto?.categoria.id)
	);

	function numero(valor: string): number {
		return Number(valor.replace(',', '.'));
	}

	function enviar() {
		errorValidacion = null;
		if (!costo.trim() || !precio.trim() || !stockMinimo.trim()) {
			errorValidacion = 'Completá costo, precio de venta y stock mínimo.';
			return;
		}
		const datos = CrearProductoSchema.safeParse({
			nombre,
			sku,
			codigo_barras: codigoBarras,
			categoria_id: Number(categoriaId),
			unidad_medida: unidad,
			costo: numero(costo),
			precio_venta: numero(precio),
			stock_minimo: numero(stockMinimo)
		});
		if (!datos.success) {
			const primero = datos.error.issues[0];
			errorValidacion = primero ? primero.message : 'Revisá los datos';
			return;
		}
		onGuardar(datos.data);
	}

	const etiquetasUnidad: { valor: UnidadMedida; etiqueta: string }[] = [
		{ valor: 'UNIDAD', etiqueta: 'Unidad' },
		{ valor: 'KG', etiqueta: 'Kilogramo' },
		{ valor: 'METRO', etiqueta: 'Metro' },
		{ valor: 'CAJA', etiqueta: 'Caja' }
	];
</script>

<div class="fixed inset-0 z-10 flex items-center justify-center bg-slate-900/40 p-4">
	<div class="max-h-full w-full max-w-2xl overflow-y-auto rounded-xl border border-slate-200 bg-white p-5 shadow-lg">
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h2 class="text-lg font-semibold">
					{producto ? 'Editar producto' : 'Nuevo producto'}
				</h2>
				<p class="text-xs text-slate-500">
					El SKU se guarda en mayúsculas y es único incluso para productos inactivos.
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
			class="grid grid-cols-1 gap-3 sm:grid-cols-2"
			onsubmit={(e) => {
				e.preventDefault();
				enviar();
			}}
		>
			<label class="flex flex-col gap-1 text-sm sm:col-span-2">
				<span class="font-medium">Nombre</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={nombre}
					maxlength="160"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">SKU</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 uppercase focus:border-slate-500 focus:outline-none"
					bind:value={sku}
					maxlength="30"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">
					Código de barras <span class="text-slate-400">(opcional)</span>
				</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={codigoBarras}
					maxlength="64"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Categoría</span>
				<select
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={categoriaId}
					required
				>
					<option value="" disabled>Elegí una categoría</option>
					{#each opciones as categoria (categoria.id)}
						<option value={String(categoria.id)}>
							{categoria.nombre}{categoria.activo ? '' : ' (inactiva)'}
						</option>
					{/each}
				</select>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Unidad de medida</span>
				<select
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={unidad}
				>
					{#each etiquetasUnidad as opcion (opcion.valor)}
						<option value={opcion.valor}>{opcion.etiqueta} ({opcion.valor})</option>
					{/each}
				</select>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Costo</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={costo}
					inputmode="decimal"
					placeholder="0.00"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Precio de venta</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={precio}
					inputmode="decimal"
					placeholder="0.00"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Stock mínimo</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={stockMinimo}
					inputmode="numeric"
					required
				/>
			</label>

			{#if error || errorValidacion}
				<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700 sm:col-span-2">
					{error ?? errorValidacion}
				</p>
			{/if}

			<div class="mt-1 flex justify-end gap-2 sm:col-span-2">
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
					{guardando ? 'Guardando…' : producto ? 'Guardar cambios' : 'Crear producto'}
				</button>
			</div>
		</form>
	</div>
</div>
