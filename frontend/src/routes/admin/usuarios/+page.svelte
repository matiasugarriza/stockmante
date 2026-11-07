<script lang="ts">
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
	import CredencialesModal from '#lib/components/usuarios/CredencialesModal.svelte';
	import PasswordTemporalModal from '#lib/components/usuarios/PasswordTemporalModal.svelte';
	import UsuarioForm from '#lib/components/usuarios/UsuarioForm.svelte';
	import UsuarioTabla from '#lib/components/usuarios/UsuarioTabla.svelte';
	import type { Rol, Usuario } from '#lib/schemas/usuarios';
	import { sesion } from '#lib/stores/sesion';
	import { paginaValida } from '#lib/utils/paginacion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	const POR_PAGINA = 20;

	let usuarios = $state<Usuario[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	let filtro = $state<FiltroActivo>('true');
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let errorModal = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let modalAbierto = $state(false);
	let editando = $state<Usuario | null>(null);
	// FR-008b: credenciales en pantalla tras alta/regenerado (modal obligatoria).
	let credenciales = $state<{ titulo: string; email: string; password: string } | null>(null);
	let regenerando = $state<Usuario | null>(null);
	let regenerandoGuardando = $state(false);
	let regenerandoError = $state<string | null>(null);

	const filtros: { valor: FiltroActivo; etiqueta: string }[] = [
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' },
		{ valor: 'all', etiqueta: 'Todos' }
	];

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
			let lista = await listarUsuarios({
				activo: filtro,
				pagina,
				porPagina: POR_PAGINA
			});
			// Una baja o un cambio de filtro puede vaciar la página actual:
			// volvemos a la última página válida antes de pintar.
			const valida = paginaValida(pagina, lista.total, POR_PAGINA);
			if (valida !== pagina) {
				pagina = valida;
				lista = await listarUsuarios({
					activo: filtro,
					pagina,
					porPagina: POR_PAGINA
				});
			}
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

	async function guardar(datos: {
		nombre: string;
		email: string;
		rol: Rol;
		password?: string;
	}) {
		guardando = true;
		error = null;
		errorModal = null;
		try {
			if (editando) {
				await actualizarUsuario(editando.id, datos);
				aviso = 'Usuario actualizado.';
				modalAbierto = false;
			} else {
				const creado = await crearUsuario(datos);
				modalAbierto = false;
				// Modal obligatoria con correo + clave (FR-008b).
				credenciales = {
					titulo: 'Usuario creado',
					email: creado.usuario.email,
					password: creado.password_temporal
				};
			}
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

	/** Abre el diálogo de regeneración (escenario 5): elige método y confirma. */
	function regenerar(usuario: Usuario) {
		error = null;
		aviso = null;
		regenerandoError = null;
		regenerando = usuario;
	}

	/** Pide la clave nueva al backend y muestra la modal con las credenciales. */
	async function generar(password?: string) {
		if (!regenerando) return;
		regenerandoGuardando = true;
		regenerandoError = null;
		try {
			const resultado = await regenerarPasswordTemporal(regenerando.id, password);
			credenciales = {
				titulo: 'Contraseña regenerada',
				email: regenerando.email,
				password: resultado.password_temporal
			};
			regenerando = null;
		} catch (e) {
			regenerandoError = mensajeError(e, 'No se pudo regenerar la contraseña temporal');
		} finally {
			regenerandoGuardando = false;
		}
	}
</script>

<svelte:head>
	<title>Usuarios · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Usuarios"
		descripcion="Alta con contraseña temporal, roles y baja lógica sin borrar historial."
	>
		<Boton icono="mas" onclick={abrirAlta}>Nuevo usuario</Boton>
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
		<UsuarioTabla
			{usuarios}
			{cargando}
			onEditar={abrirEdicion}
			onAlternarEstado={(usuario) => void alternarEstado(usuario)}
			onRegenerar={(usuario) => void regenerar(usuario)}
		/>
	</Tarjeta>

	<div class="pie">
		<Paginacion {pagina} {total} porPagina={POR_PAGINA} onCambiar={irAPagina} />
	</div>
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

{#if regenerando}
	<PasswordTemporalModal
		usuario={regenerando}
		guardando={regenerandoGuardando}
		error={regenerandoError}
		onGenerar={(password) => void generar(password)}
		onCerrar={() => (regenerando = null)}
	/>
{/if}

{#if credenciales}
	<CredencialesModal
		titulo={credenciales.titulo}
		email={credenciales.email}
		password={credenciales.password}
		onCerrar={() => (credenciales = null)}
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

	.pie {
		padding: 0 2px;
	}
</style>
