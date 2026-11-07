<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import Aviso from '../ui/Aviso.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';

	interface Props {
		items: T[];
		etiqueta: string;
		cargando?: boolean;
		error?: string | null;
		vacio?: Snippet;
		tarjeta: Snippet<[T]>;
		pie?: Snippet;
		compacta?: boolean;
	}

	let {
		items,
		etiqueta,
		cargando = false,
		error = null,
		vacio,
		tarjeta,
		pie,
		compacta = false
	}: Props = $props();
</script>

{#if error}
	<Aviso variante="error" titulo="No se pudo cargar">{error}</Aviso>
{:else}
	<p class="conteo" aria-live="polite">
		{#if cargando}Cargando…{:else}{items.length} resultados{/if}
	</p>
	<ul class="lista" class:compacta aria-label={etiqueta} aria-busy={cargando ? 'true' : undefined}>
		{#if cargando}
			{#each [1, 2, 3] as numero (numero)}
				<li class="esqueleto"></li>
			{/each}
		{:else if items.length === 0}
			<li class="vacia">
				{#if vacio}
					{@render vacio()}
				{:else}
					<EstadoVacio
						titulo="Sin resultados"
						descripcion="Probá con otro filtro o agregá el primer registro."
					/>
				{/if}
			</li>
		{:else}
			{#each items as item, posicion (posicion)}
				<li class="fila">{@render tarjeta(item)}</li>
			{/each}
		{/if}
	</ul>
	{#if pie}
		<div class="pie">{@render pie()}</div>
	{/if}
{/if}

<style>
	.conteo {
		margin: 0 0 8px;
		color: var(--texto-tenue);
		font-size: 12px;
	}

	.lista {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.lista.compacta {
		gap: 6px;
	}

	.esqueleto {
		height: 86px;
		border-radius: var(--radio-tarjeta-movil);
		background: var(--superficie-2);
	}

	.vacia {
		list-style: none;
	}

	.pie {
		margin-top: 12px;
	}
</style>
