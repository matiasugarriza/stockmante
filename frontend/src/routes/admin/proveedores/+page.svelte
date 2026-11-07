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
	import { paginaValida, totalPaginas } from '#lib/utils/paginacion';

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
	const paginas = $derived(totalPaginas(total, POR_PAGINA));

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

<div class="flex flex-col gap-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h1 class="text-xl font-semibold">Proveedores</h1>
			<p class="text-sm text-slate-500">
				Alta, edición y baja lógica. Los proveedores inactivos no se pueden usar en compras nuevas.
			</p>
		</div>
		<button
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
			onclick={abrirAlta}
		>
			Nuevo proveedor
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

	<div class="flex items-center gap-2 text-sm">
		<label class="flex items-center gap-2">
			<span class="text-slate-500">Estado</span>
			<select
				class="rounded border border-slate-300 px-3 py-1.5"
				bind:value={filtro}
				onchange={() => void cambiarFiltro(filtro)}
			>
				<option value="all">Todos</option>
				<option value="true">Activos</option>
				<option value="false">Inactivos</option>
			</select>
		</label>
	</div>

	{#if cargando}
		<p class="py-6 text-center text-sm text-slate-500">Cargando proveedores...</p>
	{:else if proveedores.length === 0}
		<div class="rounded border border-dashed border-slate-300 bg-white px-6 py-10 text-center">
			<p class="text-sm font-medium text-slate-700">Todavía no hay proveedores en este filtro.</p>
			<p class="mt-1 text-sm text-slate-500">
				Creá el primero para poder registrar compras.
			</p>
			<button
				class="mt-4 rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
				onclick={abrirAlta}
			>
				Crear el primer proveedor
			</button>
		</div>
	{:else}
		<ProveedorTabla
			{proveedores}
			onEditar={abrirEdicion}
			onAlternarEstado={(proveedor) => void alternarEstado(proveedor)}
		/>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<p class="text-xs text-slate-500">
				{proveedores.length} de {total} proveedor{total === 1 ? '' : 'es'} · página {pagina} de
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
	<ProveedorForm
		proveedor={editando}
		{guardando}
		error={errorModal}
		onGuardar={(datos) => void guardar(datos)}
		onCerrar={cerrarModal}
	/>
{/if}