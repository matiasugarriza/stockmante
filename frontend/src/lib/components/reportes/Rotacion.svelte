<script lang="ts">
	import { obtenerRotacion, urlCsv } from '#lib/api/reportes';
	import type { Rotacion } from '#lib/schemas/reportes';
	import { mensajeError } from '#lib/utils/errores';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';

	let desde = $state('');
	let hasta = $state('');
	let reporte = $state<Rotacion | null>(null);
	let error = $state<string | null>(null);
	let cargando = $state(false);

	async function consultar() {
		cargando = true;
		error = null;
		try {
			reporte = await obtenerRotacion({ desde, hasta });
		} catch (e) {
			reporte = null;
			error = mensajeError(e, 'No se pudo obtener la rotación.');
		} finally {
			cargando = false;
		}
	}
</script>

<div class="filtros">
	<Campo etiqueta="Desde" nombre="desde" tipo="date" bind:valor={desde} />
	<Campo etiqueta="Hasta" nombre="hasta" tipo="date" bind:valor={hasta} />
	<Boton onclick={() => void consultar()} cargando={cargando}>Calcular rotación</Boton>
</div>

{#if error}<Aviso variante="error">{error}</Aviso>{/if}
{#if reporte}
	<p class="resumen"><a href={urlCsv('rotacion', { desde, hasta })}>Descargar CSV</a></p>
	<table class="tabla">
		<thead>
			<tr>
				<th>Producto</th><th class="num">Salidas</th><th class="num">Stock inicial</th>
				<th class="num">Stock final</th><th class="num">Promedio</th><th class="num">Índice</th>
			</tr>
		</thead>
		<tbody>
			{#each reporte.items as i (i.producto_id)}
				<tr>
					<td>{i.nombre}</td>
					<td class="num">{i.unidades_salidas}</td>
					<td class="num">{i.stock_inicial}</td>
					<td class="num">{i.stock_final}</td>
					<td class="num">{i.stock_promedio.toFixed(2)}</td>
					<td class="num">{i.indice_rotacion.toFixed(4)}</td>
				</tr>
			{:else}
				<tr><td colspan="6">Sin movimientos en el período.</td></tr>
			{/each}
		</tbody>
	</table>
{/if}

<style>
	.filtros {
		display: flex;
		gap: 12px;
		align-items: flex-end;
		flex-wrap: wrap;
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
</style>
