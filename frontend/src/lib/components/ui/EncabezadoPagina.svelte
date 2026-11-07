<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		titulo: string;
		descripcion?: string;
		children?: Snippet;
	}

	let { titulo, descripcion, children }: Props = $props();
</script>

<header class="encabezado">
	<div class="texto">
		<h1>{titulo}</h1>
		{#if descripcion}<p>{descripcion}</p>{/if}
	</div>
	{#if children}<div class="acciones">{@render children()}</div>{/if}
</header>

<style>
	.encabezado {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 16px;
		flex-wrap: wrap;
		margin-bottom: 20px;
	}

	.texto {
		min-width: 0;
	}

	h1 {
		margin: 0;
		color: var(--texto);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.02em;
		line-height: 1.2;
	}

	p {
		margin: 5px 0 0;
		color: var(--texto-suave);
		font-size: 13px;
		line-height: 1.45;
		max-width: 78ch;
	}

	.acciones {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	@media (max-width: 640px) {
		h1 {
			font-size: 19px;
		}

		.acciones {
			width: 100%;
		}
	}
</style>
