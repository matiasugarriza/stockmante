<script lang="ts">
	import type { Snippet } from 'svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';

	type Variante = 'exito' | 'aviso' | 'peligro' | 'info' | 'neutro';

	interface Props {
		titulo: string;
		subtitulo?: string;
		mono?: boolean;
		badge?: { texto: string; variante: Variante };
		campos?: Array<{ etiqueta: string; valor: string }>;
		acciones?: Snippet;
		onabrir?: () => void;
	}

	let { titulo, subtitulo, mono = false, badge, campos = [], acciones, onabrir }: Props = $props();
</script>

<article class="tarjeta">
	<header>
		<div class="texto">
			{#if onabrir}
				<button type="button" class="abrir" onclick={onabrir}>{titulo}</button>
			{:else}
				<p class="titulo">{titulo}</p>
			{/if}
			{#if subtitulo}<p class="subtitulo" class:mono>{subtitulo}</p>{/if}
		</div>
		{#if badge}<EtiquetaEstado variante={badge.variante}>{badge.texto}</EtiquetaEstado>{/if}
	</header>

	{#if campos.length > 0}
		<dl>
			{#each campos as campo (campo.etiqueta + campo.valor)}
				<div>
					<dt>{campo.etiqueta}</dt>
					<dd>{campo.valor}</dd>
				</div>
			{/each}
		</dl>
	{/if}

	{#if acciones}<div class="acciones">{@render acciones()}</div>{/if}
</article>

<style>
	.tarjeta {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px 14px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta-movil);
		background: var(--superficie);
	}

	header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 10px;
	}

	.texto {
		min-width: 0;
	}

	.titulo,
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
		text-decoration: none;
	}

	.abrir {
		min-height: var(--toque-min);
		cursor: pointer;
	}

	.abrir:hover {
		color: var(--primario-texto);
	}

	.subtitulo {
		overflow: hidden;
		margin: 2px 0 0;
		color: var(--texto-tenue);
		font-size: 12px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.subtitulo.mono {
		font-family: var(--fuente-mono);
		letter-spacing: 0.02em;
	}

	dl {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0;
	}

	dl div {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		font-size: 12.5px;
	}

	dt {
		color: var(--texto-tenue);
	}

	dd {
		margin: 0;
		color: var(--texto-suave);
		text-align: right;
	}

	.acciones {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 2px;
	}

	@media (max-width: 360px) {
		.titulo,
		.abrir {
			font-size: 13px;
		}

		.subtitulo {
			font-size: 11px;
		}
	}
</style>
