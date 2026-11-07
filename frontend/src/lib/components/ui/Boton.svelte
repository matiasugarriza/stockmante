<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icono from './Icono.svelte';
	import type { NombreIcono } from './iconos';

	interface Props {
		variante?: 'primario' | 'secundario' | 'fantasma' | 'peligro';
		tamano?: 'sm' | 'md';
		tipo?: 'button' | 'submit' | 'reset';
		href?: string;
		cargando?: boolean;
		deshabilitado?: boolean;
		icono?: NombreIcono;
		anchoCompleto?: boolean;
		onclick?: (event: MouseEvent) => void;
		children: Snippet;
	}

	let {
		variante = 'primario',
		tamano = 'md',
		tipo = 'button',
		href,
		cargando = false,
		deshabilitado = false,
		icono,
		anchoCompleto = false,
		onclick,
		children
	}: Props = $props();

	const clases = $derived(
		['boton', variante, tamano, anchoCompleto ? 'ancho-completo' : ''].filter(Boolean).join(' ')
	);

	function alHacerClic(event: MouseEvent) {
		if (deshabilitado || cargando) {
			event.preventDefault();
			return;
		}
		onclick?.(event);
	}
</script>

{#if href}
	<a
		{href}
		class={clases}
		aria-disabled={deshabilitado || cargando ? 'true' : undefined}
		tabindex={deshabilitado || cargando ? -1 : undefined}
		onclick={alHacerClic}
	>
		{#if cargando}<span class="girador" aria-hidden="true"></span>{/if}
		{#if icono && !cargando}<Icono nombre={icono} tamano={tamano === 'sm' ? 15 : 17} />{/if}
		{@render children()}
	</a>
{:else}
	<button
		type={tipo}
		class={clases}
		disabled={deshabilitado || cargando}
		aria-busy={cargando ? 'true' : undefined}
		{onclick}
	>
		{#if cargando}<span class="girador" aria-hidden="true"></span>{/if}
		{#if icono && !cargando}<Icono nombre={icono} tamano={tamano === 'sm' ? 15 : 17} />{/if}
		{@render children()}
	</button>
{/if}

<style>
	.boton {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-height: 40px;
		padding: 0 16px;
		border: 1px solid transparent;
		border-radius: var(--radio-control);
		font-family: var(--fuente-sans);
		font-size: 14px;
		font-weight: 600;
		line-height: 1.2;
		white-space: nowrap;
		text-decoration: none;
		cursor: pointer;
		transition:
			background-color 120ms ease,
			border-color 120ms ease,
			color 120ms ease;
	}

	.boton.sm {
		min-height: 32px;
		padding: 0 11px;
		font-size: 13px;
	}

	.boton:disabled,
	.boton[aria-disabled='true'] {
		cursor: not-allowed;
		opacity: 0.6;
	}

	.primario {
		background: var(--primario);
		color: var(--primario-contraste);
	}

	.primario:hover:not(:disabled) {
		background: var(--primario-hover);
	}

	.secundario {
		border-color: var(--borde-fuerte);
		background: var(--superficie);
		color: var(--texto);
	}

	.secundario:hover:not(:disabled) {
		background: var(--superficie-2);
		border-color: var(--primario);
	}

	.fantasma {
		background: transparent;
		color: var(--texto-suave);
	}

	.fantasma:hover:not(:disabled) {
		background: var(--superficie-2);
		color: var(--texto);
	}

	.peligro {
		background: var(--peligro);
		color: var(--texto-inverso);
	}

	.peligro:hover:not(:disabled) {
		filter: brightness(0.94);
	}

	.ancho-completo {
		width: 100%;
	}

	.girador {
		width: 14px;
		height: 14px;
		border: 2px solid currentColor;
		border-right-color: transparent;
		border-radius: 50%;
		animation: girar 700ms linear infinite;
	}

	@keyframes girar {
		to {
			transform: rotate(360deg);
		}
	}

	/* Área de toque mínima y estado destructivo legible (spec 019, FR-106 y FR-103). */
	@media (max-width: 1023px) {
		.boton,
		.boton.sm {
			min-height: var(--toque-min);
			padding: 0 14px;
		}

		.peligro {
			border-color: var(--peligro);
			background: var(--peligro-suave);
			color: var(--peligro-texto);
		}
	}
</style>
