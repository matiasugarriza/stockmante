<script lang="ts">
	import type { FiltroActivo } from '#lib/api/productos';
	import type { Categoria } from '#lib/schemas/categorias';

	interface Props {
		q: string;
		categoriaId: string;
		activo: FiltroActivo;
		categorias: Categoria[];
		onFiltrar: (filtros: { q: string; categoriaId: string; activo: FiltroActivo }) => void;
	}

	let { q, categoriaId, activo, categorias, onFiltrar }: Props = $props();

	// El input se inicializa con el filtro vigente; el `$effect` siguiente lo
	// vuelve a sincronizar si el padre reinicia los filtros.
	// svelte-ignore state_referenced_locally
	let texto = $state(q);
	// Si el padre reinicia los filtros, el input acompaña.
	$effect(() => {
		texto = q;
	});

	const estados: { valor: FiltroActivo; etiqueta: string }[] = [
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' },
		{ valor: 'all', etiqueta: 'Todos' }
	];
</script>

<div class="flex flex-col gap-3 rounded-lg border border-slate-200 bg-white p-3">
	<form
		class="flex flex-wrap items-end gap-2"
		onsubmit={(e) => {
			e.preventDefault();
			onFiltrar({ q: texto, categoriaId, activo });
		}}
	>
		<label class="flex min-w-48 flex-1 flex-col gap-1 text-sm">
			<span class="font-medium">Buscar</span>
			<input
				class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
				bind:value={texto}
				placeholder="Nombre o SKU"
				maxlength="160"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm">
			<span class="font-medium">Categoría</span>
			<select
				class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
				value={categoriaId}
				onchange={(e) => onFiltrar({ q: texto, categoriaId: e.currentTarget.value, activo })}
			>
				<option value="">Todas</option>
				{#each categorias as categoria (categoria.id)}
					<option value={String(categoria.id)}>
						{categoria.nombre}{categoria.activo ? '' : ' (inactiva)'}
					</option>
				{/each}
			</select>
		</label>
		<button
			type="submit"
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
		>
			Buscar
		</button>
		<button
			type="button"
			class="rounded border border-slate-300 px-3 py-2 text-sm hover:bg-slate-100"
			onclick={() => {
				texto = '';
				onFiltrar({ q: '', categoriaId: '', activo });
			}}
		>
			Limpiar
		</button>
	</form>

	<div class="flex items-center gap-1">
		{#each estados as estado (estado.valor)}
			<button
				class="rounded px-3 py-1 text-sm {activo === estado.valor
					? 'bg-slate-800 text-white'
					: 'border border-slate-300 text-slate-600 hover:bg-slate-100'}"
				onclick={() => onFiltrar({ q: texto, categoriaId, activo: estado.valor })}
			>
				{estado.etiqueta}
			</button>
		{/each}
	</div>
</div>
