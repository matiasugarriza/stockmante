<script lang="ts">
	import LogoMarca from './LogoMarca.svelte';
	import type { Snippet } from 'svelte';
	import BotonTema from '../ui/BotonTema.svelte';

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();
</script>

<div class="lienzo">
	<header class="barra">
		<a class="marca" href="/" aria-label="Stockmante, ir al inicio">
			<LogoMarca alto={32} />
		</a>
		<BotonTema />
	</header>

	<main class="panel">{@render children()}</main>

	<p class="pie">
		Desarrollado por
		<a href="https://github.com/matiasugarriza" target="_blank" rel="noopener noreferrer">
			Matias Ugarriza
		</a>
	</p>
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
		font-size: 12.5px;
		text-align: center;
	}

	.pie a {
		color: var(--primario-texto);
		font-weight: 500;
		text-decoration: underline;
		text-underline-offset: 2px;
	}

	.pie a:hover {
		color: var(--texto);
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

	@media (max-width: 1023px) {
		.barra {
			padding: calc(var(--zona-segura-superior) + 12px) var(--espacio-movil) 4px;
		}

		.panel {
			padding: 12px var(--espacio-movil) calc(var(--zona-segura-inferior) + 24px);
		}

		.pie {
			padding: 0 var(--espacio-movil) calc(var(--zona-segura-inferior) + 12px);
		}
	}
</style>
