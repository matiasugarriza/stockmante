<script lang="ts">
	import { obtenerVentas, urlCsv } from '#lib/api/reportes';
	import type { VentasReporte } from '#lib/schemas/reportes';
	import { mensajeError } from '#lib/utils/errores';
	import { formatearMoneda } from '#lib/utils/formato';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';

	let desde = $state('');
	let hasta = $state('');
	let reporte = $state<VentasReporte | null>(null);
	let error = $state<string | null>(null);
	let cargando = $state(false);

	async function consultar() {
		cargando = true;
		error = null;
		try {
			reporte = await obtenerVentas({ desde, hasta });
		} catch (e) {
			reporte = null;
			error = mensajeError(e, 'No se pudo obtener el resumen de ventas.');
		} finally {
			cargando = false;
		}
	}
</script>

<div class="filtros">
	<Campo etiqueta="Desde" nombre="desde" tipo="date" bind:valor={desde} />
	<Campo etiqueta="Hasta" nombre="hasta" tipo="date" bind:valor={hasta} />
	<Boton onclick={() => void consultar()} cargando={cargando}>Ver ventas</Boton>
</div>

{#if error}<Aviso variante="error">{error}</Aviso>{/if}
{#if reporte}
	<p class="resumen">
		{reporte.cantidad_ventas} ventas confirmadas · total <strong>{formatearMoneda(reporte.total)}</strong> ·
		<a href={urlCsv('ventas', { desde, hasta, agrupar: 'dia' })}>CSV por día</a> ·
		<a href={urlCsv('ventas', { desde, hasta, agrupar: 'producto' })}>CSV por producto</a>
	</p>
	<div class="columnas">
		<table class="tabla">
			<thead><tr><th>Día</th><th class="num">Ventas</th><th class="num">Total</th></tr></thead>
			<tbody>
				{#each reporte.por_dia as d (d.dia)}
					<tr><td>{d.dia}</td><td class="num">{d.cantidad_ventas}</td><td class="num">{formatearMoneda(d.total)}</td></tr>
				{:else}
					<tr><td colspan="3">Sin ventas en el período.</td></tr>
				{/each}
			</tbody>
		</table>
		<table class="tabla">
			<thead><tr><th>Producto</th><th class="num">Unidades</th><th class="num">Total</th></tr></thead>
			<tbody>
				{#each reporte.por_producto as p (p.producto_id)}
					<tr><td>{p.nombre}</td><td class="num">{p.unidades}</td><td class="num">{formatearMoneda(p.total)}</td></tr>
				{:else}
					<tr><td colspan="3">Sin ventas en el período.</td></tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style>
	.filtros {
		display: flex;
		gap: 12px;
		align-items: flex-end;
		flex-wrap: wrap;
	}

	.columnas {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}

	.tabla {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	.tabla th,
	.tabla td {
		padding: 6px 8px;
		border-top: 1px solid var(--borde);
		text-align: left;
	}

	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	@media (max-width: 900px) {
		.columnas {
			grid-template-columns: 1fr;
		}
	}
</style>
