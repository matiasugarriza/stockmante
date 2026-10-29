<script lang="ts">
	import type { Producto } from '#lib/schemas/productos';

	interface Props {
		productos: Producto[];
		onEditar: (producto: Producto) => void;
		onAlternarEstado: (producto: Producto) => void;
	}

	let { productos, onEditar, onAlternarEstado }: Props = $props();

	const botonAccion =
		'rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100 disabled:opacity-60';

	function monto(valor: number): string {
		return valor.toFixed(2);
	}
</script>

<div class="overflow-x-auto rounded-lg border border-slate-200 bg-white">
	<table class="min-w-full divide-y divide-slate-200 text-sm">
		<thead class="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
			<tr>
				<th class="px-4 py-2 font-medium">Nombre</th>
				<th class="px-4 py-2 font-medium">SKU</th>
				<th class="px-4 py-2 font-medium">Categoría</th>
				<th class="px-4 py-2 text-right font-medium">Precio</th>
				<th class="px-4 py-2 text-right font-medium">Stock mín.</th>
				<th class="px-4 py-2 font-medium">Estado</th>
				<th class="px-4 py-2 text-right font-medium">Acciones</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each productos as producto (producto.id)}
				<tr>
					<td class="px-4 py-2">
						<p class="font-medium">{producto.nombre}</p>
						{#if producto.codigo_barras}
							<p class="text-xs text-slate-400">{producto.codigo_barras}</p>
						{/if}
					</td>
					<td class="px-4 py-2 font-mono text-xs text-slate-600">{producto.sku}</td>
					<td class="px-4 py-2 text-slate-600">{producto.categoria.nombre}</td>
					<td class="px-4 py-2 text-right text-slate-600">
						${monto(producto.precio_venta)}
						<span class="block text-xs text-slate-400">{producto.unidad_medida}</span>
					</td>
					<td class="px-4 py-2 text-right text-slate-600">{producto.stock_minimo}</td>
					<td class="px-4 py-2">
						<span
							class="rounded px-1.5 py-0.5 text-xs {producto.activo
								? 'bg-emerald-50 text-emerald-700'
								: 'bg-slate-100 text-slate-500'}"
						>
							{producto.activo ? 'Activo' : 'Inactivo'}
						</span>
					</td>
					<td class="px-4 py-2">
						<div class="flex flex-wrap justify-end gap-1">
							<button class={botonAccion} onclick={() => onEditar(producto)}>Editar</button>
							{#if producto.activo}
								<button class={botonAccion} onclick={() => onAlternarEstado(producto)}>
									Desactivar
								</button>
							{:else}
								<button class={botonAccion} onclick={() => onAlternarEstado(producto)}>
									Reactivar
								</button>
							{/if}
						</div>
					</td>
				</tr>
			{/each}
			{#if productos.length === 0}
				<tr>
					<td colspan="7" class="px-4 py-8 text-center text-slate-500">
						No hay productos para estos filtros.
					</td>
				</tr>
			{/if}
		</tbody>
	</table>
</div>
