<script lang="ts">
	import type { Snippet } from 'svelte';
	import { contarFiltrosActivos, resumenFiltros, type FiltroActivo } from '#lib/utils/filtros';
	import Icono from '../ui/Icono.svelte';

	interface Props {
		chips?: FiltroActivo[];
		onlimpiar?: () => void;
		children: Snippet;
	}

	let { chips = [], onlimpiar, children }: Props = $props();

	let abierto = $state(false);
	const activos = $derived(contarFiltrosActivos(chips));
	const idPanel = `filtros-movil-${Math.random().toString(36).slice(2, 8)}`;
</script>

<div class="filtros">
	<div class="cabecera">
		<button
			type="button"
			class="disparador"
			aria-expanded={abierto}
			aria-controls={idPanel}
			onclick={() => (abierto = !abierto)}
		>
			<Icono nombre="menu" tamano={16} />
			<span>{resumenFiltros(activos)}</span>
			<span class="flecha" class:abierta={abierto} aria-hidden="true"
				><Icono nombre="flecha" tamano={14} /></span
			>
		</button>
		{#if activos > 0 && onlimpiar}
			<button type="button" class="limpiar" onclick={onlimpiar}>Limpiar</button>
		{/if}
	</div>

	{#if activos > 0}
		<ul class="chips" aria-label="Filtros activos">
			{#each chips as chip (chip.etiqueta)}
				{#if chip.valor}
					<li class="chip">{chip.etiqueta}: {chip.valor}</li>
				{/if}
			{/each}
		</ul>
	{/if}

	<div id={idPanel} class="panel" hidden={!abierto}>
		{@render children()}
	</div>
</div>

<style>
	.filtros {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 10px;
	}

	.cabecera {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.disparador {
		display: inline-flex;
		flex: 1;
		align-items: center;
		gap: 8px;
		min-height: var(--toque-min);
		padding: 0 12px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto-suave);
		font-family: var(--fuente-sans);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
	}

	.disparador:hover {
		border-color: var(--primario);
		color: var(--texto);
	}

	.flecha {
		display: inline-flex;
		margin-left: auto;
		transform: rotate(90deg);
		transition: transform 120ms ease;
	}

	.flecha.abierta {
		transform: rotate(-90deg);
	}

	.limpiar {
		flex: none;
		min-height: var(--toque-min);
		padding: 0 12px;
		border: 1px solid var(--peligro);
		border-radius: var(--radio-control);
		background: var(--peligro-suave);
		color: var(--peligro-texto);
		font-family: var(--fuente-sans);
		font-size: 12.5px;
		font-weight: 600;
		cursor: pointer;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.chip {
		padding: 3px 9px;
		border: 1px solid var(--primario);
		border-radius: var(--radio-pildora);
		background: var(--primario-suave);
		color: var(--primario-texto);
		font-size: 11.5px;
	}

	.panel {
		display: grid;
		grid-template-columns: 1fr;
		gap: 10px;
		padding: 12px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta-movil);
		background: var(--superficie);
	}

	.panel[hidden] {
		display: none;
	}

	@media (min-width: 560px) {
		.panel {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}
</style>
