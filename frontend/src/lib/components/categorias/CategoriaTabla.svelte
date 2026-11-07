<script lang="ts">
	import type { Categoria } from '#lib/schemas/categorias';
	import Boton from '../ui/Boton.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';
	import TablaDatos from '../ui/TablaDatos.svelte';
	import type { Columna } from '../ui/tabla';

	interface Props {
		categorias: Categoria[];
		cargando?: boolean;
		onEditar: (categoria: Categoria) => void;
		onAlternarEstado: (categoria: Categoria) => void;
	}

	let { categorias, cargando = false, onEditar, onAlternarEstado }: Props = $props();

	const COLUMNAS: Columna<Categoria>[] = [
		{ clave: 'nombre', encabezado: 'Nombre' },
		{ clave: 'descripcion', encabezado: 'Descripción' },
		{ clave: 'activo', encabezado: 'Estado' },
		{ clave: 'acciones', encabezado: 'Acciones', alineacion: 'derecha' }
	];
</script>

<TablaDatos columnas={COLUMNAS} filas={categorias} etiqueta="Categorías" {cargando}>
	{#snippet fila(categoria)}
		<td class="principal">{categoria.nombre}</td>
		<td class="descripcion">{categoria.descripcion ?? '—'}</td>
		<td>
			<EtiquetaEstado variante={categoria.activo ? 'exito' : 'neutro'}>
				{categoria.activo ? 'Activa' : 'Inactiva'}
			</EtiquetaEstado>
		</td>
		<td>
			<div class="acciones">
				<Boton variante="secundario" tamano="sm" onclick={() => onEditar(categoria)}>
					Editar
				</Boton>
				<Boton variante="fantasma" tamano="sm" onclick={() => onAlternarEstado(categoria)}>
					{categoria.activo ? 'Desactivar' : 'Reactivar'}
				</Boton>
			</div>
		</td>
	{/snippet}
	{#snippet vacio()}
		<EstadoVacio
			icono="categorias"
			titulo="No hay categorías para este filtro"
			descripcion="Cambiá el filtro o creá la primera categoría."
		/>
	{/snippet}
</TablaDatos>

<style>
	.principal {
		font-weight: 500;
	}

	.descripcion {
		color: var(--texto-suave);
		max-width: 360px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 4px;
		flex-wrap: wrap;
	}
</style>
