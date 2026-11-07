<script lang="ts">
	import { elegirPreferencia, preferencia, type PreferenciaTema } from '#lib/tema/tema';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import Icono from './Icono.svelte';
	import type { NombreIcono } from './iconos';

	interface Props {
		compacto?: boolean;
	}

	let { compacto = false }: Props = $props();

	const opciones: Array<{ valor: PreferenciaTema; etiqueta: string; icono: NombreIcono }> = [
		{ valor: 'light', etiqueta: 'Claro', icono: 'sol' },
		{ valor: 'dark', etiqueta: 'Oscuro', icono: 'luna' },
		{ valor: 'system', etiqueta: 'Sistema', icono: 'sistema' }
	];
</script>

{#if !$dispositivo.esMovil}
	<div class="tema" role="group" aria-label="Tema de la interfaz">
	{#each opciones as opcion (opcion.valor)}
		<button
			type="button"
			class="opcion"
			class:activa={$preferencia === opcion.valor}
			aria-pressed={$preferencia === opcion.valor}
			title={opcion.etiqueta}
			onclick={() => elegirPreferencia(opcion.valor)}
		>
			<Icono nombre={opcion.icono} tamano={15} />
			{#if !compacto}<span>{opcion.etiqueta}</span>{/if}
		</button>
	{/each}
	</div>
{/if}

<style>
	.tema {
		display: inline-flex;
		gap: 2px;
		padding: 3px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie-2);
	}

	.opcion {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		min-height: 28px;
		padding: 4px 8px;
		border: 0;
		border-radius: 4px;
		background: transparent;
		color: var(--texto-tenue);
		font-family: var(--fuente-sans);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
	}

	.opcion:hover {
		color: var(--texto);
	}

	.opcion.activa {
		background: var(--superficie);
		box-shadow: 0 0 0 1px var(--borde);
		color: var(--texto);
	}
</style>
