<script lang="ts">
	import LogoSimbolo from './LogoSimbolo.svelte';

	/**
	 * Logotipo completo: símbolo + "STOCKMANTE" con el tratamiento del logo original
	 * (ST en acento, OCK en peso fuerte y MANTE en peso liviano), en Outfit.
	 */
	interface Props {
		/** Alto del símbolo en px; el texto escala en proporción. */
		alto?: number;
		/** Oculta el texto (barra lateral colapsada). */
		soloSimbolo?: boolean;
	}

	let { alto = 34, soloSimbolo = false }: Props = $props();
</script>

<span class="logo-marca" style:--alto-logo={`${alto}px`}>
	<LogoSimbolo {alto} />
	{#if !soloSimbolo}
		<span class="texto"><span class="st">ST</span><span class="ock">OCK</span><span class="mante">MANTE</span></span>
	{/if}
</span>

<style>
	.logo-marca {
		display: inline-flex;
		align-items: center;
		gap: calc(var(--alto-logo) * 0.28);
		min-width: 0;
	}

	.texto {
		color: var(--texto);
		font-family: var(--fuente-sans);
		font-size: calc(var(--alto-logo) * 0.5);
		letter-spacing: 0.08em;
		line-height: 1;
		white-space: nowrap;
	}

	.st {
		color: var(--logo-acento);
		font-weight: 700;
	}

	.ock {
		font-weight: 700;
	}

	.mante {
		font-weight: 400;
	}
</style>
