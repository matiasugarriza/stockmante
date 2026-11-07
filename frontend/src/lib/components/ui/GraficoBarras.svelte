<script lang="ts">
	import EstadoVacio from './EstadoVacio.svelte';
	import { escalaBarras, type DatoGrafico } from '#lib/utils/graficos';
	import { formatearNumero } from '#lib/utils/formato';

	interface Props {
		datos: DatoGrafico[];
		etiqueta: string;
		alto?: number;
	}

	let { datos, etiqueta, alto = 180 }: Props = $props();

	const ancho = 600;
	const separacion = 28;
	const escala = $derived(escalaBarras(datos, { ancho, alto, separacion }));
	const hayDatos = $derived(datos.some((dato) => dato.valor > 0));
	const resumen = $derived(
		datos.map((dato) => `${dato.etiqueta}: ${formatearNumero(dato.valor)}`).join('; ')
	);
</script>

{#if !hayDatos}
	<EstadoVacio
		titulo="Sin movimientos en el período"
		descripcion="Cuando registres entradas, salidas o mermas vas a ver la distribución acá."
	/>
{:else}
	<div class="grafico">
		<svg viewBox={`0 0 ${ancho} ${alto + 24}`} role="img" aria-label={etiqueta}>
			<title>{etiqueta}</title>
			<desc>{resumen}</desc>
			{#each escala.barras as barra (barra.etiqueta)}
				<rect
					x={barra.x}
					y={barra.y}
					width={barra.ancho}
					height={Math.max(2, barra.alto)}
					rx="5"
					style:fill={barra.color ?? 'var(--primario)'}
				/>
				<text
					x={barra.x + barra.ancho / 2}
					y={Math.max(12, barra.y - 6)}
					text-anchor="middle"
					class="valor"
				>
					{formatearNumero(barra.valor)}
				</text>
				<text x={barra.x + barra.ancho / 2} y={alto + 17} text-anchor="middle" class="eje">
					{barra.etiqueta}
				</text>
			{/each}
		</svg>
		<ul class="leyenda">
			{#each datos as dato, indice (dato.etiqueta)}
				<li>
					<span
						class="punto"
						style:background={dato.color ??
							['var(--primario)', 'var(--ambar)', 'var(--exito)', 'var(--peligro)'][indice % 4]}
					></span>
					{dato.etiqueta}
					<strong>{formatearNumero(dato.valor)}</strong>
				</li>
			{/each}
		</ul>
	</div>
{/if}

<style>
	.grafico {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	svg {
		display: block;
		width: 100%;
		height: auto;
	}

	.valor {
		fill: var(--texto-suave);
		font-size: 11px;
		font-weight: 600;
		font-family: var(--fuente-sans);
		font-variant-numeric: tabular-nums;
	}

	.eje {
		fill: var(--texto-tenue);
		font-size: 10.5px;
		font-family: var(--fuente-sans);
	}

	.leyenda {
		display: flex;
		flex-wrap: wrap;
		gap: 8px 16px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.leyenda li {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--texto-suave);
		font-size: 12px;
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
