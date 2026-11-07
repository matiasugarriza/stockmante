<script lang="ts">
	import { page } from '$app/state';
	import { tick } from 'svelte';
	import { sesion } from '#lib/stores/sesion';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import {
		gruposVisibles,
		type FamiliaRol
	} from '#lib/navegacion/navegacion';
	import Icono from '../ui/Icono.svelte';
	import ItemNavegacion from './ItemNavegacion.svelte';
	import LogoMarca from './LogoMarca.svelte';

	interface Props {
		colapsada?: boolean;
		movilAbierta?: boolean;
		alertasStock?: number;
		oncerrar?: () => void;
	}

	let { colapsada = false, movilAbierta = false, alertasStock = 0, oncerrar }: Props = $props();

	const familiaRol = $derived<FamiliaRol>(
		$sesion.estado === 'activa' ? $sesion.usuario.familia_rol : 'OPERACION'
	);
	const grupos = $derived(gruposVisibles(page.url.pathname, { familiaRol, alertasStock }));
	const ocultoEnMovil = $derived($dispositivo.esMovil && !movilAbierta);

	let panel: HTMLElement | undefined = $state();
	let disparador: HTMLElement | null = null;

	// Foco atrapado, Escape y bloqueo de scroll del panel móvil (FR-110).
	$effect(() => {
		if (!movilAbierta) return;
		disparador = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		const overflowPrevio = document.body.style.overflow;
		document.body.style.overflow = 'hidden';
		void tick().then(() => {
			panel?.querySelector<HTMLElement>('.cerrar')?.focus();
		});
		return () => {
			document.body.style.overflow = overflowPrevio;
			disparador?.focus();
		};
	});

	function alTeclear(event: KeyboardEvent) {
		if (event.key === 'Escape' && movilAbierta) {
			event.preventDefault();
			oncerrar?.();
			return;
		}
		if (event.key !== 'Tab' || !movilAbierta || !panel) return;
		const enfocables = Array.from(
			panel.querySelectorAll<HTMLElement>('a[href], button:not([disabled])')
		).filter((elemento) => elemento.offsetParent !== null);
		if (enfocables.length === 0) {
			event.preventDefault();
			return;
		}
		const primero = enfocables[0];
		const ultimo = enfocables[enfocables.length - 1];
		if (event.shiftKey && document.activeElement === primero) {
			event.preventDefault();
			ultimo?.focus();
		} else if (!event.shiftKey && document.activeElement === ultimo) {
			event.preventDefault();
			primero?.focus();
		}
	}
</script>

<aside
	class="lateral"
	class:colapsada
	class:abierta={movilAbierta}
	bind:this={panel}
	role={movilAbierta ? 'dialog' : undefined}
	aria-modal={movilAbierta ? 'true' : undefined}
	aria-label={movilAbierta ? 'Navegación completa' : undefined}
	inert={ocultoEnMovil}
	aria-hidden={ocultoEnMovil ? 'true' : undefined}
	onkeydown={alTeclear}
>
	<div class="marca">
		<a href="/" class="logo" aria-label="Stockmante, ir al inicio" onclick={oncerrar}>
			<LogoMarca alto={36} soloSimbolo={colapsada} />
		</a>
		<button type="button" class="cerrar" aria-label="Cerrar navegación" onclick={oncerrar}>
			<Icono nombre="cerrar" tamano={18} />
		</button>
	</div>

	<nav class="navegacion" aria-label="Navegación principal">
		{#each grupos as grupo (grupo.id)}
			<p class="grupo">{grupo.etiqueta}</p>
			{#each grupo.items as item (item.ruta)}
				<ItemNavegacion {item} {colapsada} onNavegar={oncerrar} />
			{/each}
		{/each}
	</nav>

	{#if !colapsada}
		<!-- El selector de tema vive solo en la barra superior. -->
		<div class="pie">
			<span class="version">Stockmante v0.1</span>
		</div>
	{/if}
</aside>

<style>
	.lateral {
		display: flex;
		flex: none;
		flex-direction: column;
		width: var(--sidebar-ancho);
		height: 100dvh;
		position: sticky;
		top: 0;
		z-index: 30;
		border-right: 1px solid var(--borde);
		background: var(--sidebar);
	}

	.lateral.colapsada {
		width: var(--sidebar-ancho-colapsado);
	}

	.marca {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		min-height: var(--topbar-alto);
		padding: 0 20px;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
		color: var(--texto);
		text-decoration: none;
	}

	.cerrar {
		display: none;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: var(--radio-control);
		background: transparent;
		color: var(--texto-suave);
		cursor: pointer;
	}

	.cerrar:hover {
		background: var(--superficie-2);
	}

	.navegacion {
		flex: 1;
		overflow-y: auto;
		padding: 8px 14px 20px 0;
	}

	.grupo {
		margin: 18px 0 6px 24px;
		color: var(--texto);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.grupo:first-child {
		margin-top: 4px;
	}

	.pie {
		display: flex;
		justify-content: center;
		padding: 12px 14px;
		border-top: 1px solid var(--borde);
	}

	.version {
		color: var(--texto-tenue);
		text-align: center;
		font-family: var(--fuente-sans);
		font-size: 11px;
	}

	.colapsada .grupo,
	.colapsada .version,
	.colapsada .cerrar {
		display: none;
	}

	.colapsada .marca {
		justify-content: center;
		padding: 0;
	}

	.colapsada .navegacion {
		padding: 10px 8px;
	}

	@media (max-width: 1023px) {
		.lateral,
		.lateral.colapsada {
			position: fixed;
			inset: 0 auto 0 0;
			width: min(var(--sidebar-ancho), 86vw);
			padding-top: var(--zona-segura-superior);
			padding-bottom: var(--zona-segura-inferior);
			transform: translateX(-102%);
			transition: transform 180ms ease;
			box-shadow: var(--sombra-elevada);
		}

		.lateral.abierta {
			transform: none;
		}

		.lateral.colapsada .marca {
			justify-content: space-between;
			padding: 0 16px;
		}

		.lateral.colapsada .navegacion {
			padding: 8px 14px 20px 0;
		}

		.lateral.colapsada .grupo,
		.lateral.colapsada .version {
			display: initial;
		}

		.lateral.colapsada .cerrar {
			display: inline-flex;
		}

		.cerrar {
			display: inline-flex;
		}

	}

	@media (prefers-reduced-motion: reduce) {
		.lateral {
			transition: none;
		}
	}
</style>
