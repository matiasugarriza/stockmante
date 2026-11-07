<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import BarraLateral from './BarraLateral.svelte';
	import BarraSuperior from './BarraSuperior.svelte';
	import MarcoMovil from '../movil/MarcoMovil.svelte';

	interface Props {
		titulo: string;
		alertasStock?: number;
		children: Snippet;
		acciones?: Snippet;
	}

	let { titulo, alertasStock = 0, children, acciones }: Props = $props();

	const anio = new Date().getFullYear();

	let colapsada = $state(false);
	let movilAbierta = $state(false);
	let esMovil = $state(false);
	let inicializado = false;
	let eraMovil = false;

	function evaluarAncho() {
		const ancho = window.innerWidth;
		const movil = ancho < 1024;
		esMovil = movil;
		if (!inicializado) {
			colapsada = ancho < 1280;
			inicializado = true;
		} else if (movil && !eraMovil) {
			movilAbierta = false;
		} else if (!movil && eraMovil) {
			colapsada = ancho < 1280;
		}
		eraMovil = movil;
	}

	onMount(() => {
		evaluarAncho();
		window.addEventListener('resize', evaluarAncho);
		return () => window.removeEventListener('resize', evaluarAncho);
	});

	function alternarMenu() {
		if (esMovil) {
			movilAbierta = !movilAbierta;
		} else {
			colapsada = !colapsada;
		}
	}
</script>

{#if $dispositivo.esMovil}
	<MarcoMovil {titulo} {alertasStock}>
		{@render children()}
	</MarcoMovil>
{:else}
	<a class="saltar" href="#contenido">Saltar al contenido</a>

	<div class="marco">
		<BarraLateral
			colapsada={colapsada && !esMovil}
			{movilAbierta}
			{alertasStock}
			oncerrar={() => (movilAbierta = false)}
		/>

		{#if movilAbierta}
			<button
				type="button"
				class="fondo"
				aria-label="Cerrar navegación"
				onclick={() => (movilAbierta = false)}
			></button>
		{/if}

		<div class="columna">
			<BarraSuperior {titulo} onabrirMenu={alternarMenu}>
				{#if acciones}{@render acciones()}{/if}
			</BarraSuperior>
			<main id="contenido" class="contenido" tabindex="-1">
				{@render children()}
			</main>
			<footer class="pie">
				<span>Stockmante · Sistema de gestión de inventario</span>
				<span>© {anio} Todos los derechos reservados.</span>
			</footer>
		</div>
	</div>
{/if}

<style>
	.marco {
		display: flex;
		min-height: 100dvh;
		background: var(--fondo);
	}

	.columna {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
	}

	.contenido {
		flex: 1;
		width: 100%;
		max-width: var(--contenido-max);
		margin: 0 auto;
		padding: 24px;
	}

	.pie {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 4px 16px;
		width: 100%;
		max-width: var(--contenido-max);
		margin: 0 auto;
		padding: 18px 24px 22px;
		border-top: 1px solid var(--borde);
		color: var(--texto-tenue);
		font-size: 12.5px;
	}

	.fondo {
		position: fixed;
		inset: 0;
		z-index: 25;
		border: 0;
		background: rgba(6, 14, 17, 0.5);
	}

	.saltar {
		position: absolute;
		left: -9999px;
		top: 12px;
		z-index: 100;
		padding: 8px 14px;
		border-radius: var(--radio-control);
		background: var(--primario);
		color: var(--primario-contraste);
		font-size: 13px;
		font-weight: 600;
		text-decoration: none;
	}

	.saltar:focus {
		left: 12px;
	}

	@media (max-width: 640px) {
		.contenido {
			padding: 16px 12px 28px;
		}

		.pie {
			padding: 14px 12px 18px;
		}
	}
</style>
