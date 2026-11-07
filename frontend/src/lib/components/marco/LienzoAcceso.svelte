<script lang="ts">
	import type { Snippet } from 'svelte';
	import BotonTema from '../ui/BotonTema.svelte';
	import Icono from '../ui/Icono.svelte';

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();
</script>

<div class="lienzo">
	<header class="barra">
		<a class="marca" href="/" aria-label="Stockmante, ir al inicio">
			<span class="simbolo" aria-hidden="true"><Icono nombre="stock" tamano={18} /></span>
			<span class="nombre">Stock<span class="acento">mante</span></span>
		</a>
		<BotonTema />
	</header>

	<main class="panel">{@render children()}</main>

	<p class="pie">Sistema de gestión de inventario</p>
</div>

<style>
	.lienzo {
		display: flex;
		flex-direction: column;
		min-height: 100dvh;
		background: var(--fondo);
	}

	.barra {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 18px 24px;
	}

	.marca {
		display: inline-flex;
		align-items: center;
		gap: 9px;
		color: var(--texto);
		font-size: 15px;
		font-weight: 700;
		letter-spacing: -0.02em;
		text-decoration: none;
	}

	.simbolo {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: 8px;
		background: var(--primario);
		color: var(--primario-contraste);
	}

	.nombre {
		font-size: 18px;
		font-weight: 600;
		letter-spacing: -0.02em;
	}

	.acento {
		color: var(--primario-texto);
	}

	.panel {
		display: flex;
		flex: 1;
		align-items: center;
		justify-content: center;
		padding: 16px 24px 32px;
	}

	.pie {
		margin: 0;
		padding: 0 24px 20px;
		color: var(--texto-tenue);
		font-family: var(--fuente-sans);
		font-size: 11px;
		letter-spacing: 0.08em;
		text-align: center;
		text-transform: uppercase;
	}

	@media (max-width: 480px) {
		.barra {
			padding: 14px 16px;
		}

		.panel {
			align-items: flex-start;
			padding: 8px 16px 24px;
		}
	}
</style>
