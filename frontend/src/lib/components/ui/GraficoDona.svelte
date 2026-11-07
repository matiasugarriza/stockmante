<script lang="ts">
	import EstadoVacio from './EstadoVacio.svelte';
	import { arcoDona, type DatoGrafico } from '#lib/utils/graficos';
	import { formatearPorcentaje } from '#lib/utils/formato';

	interface Props {
		datos: DatoGrafico[];
		etiqueta: string;
		centro?: string;
	}

	let { datos, etiqueta, centro }: Props = $props();

	const colores = ['var(--ambar)', 'var(--exito)', 'var(--primario)', 'var(--teal)', 'var(--peligro)'];
	const coloreados = $derived(
		datos.map((dato, indice) => ({
			...dato,
			color: dato.color ?? colores[indice % colores.length]
		}))
	);
	const segmentos = $derived(arcoDona(coloreados));
	const resumen = $derived(
		segmentos
			.map((segmento) => `${segmento.etiqueta}: ${formatearPorcentaje(segmento.porcentaje, 0)}`)
			.join('; ')
	);
</script>

{#if segmentos.length === 0}
	<EstadoVacio
		titulo="Sin datos de stock"
		descripcion="Cuando haya productos cargados vas a ver la proporción acá."
	/>
{:else}
	<div class="dona">
		<svg viewBox="0 0 200 200" role="img" aria-label={etiqueta}>
			<title>{etiqueta}</title>
			<desc>{resumen}</desc>
			{#each segmentos as segmento (segmento.etiqueta)}
				<path d={segmento.d} style:fill={segmento.color} />
			{/each}
			{#if centro}
				<text x="100" y="98" text-anchor="middle" class="centro">{centro}</text>
				<text x="100" y="114" text-anchor="middle" class="centro-etiqueta">total</text>
			{/if}
		</svg>
		<ul class="leyenda">
			{#each segmentos as segmento (segmento.etiqueta)}
				<li>
					<span class="punto" style:background={segmento.color}></span>
					<span class="nombre">{segmento.etiqueta}</span>
					<strong>{formatearPorcentaje(segmento.porcentaje, 0)}</strong>
				</li>
			{/each}
		</ul>
	</div>
{/if}

<style>
	.dona {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
	}

	svg {
		display: block;
		width: min(200px, 100%);
		height: auto;
	}

	.centro {
		fill: var(--texto);
		font-size: 26px;
		font-weight: 700;
		font-family: var(--fuente-sans);
		font-variant-numeric: tabular-nums;
	}

	.centro-etiqueta {
		fill: var(--texto-tenue);
		font-size: 10px;
		font-family: var(--fuente-sans);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}

	.leyenda {
		display: flex;
		flex-direction: column;
		gap: 6px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.leyenda li {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.nombre {
		flex: 1;
	}

	.leyenda strong {
		color: var(--texto);
		font-variant-numeric: tabular-nums;
	}

	.punto {
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}
</style>
