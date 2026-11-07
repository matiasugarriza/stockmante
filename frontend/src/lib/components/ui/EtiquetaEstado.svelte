<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variante = 'exito' | 'aviso' | 'peligro' | 'info' | 'neutro';

	interface Props {
		variante?: Variante;
		children: Snippet;
	}

	let { variante = 'neutro', children }: Props = $props();
</script>

<span class="etiqueta {variante}">{@render children()}</span>

<style>
	.etiqueta {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		border-radius: var(--radio-pildora);
		padding: 2px 8px;
		font-size: 11px;
		font-weight: 600;
		line-height: 1.5;
		letter-spacing: 0.03em;
		text-transform: uppercase;
		white-space: nowrap;
	}

	.exito {
		background: var(--exito-suave);
		color: var(--exito-texto);
	}

	.aviso {
		background: var(--ambar-suave);
		color: var(--ambar-texto);
	}

	.peligro {
		background: var(--peligro-suave);
		color: var(--peligro-texto);
	}

	.info {
		background: var(--info-suave);
		color: var(--primario-texto);
	}

	.neutro {
		background: var(--superficie-3);
		color: var(--texto-suave);
	}
</style>
