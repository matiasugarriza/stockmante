<script lang="ts">
	import { indiceAnterior, indiceSiguiente, normalizarIndice } from '#lib/utils/carrusel';
	import TarjetaKpi from '../ui/TarjetaKpi.svelte';
	import Icono from '../ui/Icono.svelte';
	import type { MetricaTablero } from './metricas';

	interface Props {
		metricas: MetricaTablero[];
	}

	let { metricas }: Props = $props();

	let indice = $state(0);
	let pista = $state<HTMLUListElement>();

	function prefiereMenosMovimiento(): boolean {
		return (
			typeof window !== 'undefined' &&
			window.matchMedia('(prefers-reduced-motion: reduce)').matches
		);
	}

	function desplazarA(nuevo: number) {
		indice = normalizarIndice(nuevo, metricas.length);
		const destino = pista?.children[indice] as HTMLElement | undefined;
		destino?.scrollIntoView({
			behavior: prefiereMenosMovimiento() ? 'auto' : 'smooth',
			block: 'nearest',
			inline: 'start'
		});
	}

	function alDesplazar() {
		if (!pista) return;
		const ancho = pista.clientWidth || 1;
		indice = normalizarIndice(Math.round(pista.scrollLeft / ancho), metricas.length);
	}
</script>

<div
	class="carrusel"
	role="group"
	aria-roledescription="carrusel"
	aria-label="Métricas del inventario"
>
	<div class="cabecera">
		<span class="posicion" aria-live="polite">{indice + 1} de {metricas.length}</span>
		<div class="controles">
			<button
				type="button"
				class="control"
				aria-label="Métrica anterior"
				onclick={() => desplazarA(indiceAnterior(indice, metricas.length))}
			>
				<Icono nombre="flecha" tamano={16} />
			</button>
			<button
				type="button"
				class="control siguiente"
				aria-label="Métrica siguiente"
				onclick={() => desplazarA(indiceSiguiente(indice, metricas.length))}
			>
				<Icono nombre="flecha" tamano={16} />
			</button>
		</div>
	</div>

	<ul class="pista" bind:this={pista} onscroll={alDesplazar}>
		{#each metricas as metrica, posicion (metrica.id)}
			<li
				class="slide"
				role="group"
				aria-roledescription="tarjeta"
				aria-label={`${posicion + 1} de ${metricas.length}: ${metrica.etiqueta}`}
			>
				<TarjetaKpi
					etiqueta={metrica.etiqueta}
					valor={metrica.valor}
					contexto={metrica.contexto}
					icono={metrica.icono}
					variante={metrica.variante}
					cargando={metrica.cargando}
				/>
			</li>
		{/each}
	</ul>

	<div class="puntos" role="group" aria-label="Ir a una métrica">
		{#each metricas as metrica, posicion (metrica.id)}
			<button
				type="button"
				class="punto"
				class:activo={posicion === indice}
				aria-label={`Ir a ${metrica.etiqueta}`}
				aria-current={posicion === indice ? 'true' : undefined}
				onclick={() => desplazarA(posicion)}
			></button>
		{/each}
	</div>
</div>

<style>
	.carrusel {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.cabecera {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 2px;
	}

	.posicion {
		color: var(--texto-tenue);
		font-size: 11px;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.controles {
		display: flex;
		gap: 4px;
	}

	.control {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--toque-min);
		height: 36px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto-suave);
		cursor: pointer;
	}

	.control.siguiente {
		transform: rotate(180deg);
	}

	.control:hover {
		border-color: var(--primario);
		color: var(--primario-texto);
	}

	.pista {
		display: flex;
		gap: 10px;
		margin: 0;
		padding: 0;
		overflow-x: auto;
		scroll-behavior: smooth;
		scroll-snap-type: x mandatory;
		scrollbar-width: none;
		list-style: none;
	}

	.pista::-webkit-scrollbar {
		display: none;
	}

	.slide {
		flex: 0 0 100%;
		scroll-snap-align: start;
	}

	.puntos {
		display: flex;
		justify-content: center;
		gap: 6px;
	}

	.punto {
		width: 18px;
		height: 18px;
		padding: 0;
		border: 0;
		background: transparent;
		cursor: pointer;
	}

	.punto::after {
		content: '';
		display: block;
		width: 7px;
		height: 7px;
		margin: 0 auto;
		border-radius: 50%;
		background: var(--borde-fuerte);
	}

	.punto.activo::after {
		background: var(--primario);
		box-shadow: var(--brillo-cian);
	}

	@media (min-width: 768px) {
		.slide {
			flex-basis: calc(50% - 5px);
		}
	}
</style>
