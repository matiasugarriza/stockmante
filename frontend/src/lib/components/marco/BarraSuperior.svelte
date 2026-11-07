<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icono from '../ui/Icono.svelte';
	import BotonTema from '../ui/BotonTema.svelte';
	import MenuUsuario from './MenuUsuario.svelte';
	import { page } from '$app/state';
	import { sesion } from '#lib/stores/sesion';

	interface Props {
		titulo: string;
		descripcion?: string;
		onabrirMenu: () => void;
		children?: Snippet;
	}

	let { titulo, descripcion, onabrirMenu, children }: Props = $props();

	// FR-099: en Inicio la barra saluda al usuario, como el encabezado de las muestras.
	const enInicio = $derived(page.url.pathname === '/');
	const nombre = $derived(
		$sesion.estado === 'activa' ? ($sesion.usuario.nombre.split(/\s+/)[0] ?? '') : ''
	);

	function saludoDelMomento(hora: number): string {
		if (hora < 12) return 'Buenos días';
		if (hora < 20) return 'Buenas tardes';
		return 'Buenas noches';
	}

	const saludo = saludoDelMomento(new Date().getHours());
</script>

<header class="superior zona-oscura">
	<div class="izquierda">
		<button type="button" class="menu" aria-label="Abrir menú de navegación" onclick={onabrirMenu}>
			<Icono nombre="menu" tamano={20} />
		</button>
		<div class="contexto">
			{#if enInicio && nombre}
				<h1 class="saludo">{saludo}, <strong>{nombre}</strong></h1>
				<p>Este es el resumen de tu inventario de hoy</p>
			{:else}
				<h1>{titulo}</h1>
				{#if descripcion}<p>{descripcion}</p>{/if}
			{/if}
		</div>
	</div>
	<div class="derecha">
		{#if children}<div class="acciones">{@render children()}</div>{/if}
		<div class="tema"><BotonTema compacto /></div>
		<MenuUsuario />
	</div>
</header>

<style>
	.superior {
		position: sticky;
		top: 0;
		z-index: 20;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		min-height: var(--topbar-alto);
		padding: 0 24px;
		border-bottom: 1px solid var(--borde);
		background: var(--topbar);
		color: var(--texto);
	}

	.izquierda,
	.derecha {
		display: flex;
		align-items: center;
		gap: 12px;
		min-width: 0;
	}

	.menu {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 34px;
		height: 34px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto-suave);
		cursor: pointer;
	}

	.menu:hover {
		background: var(--superficie-2);
		color: var(--texto);
	}

	.izquierda {
		flex: 1;
	}

	.derecha {
		flex: none;
	}

	.contexto {
		min-width: 0;
	}

	h1 {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	h1 {
		margin: 0;
		color: var(--texto);
		font-size: 19px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.2;
	}

	h1.saludo {
		font-size: 22px;
		font-weight: 300;
	}

	h1.saludo strong {
		font-weight: 600;
	}

	p {
		margin: 3px 0 0;
		color: var(--texto-suave);
		font-size: 13px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.acciones {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	@media (max-width: 640px) {
		.superior {
			padding: 0 12px;
		}

		p {
			display: none;
		}

		h1,
		h1.saludo {
			font-size: 16px;
		}

		.acciones {
			display: none;
		}
	}
</style>
