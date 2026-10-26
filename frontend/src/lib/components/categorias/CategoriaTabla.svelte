<script lang="ts">
	import type { Categoria } from '#lib/schemas/categorias';

	interface Props {
		categorias: Categoria[];
		onEditar: (categoria: Categoria) => void;
		onAlternarEstado: (categoria: Categoria) => void;
	}

	let { categorias, onEditar, onAlternarEstado }: Props = $props();

	const botonAccion =
		'rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100 disabled:opacity-60';
</script>

<div class="overflow-x-auto rounded-lg border border-slate-200 bg-white">
	<table class="min-w-full divide-y divide-slate-200 text-sm">
		<thead class="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
			<tr>
				<th class="px-4 py-2 font-medium">Nombre</th>
				<th class="px-4 py-2 font-medium">Descripción</th>
				<th class="px-4 py-2 font-medium">Estado</th>
				<th class="px-4 py-2 text-right font-medium">Acciones</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each categorias as categoria (categoria.id)}
				<tr>
					<td class="px-4 py-2 font-medium">{categoria.nombre}</td>
					<td class="px-4 py-2 text-slate-600">{categoria.descripcion ?? '—'}</td>
					<td class="px-4 py-2">
						<span
							class="rounded px-1.5 py-0.5 text-xs {categoria.activo
								? 'bg-emerald-50 text-emerald-700'
								: 'bg-slate-100 text-slate-500'}"
						>
							{categoria.activo ? 'Activa' : 'Inactiva'}
						</span>
					</td>
					<td class="px-4 py-2">
						<div class="flex flex-wrap justify-end gap-1">
							<button class={botonAccion} onclick={() => onEditar(categoria)}>Editar</button>
							{#if categoria.activo}
								<button class={botonAccion} onclick={() => onAlternarEstado(categoria)}>
									Desactivar
								</button>
							{:else}
								<button class={botonAccion} onclick={() => onAlternarEstado(categoria)}>
									Reactivar
								</button>
							{/if}
						</div>
					</td>
				</tr>
			{/each}
			{#if categorias.length === 0}
				<tr>
					<td colspan="4" class="px-4 py-8 text-center text-slate-500">
						No hay categorías para este filtro.
					</td>
				</tr>
			{/if}
		</tbody>
	</table>
</div>
