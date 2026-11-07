<script lang="ts">
	import { ICONOS, type NombreIcono } from './iconos';

	interface Props {
		nombre: NombreIcono;
		tamano?: number;
		titulo?: string;
	}

	let { nombre, tamano = 18, titulo }: Props = $props();

	const elementos = $derived(ICONOS[nombre] ?? []);
	const etiqueta = $derived(titulo ?? undefined);
</script>

<svg
	width={tamano}
	height={tamano}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.75"
	stroke-linecap="round"
	stroke-linejoin="round"
	role={etiqueta ? 'img' : undefined}
	aria-hidden={etiqueta ? undefined : 'true'}
>
	{#if etiqueta}<title>{etiqueta}</title>{/if}
	{#each elementos as elemento}
		{#if elemento.tag === 'path'}
			<path d={elemento.d} />
		{:else if elemento.tag === 'circle'}
			<circle cx={elemento.cx} cy={elemento.cy} r={elemento.r} />
		{:else}
			<rect
				x={elemento.x}
				y={elemento.y}
				width={elemento.width}
				height={elemento.height}
				rx={elemento.rx ?? 0}
			/>
		{/if}
	{/each}
</svg>
