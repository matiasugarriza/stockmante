<script lang="ts">
	import Boton from '#lib/components/ui/Boton.svelte';
	import EscanerCamara from './EscanerCamara.svelte';
	import type { Lectura } from '#lib/codigos/escaner';

	// 020 (FR-134, FR-160): botón etiquetado que abre la cámara bajo pedido explícito.

	interface Props {
		onLectura: (lectura: Lectura) => void;
		etiqueta?: string;
		deshabilitado?: boolean;
		tamano?: 'sm' | 'md';
	}

	let {
		onLectura,
		etiqueta = 'Escanear con cámara',
		deshabilitado = false,
		tamano = 'md'
	}: Props = $props();

	let abierto = $state(false);
</script>

<span class="boton-escanear">
	<Boton
		variante="secundario"
		{tamano}
		icono="camara"
		{deshabilitado}
		onclick={() => (abierto = true)}
	>
		{etiqueta}
	</Boton>
</span>

<EscanerCamara {abierto} oncerrar={() => (abierto = false)} {onLectura} />

<style>
	.boton-escanear :global(button) {
		min-height: 44px;
	}
</style>
