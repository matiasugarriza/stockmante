<script lang="ts">
	import Boton from './Boton.svelte';

	interface Props {
		pagina: number;
		total: number;
		porPagina: number;
		onCambiar: (pagina: number) => void;
		etiqueta?: string;
	}

	let { pagina, total, porPagina, onCambiar, etiqueta }: Props = $props();

	const paginas = $derived(Math.max(1, Math.ceil(total / Math.max(1, porPagina))));
	const desde = $derived(total === 0 ? 0 : (pagina - 1) * porPagina + 1);
	const hasta = $derived(Math.min(pagina * porPagina, total));
</script>

<nav class="paginacion" aria-label={etiqueta ?? 'Paginación'}>
	<span class="resumen">{desde}–{hasta} de {total}</span>
	<div class="controles">
		<Boton
			variante="secundario"
			tamano="sm"
			deshabilitado={pagina <= 1}
			onclick={() => onCambiar(pagina - 1)}
		>
			Anterior
		</Boton>
		<span class="pagina">Página {pagina} de {paginas}</span>
		<Boton
			variante="secundario"
			tamano="sm"
			deshabilitado={pagina >= paginas}
			onclick={() => onCambiar(pagina + 1)}
		>
			Siguiente
		</Boton>
	</div>
</nav>

<style>
	.paginacion {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}

	.resumen {
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	.controles {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.pagina {
		color: var(--texto-suave);
		font-size: 12.5px;
		white-space: nowrap;
	}

	@media (max-width: 520px) {
		.resumen {
			display: none;
		}

		.paginacion {
			justify-content: flex-end;
		}
	}
</style>
