<script lang="ts">
	import Kardex from '#lib/components/reportes/Kardex.svelte';
	import Rotacion from '#lib/components/reportes/Rotacion.svelte';
	import StockValorizado from '#lib/components/reportes/StockValorizado.svelte';
	import Ventas from '#lib/components/reportes/Ventas.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	type Reporte = 'kardex' | 'stock-valorizado' | 'rotacion' | 'ventas';
	const REPORTES: { id: Reporte; etiqueta: string }[] = [
		{ id: 'kardex', etiqueta: 'Kardex' },
		{ id: 'stock-valorizado', etiqueta: 'Stock valorizado' },
		{ id: 'rotacion', etiqueta: 'Rotación' },
		{ id: 'ventas', etiqueta: 'Ventas' }
	];

	let actual = $state<Reporte>('kardex');
</script>

<svelte:head>
	<title>Reportes · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Reportes"
		descripcion="Kardex, valorización, rotación y ventas por período (días en hora de Argentina). Todos se descargan en CSV."
	/>

	<div class="pestanas" role="tablist" aria-label="Reportes">
		{#each REPORTES as r (r.id)}
			<button
				type="button"
				role="tab"
				aria-selected={actual === r.id}
				class:activa={actual === r.id}
				onclick={() => (actual = r.id)}
			>
				{r.etiqueta}
			</button>
		{/each}
	</div>

	<Tarjeta>
		{#if actual === 'kardex'}
			<Kardex />
		{:else if actual === 'stock-valorizado'}
			<StockValorizado />
		{:else if actual === 'rotacion'}
			<Rotacion />
		{:else}
			<Ventas />
		{/if}
	</Tarjeta>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.pestanas {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.pestanas button {
		padding: 6px 12px;
		border: 1px solid var(--borde);
		border-radius: 999px;
		background: none;
		color: inherit;
		font: inherit;
		cursor: pointer;
	}

	.pestanas button.activa {
		border-color: var(--primario);
		color: var(--primario);
		font-weight: 600;
	}
</style>
