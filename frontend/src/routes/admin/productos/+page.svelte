<script lang="ts">
	import { listarCategorias } from '#lib/api/categorias';
	import { listarDepositos, ubicacionesDeProductos } from '#lib/api/depositos';
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
	import type { Deposito, Ubicacion } from '#lib/schemas/depositos';
	import type { CrearProductoInput, Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';
	import { paginaValida } from '#lib/utils/paginacion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	const POR_PAGINA = 20;

	let productos = $state<Producto[]>([]);
	let categorias = $state<Categoria[]>([]);
	let depositos = $state<Deposito[]>([]);
	let ubicacionesPorProducto = $state<Record<number, Ubicacion[]>>({});
	let advertenciaUbicaciones = $state<string | null>(null);
	let advertenciaDepositos = $state<string | null>(null);
	let generacion = 0;
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
		await Promise.all([cargarCategorias(), cargarDepositos()]);
		await cargar();
	}

	async function cargarDepositos() {
		try {
			// `all` para poder mostrar el nombre aunque el depósito esté inactivo.
			depositos = await listarDepositos('all');
		} catch (e) {
			depositos = [];
			advertenciaDepositos = `No se pudieron cargar los depósitos: ${mensajeError(e, 'error desconocido')}. Las ubicaciones se muestran sin el nombre del depósito.`;
		}
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
			await cargarUbicacionesDePagina(productos);
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los productos');
		} finally {
			cargando = false;
		}
	}

	/** Una sola llamada por página (FR-075); no se confunde «sin ubicación» con «sin datos». */
	async function cargarUbicacionesDePagina(items: Producto[]) {
		if (items.length === 0) {
			ubicacionesPorProducto = {};
			return;
		}
		const actual = ++generacion;
		try {
			const mapa = await ubicacionesDeProductos(items.map((item) => item.id));
			if (actual !== generacion) return;
			ubicacionesPorProducto = mapa;
			advertenciaUbicaciones = null;
		} catch (e) {
			if (actual !== generacion) return;
			ubicacionesPorProducto = {};
			advertenciaUbicaciones = `No se pudieron cargar las ubicaciones: ${mensajeError(e, 'error desconocido')}. El listado sigue disponible.`;
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

	function irAPagina(nueva: number) {
		if (cargando || nueva === pagina || nueva < 1) return;
		pagina = nueva;
		void cargar();
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

<div class="pagina">
	<EncabezadoPagina
		titulo="Productos"
		descripcion="Catálogo maestro con SKU único y baja lógica: nunca se borra un producto."
	>
		<Boton icono="mas" onclick={abrirAlta}>Nuevo producto</Boton>
	</EncabezadoPagina>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}
	{#if advertenciaDepositos}<Aviso variante="advertencia">{advertenciaDepositos}</Aviso>{/if}
	{#if advertenciaUbicaciones}<Aviso variante="advertencia">{advertenciaUbicaciones}</Aviso>{/if}

	<FiltrosCatalogo
		{q}
		{categoriaId}
		activo={filtro}
		{categorias}
		onFiltrar={(nuevos) => void aplicarFiltros(nuevos)}
	/>

	<Tarjeta densa={true}>
		<ProductoTabla
			{productos}
			{ubicacionesPorProducto}
			{depositos}
			{cargando}
			onEditar={abrirEdicion}
			onAlternarEstado={(producto) => void alternarEstado(producto)}
		/>
	</Tarjeta>

	<div class="pie">
		<Paginacion {pagina} {total} porPagina={POR_PAGINA} onCambiar={irAPagina} />
	</div>
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

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.pie {
		padding: 0 2px;
	}
</style>
