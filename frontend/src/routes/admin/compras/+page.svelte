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
	import { avisoResultadoCompra } from '#lib/utils/compras';
	import { paginaValida } from '#lib/utils/paginacion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	const POR_PAGINA = 20;

	const OPCIONES_ESTADO = [
		{ valor: '', etiqueta: 'Todas' },
		{ valor: 'BORRADOR', etiqueta: 'Borrador' },
		{ valor: 'CONFIRMADA', etiqueta: 'Confirmada' },
		{ valor: 'ANULADA', etiqueta: 'Anulada' }
	];

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
			aviso = avisoResultadoCompra(await confirmarCompra(compra.id));
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
			aviso = avisoResultadoCompra(await anularCompra(compra.id));
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

<div class="pagina">
	<EncabezadoPagina
		titulo="Compras"
		descripcion="Compras en borrador, confirmación de recepción con entrada de stock y anulación."
	>
		<Boton icono="mas" deshabilitado={!puedeCrear} onclick={abrirAlta}>Nueva compra</Boton>
	</EncabezadoPagina>

	{#if !puedeCrear}
		<Aviso variante="advertencia">
			Para crear una compra necesitás al menos un proveedor activo, un depósito activo y un
			producto activo.
		</Aviso>
	{/if}
	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}

	<Tarjeta>
		<div class="filtros">
			<Campo
				etiqueta="Estado"
				nombre="filtro-estado"
				tipo="select"
				opciones={OPCIONES_ESTADO}
				bind:valor={filtroEstado}
				onchange={(valor) => void cambiarFiltro(valor as '' | EstadoCompra)}
			/>
		</div>
	</Tarjeta>

	{#if !cargando && compras.length === 0}
		<Tarjeta>
			<EstadoVacio
				icono="compras"
				titulo="No hay compras en este filtro"
				descripcion={puedeCrear
					? 'Creá una compra en borrador y confirmá la recepción cuando llegue la mercadería.'
					: 'Primero cargá proveedores, depósitos y productos activos.'}
			>
				{#if puedeCrear}
					<Boton icono="mas" onclick={abrirAlta}>Crear la primera compra</Boton>
				{/if}
			</EstadoVacio>
		</Tarjeta>
	{:else}
		<Tarjeta densa={true}>
			<CompraTabla
				{compras}
				{cargando}
				onEditar={(compra) => void abrirEdicion(compra)}
				onConfirmar={(compra) => void confirmar(compra)}
				onAnular={(compra) => void anular(compra)}
				onEliminar={(compra) => void eliminar(compra)}
			/>
		</Tarjeta>
		<div class="pie">
			<Paginacion {pagina} {total} porPagina={POR_PAGINA} onCambiar={irAPagina} />
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

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filtros {
		display: flex;
		align-items: flex-end;
		gap: 12px;
	}

	.filtros :global(.campo) {
		min-width: 220px;
	}

	.pie {
		padding: 0 2px;
	}
</style>
