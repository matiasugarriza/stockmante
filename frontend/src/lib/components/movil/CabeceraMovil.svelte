<script lang="ts">
	import Icono from '../ui/Icono.svelte';
	import MenuUsuario from '../marco/MenuUsuario.svelte';

	interface Props {
		titulo: string;
		onabrirBuscador: () => void;
	}

	let { titulo, onabrirBuscador }: Props = $props();
</script>

<header class="cabecera">
	<h1 title={titulo}>{titulo}</h1>
	<div class="acciones">
		<button
			type="button"
			class="icono-boton"
			aria-label="Buscar producto"
			onclick={onabrirBuscador}
		>
			<Icono nombre="buscar" tamano={20} />
		</button>
		<MenuUsuario />
	</div>
</header>

<style>
	.cabecera {
		position: sticky;
		top: 0;
		z-index: 20;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		min-height: calc(var(--cabecera-alto-movil) + var(--zona-segura-superior));
		padding: var(--zona-segura-superior) var(--espacio-movil) 0;
		border-bottom: 1px solid var(--borde);
		background: var(--fondo-elevado);
	}

	h1 {
		overflow: hidden;
		margin: 0;
		color: var(--texto);
		font-size: 17px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.2;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.acciones {
		display: flex;
		flex: none;
		align-items: center;
		gap: 6px;
	}

	.icono-boton {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--toque-min);
		height: var(--toque-min);
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto-suave);
		cursor: pointer;
	}

	.icono-boton:hover {
		border-color: var(--primario);
		color: var(--primario-texto);
	}
</style>
