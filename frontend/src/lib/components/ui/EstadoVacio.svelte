<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icono from './Icono.svelte';
	import type { NombreIcono } from './iconos';

	interface Props {
		icono?: NombreIcono;
		titulo: string;
		descripcion?: string;
		children?: Snippet;
	}

	let { icono = 'buscar', titulo, descripcion, children }: Props = $props();
</script>

<div class="vacio">
	<span class="marca" aria-hidden="true">
		<Icono nombre={icono} tamano={26} />
	</span>
	<h3>{titulo}</h3>
	{#if descripcion}<p>{descripcion}</p>{/if}
	{#if children}<div class="accion">{@render children()}</div>{/if}
</div>

<style>
	.vacio {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 32px 20px;
		text-align: center;
	}

	.marca {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 46px;
		height: 46px;
		margin-bottom: 4px;
		border-radius: 50%;
		background: var(--superficie-2);
		color: var(--texto-tenue);
	}

	h3 {
		margin: 0;
		color: var(--texto);
		font-size: 14px;
		font-weight: 600;
	}

	p {
		margin: 0;
		max-width: 46ch;
		color: var(--texto-suave);
		font-size: 13px;
		line-height: 1.45;
	}

	.accion {
		margin-top: 10px;
	}
</style>
