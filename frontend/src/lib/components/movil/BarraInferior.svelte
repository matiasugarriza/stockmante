<script lang="ts">
	import { page } from '$app/state';
	import { sesion } from '#lib/stores/sesion';
	import { destinosBarraInferior, type FamiliaRol } from '#lib/navegacion/navegacion';
	import Icono from '../ui/Icono.svelte';

	interface Props {
		alertasStock?: number;
		onabrirPanel: () => void;
	}

	let { alertasStock = 0, onabrirPanel }: Props = $props();

	const familiaRol = $derived<FamiliaRol>(
		$sesion.estado === 'activa' ? $sesion.usuario.familia_rol : 'OPERACION'
	);
	const destinos = $derived(
		destinosBarraInferior(page.url.pathname, { familiaRol, alertasStock })
	);
</script>

<nav class="barra" aria-label="Navegación rápida">
	{#each destinos as destino (destino.ruta)}
		{#if destino.ruta === '#panel'}
			<button
				type="button"
				class="destino"
				class:activo={destino.activo}
				aria-current={destino.activo ? 'page' : undefined}
				onclick={onabrirPanel}
			>
				<span class="icono" aria-hidden="true"><Icono nombre={destino.icono} tamano={22} /></span>
				<span class="texto">{destino.etiqueta}</span>
			</button>
		{:else}
			<a
				href={destino.ruta}
				class="destino"
				class:activo={destino.activo}
				aria-current={destino.activo ? 'page' : undefined}
			>
				<span class="icono" aria-hidden="true"><Icono nombre={destino.icono} tamano={22} /></span>
				<span class="texto">{destino.etiqueta}</span>
				{#if destino.contadorVisible}
					<span class="contador" aria-label={`${destino.contadorVisible} alertas`}>
						{destino.contadorVisible}
					</span>
				{/if}
			</a>
		{/if}
	{/each}
</nav>

<style>
	.barra {
		position: fixed;
		right: 0;
		bottom: 0;
		left: 0;
		z-index: 28;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		padding-bottom: var(--zona-segura-inferior);
		border-top: 1px solid var(--borde);
		background: var(--fondo-elevado);
		box-shadow: 0 -12px 24px -20px rgba(0, 0, 0, 0.9);
	}

	.destino {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		min-height: var(--barra-inferior-alto);
		padding: 6px 2px;
		border: 0;
		background: transparent;
		color: var(--texto-tenue);
		font-family: var(--fuente-sans);
		font-size: 10.5px;
		font-weight: 500;
		text-decoration: none;
		cursor: pointer;
	}

	.icono {
		display: inline-flex;
	}

	.destino.activo {
		color: var(--primario-texto);
	}

	.destino.activo::before {
		content: '';
		position: absolute;
		top: 0;
		width: 24px;
		height: 2px;
		border-radius: 0 0 2px 2px;
		background: var(--primario);
		box-shadow: var(--brillo-cian);
	}

	.destino:focus-visible {
		outline: 2px solid var(--foco);
		outline-offset: -2px;
	}

	.contador {
		position: absolute;
		top: 4px;
		right: calc(50% - 22px);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 18px;
		height: 18px;
		padding: 0 5px;
		border-radius: var(--radio-pildora);
		background: var(--peligro);
		color: var(--texto-inverso);
		font-size: 10px;
		font-weight: 700;
	}

	@media (max-width: 360px) {
		.destino {
			font-size: 9.5px;
		}
	}
</style>
