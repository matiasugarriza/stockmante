<script lang="ts">
	interface Props {
		variante?: 'linea' | 'bloque' | 'tarjeta';
		alto?: string;
		ancho?: string;
	}

	let { variante = 'linea', alto, ancho }: Props = $props();

	const altura = $derived(
		alto ?? (variante === 'linea' ? '12px' : variante === 'bloque' ? '120px' : '180px')
	);
	const anchura = $derived(ancho ?? '100%');
</script>

<div
	class="esqueleto {variante}"
	style:height={altura}
	style:width={anchura}
	aria-hidden="true"
></div>

<style>
	.esqueleto {
		position: relative;
		overflow: hidden;
		border-radius: var(--radio-control);
		background: var(--superficie-2);
	}

	.esqueleto::after {
		position: absolute;
		inset: 0;
		content: '';
		transform: translateX(-100%);
		background: linear-gradient(
			90deg,
			transparent,
			color-mix(in oklab, var(--texto) 7%, transparent),
			transparent
		);
		animation: brillo 1.4s ease-in-out infinite;
	}

	.tarjeta {
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta);
		background: var(--superficie);
	}

	@keyframes brillo {
		to {
			transform: translateX(100%);
		}
	}
</style>
