<script lang="ts">
	import { goto } from '$app/navigation';
	import { listarDepositos } from '#lib/api/depositos';
	import {
		actualizarCompra,
		anularCompra,
		confirmarCompra,
		crearCompra,
		eliminarCompra,
		listarCompras,
		obtenerCompra
	} from '#lib/api/compras';
	import { listarProductos } from '#lib/api/productos';
	import { listarProveedores } from '#lib/api/proveedores';
	import CompraForm from '#lib/components/compras/CompraForm.svelte';
	import CompraTabla from '#lib/components/compras/CompraTabla.svelte';
	import type { Compra, CompraDetalle, CrearCompraInput, EstadoCompra } from '#lib/schemas/compras';
	import type { Deposito } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import type { Proveedor } from '#lib/schemas/proveedores';
	import { sesion } from '#lib/stores/sesion';
	import { mensajeError } from '#lib/utils/errores';
	import { paginaValida, totalPaginas } from '#lib/utils/paginacion';

	const POR_PAGINA = 20;

	let compras = $state<Compra[]>([]);
	let proveedores = $state<Proveedor[]>([]);
	let depositos = $state<Deposito[]>([]);
	let productos = $state<Producto[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	let filtroEstado = $state<'' | EstadoCompra>('');
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let errorModal = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let modalAbierto = $state(false);
	let editando = $state<CompraDetalle | null>(null);
	const paginas = $derived(totalPaginas(total, POR_PAGINA));
	const puedeCrear = $derived(
		proveedores.length > 0 && depositos.length > 0 && productos.length > 0
	);

	$effect(() => {
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol !== 'GESTION') {
			void goto('/caja');
		}
	});

	$effect(() => {
		if (!cargado && $sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION') {
			cargado = true;
			void iniciar();
		}
	});

	async function iniciar() {
		await Promise.all([cargarCatalogos(), cargar()]);
	}

	async function cargarCatalogos() {
		try {
			const [listaProveedores, listaDepositos, listaProductos] = await Promise.all([
				listarProveedores({ activo: 'true', porPagina: 100 }),
				listarDepositos('true'),
				listarProductos({ activo: 'true', porPagina: 100 })
			]);
			proveedores = listaProveedores.items;
			depositos = listaDepositos;
			productos = listaProductos.items;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los catálogos para la compra.');
		}
	}

	async function cargar() {
		cargando = true;
		error = null;
		try {
			let lista = await listarCompras({
				estado: filtroEstado || undefined,
				pagina,
				porPagina: POR_PAGINA
			});
			const valida = paginaValida(pagina, lista.total, POR_PAGINA);
			if (valida !== pagina) {
				pagina = valida;
				lista = await listarCompras({
					estado: filtroEstado || undefined,
					pagina,
					porPagina: POR_PAGINA
				});
			}
			compras = lista.items;
			total = lista.total;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar las compras.');
		} finally {
			cargando = false;
		}
	}

	async function cambiarFiltro(nuevo: '' | EstadoCompra) {
		filtroEstado = nuevo;
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

	async function abrirEdicion(compra: Compra) {
		error = null;
		errorModal = null;
		aviso = null;
		try {
			editando = await obtenerCompra(compra.id);
			modalAbierto = true;
		} catch (e) {
			error = mensajeError(e, 'No se pudo abrir la compra.');
		}
	}

	function cerrarModal() {
		if (!guardando) {
			modalAbierto = false;
			errorModal = null;
		}
	}

	async function guardar(datos: CrearCompraInput) {
		guardando = true;
		error = null;
		errorModal = null;
		try {
			if (editando) {
				await actualizarCompra(editando.compra.id, datos);
				aviso = 'Compra actualizada.';
			} else {
				await crearCompra(datos);
				aviso = 'Compra creada en borrador.';
			}
			modalAbierto = false;
			await cargar();
		} catch (e) {
			errorModal = mensajeError(e, 'No se pudo guardar la compra.');
		} finally {
			guardando = false;
		}
	}

	async function confirmar(compra: Compra) {
		if (!confirm(`¿Confirmar la recepción de la compra #${compra.id}? Se moverá stock.`)) return;
		error = null;
		aviso = null;
		try {
			await confirmarCompra(compra.id);
			aviso = `Compra #${compra.id} confirmada. El stock ya está actualizado.`;
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo confirmar la compra.');
		}
	}

	async function anular(compra: Compra) {
		if (!confirm(`¿Anular la compra #${compra.id}? Se revertirá el stock recibido.`)) return;
		error = null;
		aviso = null;
		try {
			await anularCompra(compra.id);
			aviso = `Compra #${compra.id} anulada.`;
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo anular la compra.');
		}
	}

	async function eliminar(compra: Compra) {
		if (!confirm(`¿Eliminar el borrador #${compra.id}? Esta acción no mueve stock.`)) return;
		error = null;
		aviso = null;
		try {
			await eliminarCompra(compra.id);
			aviso = `Borrador #${compra.id} eliminado.`;
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo eliminar el borrador.');
		}
	}
</script>

<svelte:head>
	<title>Compras · Stockmante</title>
</svelte:head>

<div class="flex flex-col gap-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h1 class="text-xl font-semibold">Compras</h1>
			<p class="text-sm text-slate-500">
				Compras en borrador, confirmación de recepción con entrada de stock y anulación.
			</p>
		</div>
		<button
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40"
			onclick={abrirAlta}
			disabled={!puedeCrear}
		>
			Nueva compra
		</button>
	</div>

	{#if !puedeCrear && (proveedores.length === 0 || depositos.length === 0 || productos.length === 0)}
		<p class="rounded border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
			Para crear una compra necesitás al menos un proveedor activo, un depósito activo y un
			producto activo.
		</p>
	{/if}

	{#if error}
		<p class="rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
	{/if}
	{#if aviso}
		<p class="rounded border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
			{aviso}
		</p>
	{/if}

	<div class="flex items-center gap-2 text-sm">
		<label class="flex items-center gap-2">
			<span class="text-slate-500">Estado</span>
			<select
				class="rounded border border-slate-300 px-3 py-1.5"
				bind:value={filtroEstado}
				onchange={() => void cambiarFiltro(filtroEstado)}
			>
				<option value="">Todas</option>
				<option value="BORRADOR">Borrador</option>
				<option value="CONFIRMADA">Confirmada</option>
				<option value="ANULADA">Anulada</option>
			</select>
		</label>
	</div>

	{#if cargando}
		<p class="py-6 text-center text-sm text-slate-500">Cargando compras...</p>
	{:else if compras.length === 0}
		<div class="rounded border border-dashed border-slate-300 bg-white px-6 py-10 text-center">
			<p class="text-sm font-medium text-slate-700">No hay compras en este filtro.</p>
			<p class="mt-1 text-sm text-slate-500">
				{puedeCrear
					? 'Creá una compra en borrador y confirmá la recepción cuando llegue la mercadería.'
					: 'Primero cargá proveedores, depósitos y productos activos.'}
			</p>
			{#if puedeCrear}
				<button
					class="mt-4 rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
					onclick={abrirAlta}
				>
					Crear la primera compra
				</button>
			{/if}
		</div>
	{:else}
		<CompraTabla
			{compras}
			onEditar={(compra) => void abrirEdicion(compra)}
			onConfirmar={(compra) => void confirmar(compra)}
			onAnular={(compra) => void anular(compra)}
			onEliminar={(compra) => void eliminar(compra)}
		/>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<p class="text-xs text-slate-500">
				{compras.length} de {total} compra{total === 1 ? '' : 's'} · página {pagina} de
				{paginas}
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
	<CompraForm
		compra={editando}
		{proveedores}
		{depositos}
		{productos}
		{guardando}
		error={errorModal}
		onGuardar={(datos) => void guardar(datos)}
		onCerrar={cerrarModal}
	/>
{/if}