<script lang="ts">
	import type { Snippet } from 'svelte';
	import BarraLateral from '../marco/BarraLateral.svelte';
	import BarraInferior from './BarraInferior.svelte';
	import BuscadorMovil from './BuscadorMovil.svelte';
	import CabeceraMovil from './CabeceraMovil.svelte';

	interface Props {
		titulo: string;
		alertasStock?: number;
		children: Snippet;
	}

	let { titulo, alertasStock = 0, children }: Props = $props();

	let panelAbierto = $state(false);
	let buscadorAbierto = $state(false);

	function cerrarPanel() {
		panelAbierto = false;
	}
</script>

<a class="saltar" href="#contenido">Saltar al contenido</a>

<div class="marco">
	<BarraLateral movilAbierta={panelAbierto} {alertasStock} oncerrar={cerrarPanel} />

	{#if panelAbierto}
		<button
			type="button"
			class="fondo"
			aria-label="Cerrar navegación"
			onclick={cerrarPanel}
		></button>
	{/if}

	<div class="columna">
		<CabeceraMovil {titulo} onabrirBuscador={() => (buscadorAbierto = true)} />
		<main id="contenido" class="contenido" tabindex="-1">
			{@render children()}
		</main>
	</div>

	<BarraInferior {alertasStock} onabrirPanel={() => (panelAbierto = true)} />
	<BuscadorMovil abierto={buscadorAbierto} oncerrar={() => (buscadorAbierto = false)} />
</div>

<style>
	.marco {
		min-height: 100dvh;
		background: var(--fondo);
	}

	.columna {
		display: flex;
		flex-direction: column;
		min-height: 100dvh;
	}

	.contenido {
		flex: 1;
		width: 100%;
		max-width: var(--contenido-max-movil);
		margin: 0 auto;
		padding: 12px var(--espacio-movil)
			calc(var(--barra-inferior-alto) + var(--zona-segura-inferior) + 16px);
		scroll-padding-top: calc(var(--cabecera-alto-movil) + 12px);
		scroll-padding-bottom: var(--scroll-padding-inferior);
	}

	.fondo {
		position: fixed;
		inset: 0;
		z-index: 29;
		border: 0;
		background: rgba(0, 0, 0, 0.62);
	}

	.saltar {
		position: absolute;
		top: 12px;
		left: -9999px;
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
</style>
