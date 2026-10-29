<script lang="ts">
	import { goto } from '$app/navigation';
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

	$effect(() => {
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol !== 'GESTION') {
			void goto('/');
		}
	});

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

	const filtros: { valor: FiltroActivo; etiqueta: string }[] = [
		{ valor: 'true', etiqueta: 'Activas' },
		{ valor: 'false', etiqueta: 'Inactivas' },
		{ valor: 'all', etiqueta: 'Todas' }
	];
</script>

<svelte:head>
	<title>Categorías · Stockmante</title>
</svelte:head>

<div class="flex flex-col gap-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h1 class="text-xl font-semibold">Categorías</h1>
			<p class="text-sm text-slate-500">
				Nombre único sin distinguir mayúsculas. La baja es lógica: nunca se borra una categoría.
			</p>
		</div>
		<button
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
			onclick={abrirAlta}
		>
			Nueva categoría
		</button>
	</div>

	{#if error}
		<p class="rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
	{/if}
	{#if aviso}
		<p class="rounded border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
			{aviso}
		</p>
	{/if}

	<div class="flex items-center gap-1">
		{#each filtros as opcion (opcion.valor)}
			<button
				class="rounded px-3 py-1 text-sm {filtro === opcion.valor
					? 'bg-slate-800 text-white'
					: 'border border-slate-300 text-slate-600 hover:bg-slate-100'}"
				onclick={() => void cambiarFiltro(opcion.valor)}
			>
				{opcion.etiqueta}
			</button>
		{/each}
	</div>

	{#if cargando}
		<p class="py-6 text-center text-sm text-slate-500">Cargando categorías…</p>
	{:else}
		<CategoriaTabla
			{categorias}
			onEditar={abrirEdicion}
			onAlternarEstado={(categoria) => void alternarEstado(categoria)}
		/>
	{/if}
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
