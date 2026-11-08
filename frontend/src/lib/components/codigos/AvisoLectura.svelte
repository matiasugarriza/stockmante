<script lang="ts">
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import type { AvisoLectura } from '#lib/codigos/lectura';

	// 020: aviso del resultado de una lectura; en error de red ofrece reintentar con el mismo
	// valor (FR-155). `Aviso` ya se anuncia a lectores de pantalla (FR-156).

	interface Props {
		aviso: AvisoLectura | null;
		onReintentar?: (valor: string) => void;
	}

	let { aviso, onReintentar }: Props = $props();
</script>

{#if aviso}
	<Aviso variante={aviso.variante}>
		{aviso.texto}
		{#snippet accion()}
			{#if aviso.variante === 'error' && onReintentar}
				<Boton tamano="sm" onclick={() => onReintentar(aviso.valor)}>Reintentar</Boton>
			{/if}
		{/snippet}
	</Aviso>
{/if}
