<script lang="ts">
	import { page } from '$app/state';
	import { sesion } from '#lib/stores/sesion';
	import {
		gruposVisibles,
		type FamiliaRol
	} from '#lib/navegacion/navegacion';
	import Icono from '../ui/Icono.svelte';
	import BotonTema from '../ui/BotonTema.svelte';
	import ItemNavegacion from './ItemNavegacion.svelte';

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
</script>

<aside class="lateral" class:colapsada class:abierta={movilAbierta}>
	<div class="marca">
		<a href="/" class="logo" aria-label="Stockmante, ir al inicio" onclick={oncerrar}>
			<span class="simbolo" aria-hidden="true"><Icono nombre="stock" tamano={19} /></span>
			<span class="nombre">Stock<span class="acento">mante</span></span>
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
		<!-- Colapsada, el selector de tema queda solo en la barra superior (no entra en 76 px). -->
		<div class="pie">
			<BotonTema />
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

	.simbolo {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 8px;
		background: var(--primario);
		color: var(--primario-contraste);
	}

	.nombre {
		font-size: 21px;
		font-weight: 600;
		letter-spacing: -0.02em;
		white-space: nowrap;
	}

	.acento {
		color: var(--primario-texto);
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
		flex-direction: column;
		align-items: stretch;
		gap: 8px;
		padding: 12px 14px;
		border-top: 1px solid var(--borde);
	}

	.version {
		color: var(--texto-tenue);
		text-align: center;
		font-family: var(--fuente-sans);
		font-size: 11px;
	}

	.colapsada .nombre,
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
			width: var(--sidebar-ancho);
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

		.lateral.colapsada .nombre,
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
