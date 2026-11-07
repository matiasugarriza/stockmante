<script lang="ts">
	import type { FiltroActivo } from '#lib/api/categorias';
	import {
		actualizarCategoria,
		crearCategoria,
		desactivarCategoria,
		listarCategorias,
		reactivarCategoria
	} from '#lib/api/categorias';
	import { ApiError } from '#lib/api/client';
	import CategoriaForm from '#lib/components/categorias/CategoriaForm.svelte';
	import CategoriaTabla from '#lib/components/categorias/CategoriaTabla.svelte';
	import type { Categoria } from '#lib/schemas/categorias';
	import { sesion } from '#lib/stores/sesion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	let categorias = $state<Categoria[]>([]);
	let filtro = $state<FiltroActivo>('true');
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let errorModal = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let modalAbierto = $state(false);
	let editando = $state<Categoria | null>(null);

	let filtros: { valor: FiltroActivo; etiqueta: string }[] = [
		{ valor: 'true', etiqueta: 'Activas' },
		{ valor: 'false', etiqueta: 'Inactivas' },
		{ valor: 'all', etiqueta: 'Todas' }
	];

	$effect(() => {
		if (
			!cargado &&
			$sesion.estado === 'activa' &&
			$sesion.usuario.familia_rol === 'GESTION'
		) {
			cargado = true;
			void cargar();
		}
	});

	function mensajeError(e: unknown, fallback: string): string {
		if (e instanceof ApiError) return e.message;
		if (e instanceof Error) return e.message;
		return fallback;
	}

	async function cargar() {
		cargando = true;
		error = null;
		try {
			categorias = await listarCategorias(filtro);
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar las categorías');
		} finally {
			cargando = false;
		}
	}

	async function cambiarFiltro(nuevo: FiltroActivo) {
		if (nuevo === filtro) return;
		filtro = nuevo;
		await cargar();
	}

	function abrirAlta() {
		editando = null;
		modalAbierto = true;
		error = null;
		errorModal = null;
		aviso = null;
	}

	function abrirEdicion(categoria: Categoria) {
		editando = categoria;
		modalAbierto = true;
		error = null;
		errorModal = null;
		aviso = null;
	}

	function cerrarModal() {
		if (!guardando) {
			modalAbierto = false;
			errorModal = null;
		}
	}

	async function guardar(datos: { nombre: string; descripcion: string | null }) {
		guardando = true;
		error = null;
		errorModal = null;
		try {
			if (editando) {
				await actualizarCategoria(editando.id, datos);
				aviso = 'Categoría actualizada.';
			} else {
				await crearCategoria(datos);
				aviso = 'Categoría creada.';
			}
			modalAbierto = false;
			await cargar();
		} catch (e) {
			errorModal = mensajeError(e, 'No se pudo guardar la categoría');
		} finally {
			guardando = false;
		}
	}

	async function alternarEstado(categoria: Categoria) {
		error = null;
		aviso = null;
		try {
			if (categoria.activo) {
				await desactivarCategoria(categoria.id);
				aviso = `${categoria.nombre} quedó inactiva. Sus productos no se modificaron.`;
			} else {
				await reactivarCategoria(categoria.id);
				aviso = `${categoria.nombre} volvió a estar activa.`;
			}
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado de la categoría');
		}
	}
</script>

<svelte:head>
	<title>Categorías · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Categorías"
		descripcion="Nombre único sin distinguir mayúsculas. La baja es lógica: nunca se borra una categoría."
	>
		<Boton icono="mas" onclick={abrirAlta}>Nueva categoría</Boton>
	</EncabezadoPagina>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}

	<div class="filtros" role="group" aria-label="Filtro de estado">
		{#each filtros as opcion (opcion.valor)}
			<Boton
				variante={filtro === opcion.valor ? 'primario' : 'secundario'}
				tamano="sm"
				onclick={() => void cambiarFiltro(opcion.valor)}
			>
				{opcion.etiqueta}
			</Boton>
		{/each}
	</div>

	<Tarjeta densa={true}>
		<CategoriaTabla
			{categorias}
			{cargando}
			onEditar={abrirEdicion}
			onAlternarEstado={(categoria) => void alternarEstado(categoria)}
		/>
	</Tarjeta>
</div>

{#if modalAbierto}
	<CategoriaForm
		categoria={editando}
		{guardando}
		error={errorModal}
		onGuardar={(datos) => void guardar(datos)}
		onCerrar={cerrarModal}
	/>
{/if}

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filtros {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}
</style>
