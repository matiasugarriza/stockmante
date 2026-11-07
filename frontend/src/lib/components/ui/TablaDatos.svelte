<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import Aviso from './Aviso.svelte';
	import EstadoVacio from './EstadoVacio.svelte';
	import type { Columna } from './tabla';

	interface Props {
		columnas: Columna<T>[];
		filas: T[];
		etiqueta: string;
		cargando?: boolean;
		error?: string | null;
		vacio?: Snippet;
		fila?: Snippet<[T]>;
		pie?: Snippet;
		compacta?: boolean;
	}

	let {
		columnas,
		filas,
		etiqueta,
		cargando = false,
		error = null,
		vacio,
		fila,
		pie,
		compacta = false
	}: Props = $props();

	function valorCelda(dato: T, clave: string): string {
		const valor = (dato as Record<string, unknown>)[clave];
		return valor === null || valor === undefined || valor === '' ? '—' : String(valor);
	}

	const alineacion = (columna: Columna<T>): string => `alinear-${columna.alineacion ?? 'izquierda'}`;
</script>

{#if error}
	<Aviso variante="error" titulo="No se pudo cargar">{error}</Aviso>
{:else}
	<div class="envoltura">
		<table class="tabla" class:compacta aria-label={etiqueta} aria-busy={cargando ? 'true' : undefined}>
			<thead>
				<tr>
					{#each columnas as columna (columna.clave)}
						<th scope="col" class="{alineacion(columna)} {columna.clase ?? ''}">
							{columna.encabezado}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#if cargando}
					{#each Array(5) as _, indice (indice)}
						<tr>
							{#each columnas as columna (columna.clave)}
								<td class={alineacion(columna)}>
									<span class="linea-esqueleto" style:width={indice % 2 === 0 ? '70%' : '50%'}
									></span>
								</td>
							{/each}
						</tr>
					{/each}
				{:else if filas.length === 0}
					<tr>
						<td colspan={columnas.length} class="celda-vacia">
							{#if vacio}
								{@render vacio()}
							{:else}
								<EstadoVacio
									titulo="Sin resultados"
									descripcion="Probá con otro filtro o agregá el primer registro."
								/>
							{/if}
						</td>
					</tr>
				{:else}
					{#each filas as dato, indice (indice)}
						<tr>
							{#if fila}
								{@render fila(dato)}
							{:else}
								{#each columnas as columna (columna.clave)}
									<td class={alineacion(columna)}>{valorCelda(dato, columna.clave)}</td>
								{/each}
							{/if}
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>
	{#if pie}
		<div class="pie">{@render pie()}</div>
	{/if}
{/if}

<style>
	.envoltura {
		width: 100%;
		overflow-x: auto;
	}

	.tabla {
		width: 100%;
		border-collapse: collapse;
		font-size: 13.5px;
	}

	th {
		padding: 10px 14px;
		border-bottom: 1px solid var(--borde);
		background: var(--superficie-2);
		color: var(--texto-tenue);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		white-space: nowrap;
	}

	td {
		padding: 12px 14px;
		border-bottom: 1px solid var(--borde);
		color: var(--texto);
		vertical-align: middle;
	}

	tbody tr:last-child td {
		border-bottom: 0;
	}

	tbody tr:hover td {
		background: color-mix(in oklab, var(--primario) 4%, transparent);
	}

	.compacta th {
		padding: 8px 10px;
	}

	.compacta td {
		padding: 9px 10px;
	}

	.alinear-derecha {
		text-align: right;
	}

	.alinear-centro {
		text-align: center;
	}

	.alinear-izquierda {
		text-align: left;
	}

	.linea-esqueleto {
		display: inline-block;
		height: 10px;
		border-radius: 4px;
		background: var(--superficie-3);
	}

	.celda-vacia {
		padding: 0;
	}

	.pie {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
		border-top: 1px solid var(--borde);
		padding: 12px 16px;
		background: var(--superficie);
	}
</style>
