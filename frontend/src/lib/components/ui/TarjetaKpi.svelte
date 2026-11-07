<script lang="ts">
	import Esqueleto from './Esqueleto.svelte';
	import Icono from './Icono.svelte';
	import type { NombreIcono } from './iconos';

	type Variante = 'neutro' | 'aviso' | 'peligro' | 'exito' | 'info';

	interface Props {
		etiqueta: string;
		valor: string;
		contexto?: string;
		icono?: NombreIcono;
		cargando?: boolean;
		variante?: Variante;
		/** Franja de métricas sin tarjeta, como los indicadores de las muestras (FR-099). */
		plano?: boolean;
	}

	let {
		etiqueta,
		valor,
		contexto,
		icono,
		cargando = false,
		variante = 'neutro',
		plano = false
	}: Props = $props();
</script>

<article class="kpi {variante}" class:plano>
	<header>
		<span class="etiqueta">{etiqueta}</span>
		{#if icono && !plano}
			<span class="marca" aria-hidden="true"><Icono nombre={icono} tamano={17} /></span>
		{/if}
	</header>
	{#if cargando}
		<Esqueleto alto="30px" ancho="70%" />
	{:else}
		<p class="valor">{valor}</p>
	{/if}
	{#if contexto}<p class="contexto">{contexto}</p>{/if}
</article>

<style>
	.kpi {
		display: flex;
		flex-direction: column;
		gap: 8px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta);
		background: var(--superficie);
		box-shadow: var(--sombra-tarjeta);
		padding: 16px 18px;
		min-height: 118px;
	}

	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.etiqueta {
		color: var(--texto-suave);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.marca {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: 8px;
		background: var(--superficie-2);
		color: var(--texto-suave);
	}

	.valor {
		margin: 0;
		color: var(--texto);
		font-size: 27px;
		font-weight: 700;
		line-height: 1.1;
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.02em;
	}

	.contexto {
		margin: 0;
		color: var(--texto-tenue);
		font-size: 12px;
		line-height: 1.35;
	}

	.aviso .marca {
		background: var(--ambar-suave);
		color: var(--ambar-texto);
	}

	.peligro .marca {
		background: var(--peligro-suave);
		color: var(--peligro-texto);
	}

	.exito .marca {
		background: var(--exito-suave);
		color: var(--exito-texto);
	}

	.info .marca {
		background: var(--info-suave);
		color: var(--primario-texto);
	}

	.plano {
		gap: 6px;
		min-height: 0;
		padding: 4px 2px;
		border: 0;
		background: transparent;
		box-shadow: none;
	}

	.plano .etiqueta {
		font-size: 13.5px;
		font-weight: 400;
		letter-spacing: 0;
		text-transform: none;
	}

	.plano .valor {
		font-size: 30px;
		font-weight: 600;
	}

	.plano .contexto {
		font-size: 12.5px;
		font-weight: 500;
	}

	.plano.aviso .contexto {
		color: var(--ambar-texto);
	}

	.plano.peligro .contexto {
		color: var(--peligro-texto);
	}

	.plano.exito .contexto {
		color: var(--exito-texto);
	}

	.plano.info .contexto {
		color: var(--primario-texto);
	}
</style>
