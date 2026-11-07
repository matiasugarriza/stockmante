<script lang="ts">
	import type { FiltroActivo } from '#lib/api/productos';
	import type { Categoria } from '#lib/schemas/categorias';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Tarjeta from '../ui/Tarjeta.svelte';

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
	$effect(() => {
		texto = q;
	});

	const estados: { valor: FiltroActivo; etiqueta: string }[] = [
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' },
		{ valor: 'all', etiqueta: 'Todos' }
	];

	const opcionesCategorias = $derived([
		{ valor: '', etiqueta: 'Todas' },
		...categorias.map((categoria) => ({
			valor: String(categoria.id),
			etiqueta: categoria.activo ? categoria.nombre : `${categoria.nombre} (inactiva)`
		}))
	]);

	function enviar() {
		onFiltrar({ q: texto, categoriaId, activo });
	}

	function limpiar() {
		texto = '';
		onFiltrar({ q: '', categoriaId: '', activo });
	}
</script>

<Tarjeta>
	<form
		class="filtros"
		onsubmit={(event) => {
			event.preventDefault();
			enviar();
		}}
	>
		<div class="campo-busqueda">
			<Campo
				etiqueta="Buscar"
				nombre="q"
				tipo="search"
				marcador="Nombre o SKU"
				maxlength={160}
				bind:valor={texto}
			/>
		</div>
		<div class="campo-categoria">
			<Campo
				etiqueta="Categoría"
				nombre="categoria"
				tipo="select"
				opciones={opcionesCategorias}
				valor={categoriaId}
				onchange={(valor) => onFiltrar({ q: texto, categoriaId: valor, activo })}
			/>
		</div>
		<div class="acciones-filtro">
			<Boton tipo="submit">Buscar</Boton>
			<Boton variante="secundario" onclick={limpiar}>Limpiar</Boton>
		</div>
	</form>

	<div class="tabs" role="group" aria-label="Filtro de estado">
		{#each estados as estado (estado.valor)}
			<Boton
				variante={activo === estado.valor ? 'primario' : 'secundario'}
				tamano="sm"
				onclick={() => onFiltrar({ q: texto, categoriaId, activo: estado.valor })}
			>
				{estado.etiqueta}
			</Boton>
		{/each}
	</div>
</Tarjeta>

<style>
	.filtros {
		display: grid;
		grid-template-columns: minmax(220px, 2fr) 220px auto;
		gap: 12px;
		align-items: end;
	}

	.acciones-filtro {
		display: flex;
		gap: 8px;
		padding-bottom: 1px;
	}

	.tabs {
		display: flex;
		gap: 6px;
		margin-top: 14px;
		flex-wrap: wrap;
	}

	@media (max-width: 760px) {
		.filtros {
			grid-template-columns: 1fr;
		}
	}
</style>
