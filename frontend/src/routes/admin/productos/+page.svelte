<script lang="ts">
	import { goto } from '$app/navigation';
	import { listarCategorias } from '#lib/api/categorias';
	import type { FiltroActivo } from '#lib/api/productos';
	import {
		actualizarProducto,
		crearProducto,
		desactivarProducto,
		listarProductos,
		reactivarProducto
	} from '#lib/api/productos';
	import { ApiError } from '#lib/api/client';
	import FiltrosCatalogo from '#lib/components/productos/FiltrosCatalogo.svelte';
	import ProductoForm from '#lib/components/productos/ProductoForm.svelte';
	import ProductoTabla from '#lib/components/productos/ProductoTabla.svelte';
	import type { Categoria } from '#lib/schemas/categorias';
	import type { CrearProductoInput, Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';
	import { paginaValida, totalPaginas } from '#lib/utils/paginacion';

	const POR_PAGINA = 20;

	let productos = $state<Producto[]>([]);
	let categorias = $state<Categoria[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	let q = $state('');
	let categoriaId = $state('');
	let filtro = $state<FiltroActivo>('true');
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let errorModal = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let modalAbierto = $state(false);
	let editando = $state<Producto | null>(null);
	const paginas = $derived(totalPaginas(total, POR_PAGINA));

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
			void iniciar();
		}
	});

	function mensajeError(e: unknown, fallback: string): string {
		if (e instanceof ApiError) return e.message;
		if (e instanceof Error) return e.message;
		return fallback;
	}

	async function iniciar() {
		await cargarCategorias();
		await cargar();
	}

	async function cargarCategorias() {
		try {
			// `all` alimenta el filtro y el formulario; el formulario deshabilita
			// las inactivas salvo la categoría actual del producto editado.
			categorias = await listarCategorias('all');
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar las categorías');
		}
	}

	async function cargar() {
		cargando = true;
		error = null;
		try {
			let lista = await listarProductos({
				q: q.trim() || undefined,
				categoriaId: categoriaId ? Number(categoriaId) : undefined,
				activo: filtro,
				pagina,
				porPagina: POR_PAGINA
			});
			// Una baja o un cambio de filtro puede vaciar la página actual.
			const valida = paginaValida(pagina, lista.total, POR_PAGINA);
			if (valida !== pagina) {
				pagina = valida;
				lista = await listarProductos({
					q: q.trim() || undefined,
					categoriaId: categoriaId ? Number(categoriaId) : undefined,
					activo: filtro,
					pagina,
					porPagina: POR_PAGINA
				});
			}
			productos = lista.items;
			total = lista.total;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los productos');
		} finally {
			cargando = false;
		}
	}

	async function aplicarFiltros(nuevos: {
		q: string;
		categoriaId: string;
		activo: FiltroActivo;
	}) {
		q = nuevos.q;
		categoriaId = nuevos.categoriaId;
		filtro = nuevos.activo;
		pagina = 1;
		await cargar();
	}

	async function irAPagina(nueva: number) {
		if (cargando || nueva === pagina || nueva < 1 || nueva > paginas) return;
		pagina = nueva;
		await cargar();
	}

	function abrirAlta() {
		editando = null;
		modalAbierto = true;
		error = null;
		errorModal = null;
		aviso = null;
	}

	function abrirEdicion(producto: Producto) {
		editando = producto;
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

	async function guardar(datos: CrearProductoInput) {
		guardando = true;
		error = null;
		errorModal = null;
		try {
			if (editando) {
				await actualizarProducto(editando.id, datos);
				aviso = 'Producto actualizado.';
			} else {
				await crearProducto(datos);
				aviso = 'Producto creado.';
			}
			modalAbierto = false;
			await cargar();
		} catch (e) {
			// El modal tapa el banner de la página: el 409/422 se muestra dentro.
			errorModal = mensajeError(e, 'No se pudo guardar el producto');
		} finally {
			guardando = false;
		}
	}

	async function alternarEstado(producto: Producto) {
		error = null;
		aviso = null;
		try {
			if (producto.activo) {
				await desactivarProducto(producto.id);
				aviso = `${producto.nombre} quedó inactivo. No se borró ningún dato.`;
			} else {
				await reactivarProducto(producto.id);
				aviso = `${producto.nombre} volvió a estar activo.`;
			}
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado del producto');
		}
	}
</script>

<svelte:head>
	<title>Productos · Stockmante</title>
</svelte:head>

<div class="flex flex-col gap-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h1 class="text-xl font-semibold">Productos</h1>
			<p class="text-sm text-slate-500">
				Catálogo maestro con SKU único y baja lógica: nunca se borra un producto.
			</p>
		</div>
		<button
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
			onclick={abrirAlta}
		>
			Nuevo producto
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

	<FiltrosCatalogo
		{q}
		{categoriaId}
		activo={filtro}
		{categorias}
		onFiltrar={(nuevos) => void aplicarFiltros(nuevos)}
	/>

	{#if cargando}
		<p class="py-6 text-center text-sm text-slate-500">Cargando productos…</p>
	{:else}
		<ProductoTabla
			{productos}
			onEditar={abrirEdicion}
			onAlternarEstado={(producto) => void alternarEstado(producto)}
		/>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<p class="text-xs text-slate-500">
				{productos.length} de {total} producto{total === 1 ? '' : 's'} en este filtro · página
				{pagina} de {paginas}
			</p>
			<div class="flex items-center gap-2">
				<button
					class="rounded border border-slate-300 px-3 py-1 text-sm text-slate-600 hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-40"
					disabled={pagina <= 1}
					onclick={() => void irAPagina(pagina - 1)}
				>
					Anterior
				</button>
				<button
					class="rounded border border-slate-300 px-3 py-1 text-sm text-slate-600 hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-40"
					disabled={pagina >= paginas}
					onclick={() => void irAPagina(pagina + 1)}
				>
					Siguiente
				</button>
			</div>
		</div>
	{/if}
</div>

{#if modalAbierto}
	<ProductoForm
		producto={editando}
		{categorias}
		{guardando}
		error={errorModal}
		onGuardar={(datos) => void guardar(datos)}
		onCerrar={cerrarModal}
	/>
{/if}
