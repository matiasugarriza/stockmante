<script lang="ts">
	import type { Snippet } from 'svelte';
	import { formatearNumero } from '#lib/utils/formato';
	import { clasificarStock, etiquetaStock, varianteStock } from '#lib/utils/inventario';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';

	interface Props {
		nombre: string;
		sku: string;
		stockActual: number;
		bajoMinimo: boolean;
		contexto?: string;
		acciones?: Snippet;
		onabrir?: () => void;
	}

	let { nombre, sku, stockActual, bajoMinimo, contexto, acciones, onabrir }: Props = $props();

	const estado = $derived(clasificarStock(stockActual, bajoMinimo));
</script>

<article class="inventario">
	<header>
		<div class="texto">
			{#if onabrir}
				<button type="button" class="abrir" onclick={onabrir}>{nombre}</button>
			{:else}
				<p class="nombre">{nombre}</p>
			{/if}
			<p class="sku">{sku}</p>
		</div>
		<div class="stock">
			<span class="valor">{formatearNumero(stockActual)}</span>
			<EtiquetaEstado variante={varianteStock(estado)}>{etiquetaStock(estado)}</EtiquetaEstado>
		</div>
	</header>

	{#if contexto}<p class="contexto">{contexto}</p>{/if}
	{#if acciones}<div class="acciones">{@render acciones()}</div>{/if}
</article>

<style>
	.inventario {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px 14px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta-movil);
		background: var(--superficie);
	}

	header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
	}

	.texto {
		min-width: 0;
		flex: 1;
	}

	.nombre,
	.abrir {
		display: -webkit-box;
		overflow: hidden;
		margin: 0;
		padding: 0;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		border: 0;
		background: transparent;
		color: var(--texto);
		font-family: var(--fuente-sans);
		font-size: 14px;
		font-weight: 600;
		line-height: 1.3;
		text-align: left;
	}

	.abrir {
		min-height: var(--toque-min);
		cursor: pointer;
	}

	.abrir:hover {
		color: var(--primario-texto);
	}

	.sku {
		overflow: hidden;
		margin: 2px 0 0;
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
		font-size: 12px;
		letter-spacing: 0.02em;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.stock {
		display: flex;
		flex: none;
		flex-direction: column;
		align-items: flex-end;
		gap: 4px;
	}

	.valor {
		color: var(--texto);
		font-family: var(--fuente-mono);
		font-size: 22px;
		font-variant-numeric: tabular-nums;
		font-weight: 600;
		line-height: 1;
	}

	.contexto {
		margin: 0;
		color: var(--texto-tenue);
		font-size: 12px;
	}

	.acciones {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
	}

	@media (max-width: 360px) {
		.nombre,
		.abrir {
			font-size: 13px;
		}

		.valor {
			font-size: 19px;
		}
	}
</style>
