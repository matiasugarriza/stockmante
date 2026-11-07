<script lang="ts">
	import { goto } from '$app/navigation';
	import {
		actualizarProveedor,
		crearProveedor,
		desactivarProveedor,
		listarProveedores,
		reactivarProveedor,
		type FiltroActivo
	} from '#lib/api/proveedores';
	import ProveedorForm from '#lib/components/proveedores/ProveedorForm.svelte';
	import ProveedorTabla from '#lib/components/proveedores/ProveedorTabla.svelte';
	import type { CrearProveedorInput, Proveedor } from '#lib/schemas/proveedores';
	import { sesion } from '#lib/stores/sesion';
	import { mensajeError } from '#lib/utils/errores';
	import { paginaValida } from '#lib/utils/paginacion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	const POR_PAGINA = 20;

	let proveedores = $state<Proveedor[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	let filtro = $state<FiltroActivo>('all');
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let errorModal = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let modalAbierto = $state(false);
	let editando = $state<Proveedor | null>(null);

	const OPCIONES_ESTADO = [
		{ valor: 'all', etiqueta: 'Todos' },
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' }
	];

	$effect(() => {
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol !== 'GESTION') {
			void goto('/caja');
		}
	});

	$effect(() => {
		if (!cargado && $sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION') {
			cargado = true;
			void cargar();
		}
	});

	async function cargar() {
		cargando = true;
		error = null;
		try {
			let lista = await listarProveedores({ activo: filtro, pagina, porPagina: POR_PAGINA });
			const valida = paginaValida(pagina, lista.total, POR_PAGINA);
			if (valida !== pagina) {
				pagina = valida;
				lista = await listarProveedores({ activo: filtro, pagina, porPagina: POR_PAGINA });
			}
			proveedores = lista.items;
			total = lista.total;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los proveedores.');
		} finally {
			cargando = false;
		}
	}

	async function cambiarFiltro(nuevo: FiltroActivo) {
		filtro = nuevo;
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

	function abrirEdicion(proveedor: Proveedor) {
		editando = proveedor;
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

	async function guardar(datos: CrearProveedorInput) {
		guardando = true;
		error = null;
		errorModal = null;
		try {
			if (editando) {
				await actualizarProveedor(editando.id, datos);
				aviso = 'Proveedor actualizado.';
			} else {
				await crearProveedor(datos);
				aviso = 'Proveedor creado.';
			}
			modalAbierto = false;
			await cargar();
		} catch (e) {
			errorModal = mensajeError(e, 'No se pudo guardar el proveedor.');
		} finally {
			guardando = false;
		}
	}

	async function alternarEstado(proveedor: Proveedor) {
		error = null;
		aviso = null;
		try {
			if (proveedor.activo) {
				await desactivarProveedor(proveedor.id);
				aviso = `${proveedor.nombre} quedó inactivo. No se borró ningún dato.`;
			} else {
				await reactivarProveedor(proveedor.id);
				aviso = `${proveedor.nombre} volvió a estar activo.`;
			}
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado del proveedor.');
		}
	}
</script>

<svelte:head>
	<title>Proveedores · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Proveedores"
		descripcion="Alta, edición y baja lógica. Los proveedores inactivos no se pueden usar en compras nuevas."
	>
		<Boton icono="mas" onclick={abrirAlta}>Nuevo proveedor</Boton>
	</EncabezadoPagina>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}

	<Tarjeta>
		<div class="filtros">
			<Campo
				etiqueta="Estado"
				nombre="filtro-estado"
				tipo="select"
				opciones={OPCIONES_ESTADO}
				bind:valor={filtro}
				onchange={(valor) => void cambiarFiltro(valor as FiltroActivo)}
			/>
		</div>
	</Tarjeta>

	{#if !cargando && proveedores.length === 0}
		<Tarjeta>
			<EstadoVacio
				icono="proveedores"
				titulo="Todavía no hay proveedores en este filtro"
				descripcion="Creá el primero para poder registrar compras."
			>
				<Boton icono="mas" onclick={abrirAlta}>Crear el primer proveedor</Boton>
			</EstadoVacio>
		</Tarjeta>
	{:else}
		<Tarjeta densa={true}>
			<ProveedorTabla
				{proveedores}
				{cargando}
				onEditar={abrirEdicion}
				onAlternarEstado={(proveedor) => void alternarEstado(proveedor)}
			/>
		</Tarjeta>
		<div class="pie">
			<Paginacion {pagina} {total} porPagina={POR_PAGINA} onCambiar={irAPagina} />
		</div>
	{/if}
</div>

{#if modalAbierto}
	<ProveedorForm
		proveedor={editando}
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
		min-width: 200px;
	}

	.pie {
		padding: 0 2px;
	}
</style>
