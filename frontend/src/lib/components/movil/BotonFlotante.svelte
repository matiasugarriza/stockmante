<script lang="ts">
	import Icono from '../ui/Icono.svelte';
	import type { NombreIcono } from '../ui/iconos';

	interface Props {
		etiqueta: string;
		icono?: NombreIcono;
		onclick: () => void;
	}

	let { etiqueta, icono = 'mas', onclick }: Props = $props();
</script>

<button type="button" class="fab" aria-label={etiqueta} title={etiqueta} {onclick}>
	<Icono nombre={icono} tamano={24} />
</button>

<style>
	.fab {
		position: fixed;
		right: 16px;
		bottom: calc(var(--barra-inferior-alto) + var(--zona-segura-inferior) + 16px);
		z-index: 26;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--fab-tamano);
		height: var(--fab-tamano);
		border: 1px solid var(--primario);
		border-radius: 50%;
		background: var(--primario);
		color: var(--primario-contraste);
		box-shadow: var(--brillo-cian);
		cursor: pointer;
	}
</style>
