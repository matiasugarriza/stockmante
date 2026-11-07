<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		titulo?: string;
		descripcion?: string;
		accion?: Snippet;
		children: Snippet;
		pie?: Snippet;
		densa?: boolean;
	}

	let { titulo, descripcion, accion, children, pie, densa = false }: Props = $props();
</script>

<section class="tarjeta" class:densa>
	{#if titulo}
		<header class="cabecera">
			<div>
				<h2>{titulo}</h2>
				{#if descripcion}<p>{descripcion}</p>{/if}
			</div>
			{#if accion}<div class="accion">{@render accion()}</div>{/if}
		</header>
	{/if}
	<div class="cuerpo">{@render children()}</div>
	{#if pie}<footer class="pie">{@render pie()}</footer>{/if}
</section>

<style>
	.tarjeta {
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta);
		background: var(--superficie);
		box-shadow: var(--sombra-tarjeta);
		overflow: hidden;
	}

	.cabecera {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		padding: 18px 20px 0;
	}

	h2 {
		margin: 0;
		color: var(--texto);
		font-size: 15px;
		font-weight: 600;
		line-height: 1.3;
	}

	p {
		margin: 4px 0 0;
		color: var(--texto-suave);
		font-size: 12.5px;
		line-height: 1.4;
	}

	.cuerpo {
		padding: 20px;
	}

	.densa .cuerpo {
		padding: 14px 16px;
	}

	.pie {
		border-top: 1px solid var(--borde);
		padding: 12px 20px;
		background: var(--superficie-2);
	}

	.accion {
		flex: none;
	}
</style>
