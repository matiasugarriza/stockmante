<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icono from './Icono.svelte';

	type Variante = 'info' | 'exito' | 'error' | 'advertencia';

	interface Props {
		variante?: Variante;
		titulo?: string;
		accion?: Snippet;
		children: Snippet;
	}

	let { variante = 'info', titulo, accion, children }: Props = $props();

	const icono = $derived(variante === 'exito' ? 'check' : 'alerta');
</script>

<div class="aviso {variante}" role={variante === 'error' ? 'alert' : 'status'}>
	<span class="marca" aria-hidden="true">
		<Icono nombre={icono} tamano={17} />
	</span>
	<div class="contenido">
		{#if titulo}<p class="titulo">{titulo}</p>{/if}
		<div class="texto">{@render children()}</div>
	</div>
	{#if accion}<div class="accion">{@render accion()}</div>{/if}
</div>

<style>
	.aviso {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		border: 1px solid transparent;
		border-radius: var(--radio-control);
		padding: 10px 12px;
		font-size: 13px;
		line-height: 1.45;
	}

	.marca {
		flex: none;
		margin-top: 1px;
	}

	.contenido {
		min-width: 0;
		flex: 1;
	}

	.titulo {
		margin: 0 0 2px;
		font-weight: 600;
	}

	.texto :global(p) {
		margin: 0;
	}

	.accion {
		flex: none;
	}

	.info {
		border-color: color-mix(in oklab, var(--info) 30%, transparent);
		background: var(--info-suave);
		color: var(--primario-texto);
	}

	.exito {
		border-color: color-mix(in oklab, var(--exito) 30%, transparent);
		background: var(--exito-suave);
		color: var(--exito-texto);
	}

	.error {
		border-color: color-mix(in oklab, var(--peligro) 30%, transparent);
		background: var(--peligro-suave);
		color: var(--peligro-texto);
	}

	.advertencia {
		border-color: color-mix(in oklab, var(--ambar) 35%, transparent);
		background: var(--ambar-suave);
		color: var(--ambar-texto);
	}
</style>
