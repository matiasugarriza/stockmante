<script lang="ts">
	import { onMount } from 'svelte';
	import { obtenerStockValorizado, urlCsv } from '#lib/api/reportes';
	import type { StockValorizado } from '#lib/schemas/reportes';
	import { mensajeError } from '#lib/utils/errores';
	import { formatearMoneda } from '#lib/utils/formato';
	import Aviso from '#lib/components/ui/Aviso.svelte';

	let reporte = $state<StockValorizado | null>(null);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			reporte = await obtenerStockValorizado();
		} catch (e) {
			error = mensajeError(e, 'No se pudo obtener el stock valorizado.');
		}
	});
</script>

{#if error}<Aviso variante="error">{error}</Aviso>{/if}
{#if reporte}
	<p class="resumen">
		Valor total del inventario <strong>{formatearMoneda(reporte.total)}</strong> ·
		<a href={urlCsv('stock-valorizado')}>Descargar CSV</a>
	</p>
	<table class="tabla">
		<thead>
			<tr><th>Producto</th><th>SKU</th><th class="num">Cantidad</th><th class="num">Costo</th><th class="num">Valor</th></tr>
		</thead>
		<tbody>
			{#each reporte.items as i (i.producto_id)}
				<tr>
					<td>{i.nombre}</td>
					<td>{i.sku}</td>
					<td class="num">{i.cantidad}</td>
					<td class="num">{formatearMoneda(i.costo_unitario)}</td>
					<td class="num">{formatearMoneda(i.valor)}</td>
				</tr>
			{:else}
				<tr><td colspan="5">No hay stock registrado.</td></tr>
			{/each}
		</tbody>
	</table>
{/if}

<style>
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
</style>
