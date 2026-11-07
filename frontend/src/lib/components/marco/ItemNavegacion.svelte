<script lang="ts">
	import Icono from '../ui/Icono.svelte';
	import type { ItemNavegacionVisible } from '#lib/navegacion/navegacion';

	interface Props {
		item: ItemNavegacionVisible;
		colapsada?: boolean;
		onNavegar?: () => void;
	}

	let { item, colapsada = false, onNavegar }: Props = $props();
</script>

<a
	href={item.ruta}
	class="item"
	class:activo={item.activo}
	class:colapsada
	aria-current={item.activo ? 'page' : undefined}
	aria-label={colapsada ? item.etiqueta : undefined}
	title={colapsada ? item.etiqueta : undefined}
	onclick={onNavegar}
>
	<span class="icono" aria-hidden="true">
		<Icono nombre={item.icono} tamano={18} />
	</span>
	<span class="etiqueta">{item.etiqueta}</span>
	{#if item.contadorVisible !== null}
		<span class="contador" aria-label={`${item.contadorVisible} alertas de stock`}>
			{item.contadorVisible}
		</span>
	{/if}
</a>

<style>
	.item {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 36px;
		padding: 7px 14px 7px 24px;
		border-radius: 0 var(--radio-pildora) var(--radio-pildora) 0;
		color: var(--texto-suave);
		font-size: 14px;
		font-weight: 400;
		text-decoration: none;
		transition:
			background-color 120ms ease,
			color 120ms ease;
	}

	.item:hover {
		background: var(--superficie-2);
		color: var(--texto);
	}

	.item.activo {
		background: var(--primario-suave);
		color: var(--primario-texto);
		font-weight: 600;
	}

	.icono {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 20px;
	}

	.etiqueta {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.contador {
		flex: none;
		min-width: 18px;
		padding: 0 5px;
		border-radius: var(--radio-pildora);
		background: var(--peligro);
		color: var(--texto-inverso);
		font-size: 10.5px;
		font-weight: 700;
		line-height: 18px;
		text-align: center;
	}

	.colapsada {
		justify-content: center;
		padding: 9px 0;
		border-radius: 10px;
	}

	.colapsada .etiqueta,
	.colapsada .contador {
		display: none;
	}
</style>
