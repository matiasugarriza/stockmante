<script lang="ts">
	import { goto } from '$app/navigation';
	import type { FiltroActivo } from '#lib/api/usuarios';
	import {
		actualizarUsuario,
		crearUsuario,
		desactivarUsuario,
		listarUsuarios,
		reactivarUsuario,
		regenerarPasswordTemporal
	} from '#lib/api/usuarios';
	import { ApiError } from '#lib/api/client';
	import UsuarioForm from '#lib/components/usuarios/UsuarioForm.svelte';
	import UsuarioTabla from '#lib/components/usuarios/UsuarioTabla.svelte';
	import type { Rol, Usuario } from '#lib/schemas/usuarios';
	import { sesion } from '#lib/stores/sesion';

	let usuarios = $state<Usuario[]>([]);
	let total = $state(0);
	let filtro = $state<FiltroActivo>('true');
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let errorModal = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let modalAbierto = $state(false);
	let editando = $state<Usuario | null>(null);

	$effect(() => {
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol !== 'GESTION') {
			void goto('/');
		}
	});

	// La sesión la resuelve el layout; recién ahí pedimos la lista.
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
			const lista = await listarUsuarios({ activo: filtro, porPagina: 100 });
			usuarios = lista.items;
			total = lista.total;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los usuarios');
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

	function abrirEdicion(usuario: Usuario) {
		editando = usuario;
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

	async function guardar(datos: { nombre: string; email: string; rol: Rol }) {
		guardando = true;
		error = null;
		errorModal = null;
		try {
			if (editando) {
				await actualizarUsuario(editando.id, datos);
				aviso = 'Usuario actualizado.';
			} else {
				await crearUsuario(datos);
				aviso = `Usuario creado. La contraseña temporal se envió a ${datos.email}.`;
			}
			modalAbierto = false;
			await cargar();
		} catch (e) {
			// El modal tapa el banner de la página: el error se muestra dentro.
			errorModal = mensajeError(e, 'No se pudo guardar el usuario');
		} finally {
			guardando = false;
		}
	}

	async function alternarEstado(usuario: Usuario) {
		error = null;
		aviso = null;
		try {
			if (usuario.activo) {
				await desactivarUsuario(usuario.id);
				aviso = `${usuario.nombre} quedó inactivo y sus sesiones fueron revocadas.`;
			} else {
				await reactivarUsuario(usuario.id);
				aviso = `${usuario.nombre} volvió a estar activo.`;
			}
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado del usuario');
		}
	}

	async function regenerar(usuario: Usuario) {
		error = null;
		aviso = null;
		try {
			await regenerarPasswordTemporal(usuario.id);
			aviso = `Se envió una nueva contraseña temporal a ${usuario.email}.`;
		} catch (e) {
			error = mensajeError(e, 'No se pudo regenerar la contraseña temporal');
		}
	}

	const filtros: { valor: FiltroActivo; etiqueta: string }[] = [
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' },
		{ valor: 'all', etiqueta: 'Todos' }
	];
</script>

<svelte:head>
	<title>Usuarios · Stockmante</title>
</svelte:head>

<div class="flex flex-col gap-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h1 class="text-xl font-semibold">Usuarios</h1>
			<p class="text-sm text-slate-500">
				Alta con contraseña temporal, roles y baja lógica sin borrar historial.
			</p>
		</div>
		<button
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
			onclick={abrirAlta}
		>
			Nuevo usuario
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
		<p class="py-6 text-center text-sm text-slate-500">Cargando usuarios…</p>
	{:else}
		<UsuarioTabla
			{usuarios}
			onEditar={abrirEdicion}
			onAlternarEstado={(usuario) => void alternarEstado(usuario)}
			onRegenerar={(usuario) => void regenerar(usuario)}
		/>
		<p class="text-xs text-slate-500">
			{usuarios.length} de {total} usuario{total === 1 ? '' : 's'} en este filtro.
		</p>
	{/if}
</div>

{#if modalAbierto}
	<UsuarioForm
		usuario={editando}
		{guardando}
		error={errorModal}
		onGuardar={(datos) => void guardar(datos)}
		onCerrar={cerrarModal}
	/>
{/if}
