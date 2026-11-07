<script lang="ts">
	import LogoSimbolo from '#lib/components/marco/LogoSimbolo.svelte';
	import { goto } from '$app/navigation';
	import { ApiError } from '#lib/api/client';
	import { cambiarPassword, login, obtenerSsoConfig, urlInicioSso } from '#lib/api/auth';
	import { CambioPasswordSchema, LoginSchema } from '#lib/schemas/auth';
	import { refrescarSesion, sesion } from '#lib/stores/sesion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	type SsoConfig = Awaited<ReturnType<typeof obtenerSsoConfig>>;

	let modo = $state<'login' | 'cambio'>('login');
	let email = $state('');
	let password = $state('');
	let passwordActual = $state('');
	let passwordNueva = $state('');
	let repetir = $state('');
	let error = $state<string | null>(null);
	let enviando = $state(false);
	let ssoConfig = $state<SsoConfig | null>(null);

	// Si llegamos con sesión activa y password temporal, directo al cambio.
	$effect(() => {
		const estado = $sesion;
		if (estado.estado === 'activa' && estado.usuario.debe_cambiar_password) {
			modo = 'cambio';
		} else if (estado.estado === 'activa' && modo === 'login') {
			void goto('/');
		}
	});

	// Cargar disponibilidad de SSO para mostrar/ocultar botones.
	$effect(() => {
		obtenerSsoConfig()
			.then((cfg) => {
				ssoConfig = cfg;
			})
			.catch(() => {
				// Si falla, ocultamos SSO por seguridad
				ssoConfig = { google_disponible: false, microsoft_disponible: false };
			});
	});

	function mensajeError(e: unknown, fallback: string): string {
		if (e instanceof ApiError) return e.message;
		if (e instanceof Error) return e.message;
		return fallback;
	}

	async function enviarLogin() {
		error = null;
		const datos = LoginSchema.safeParse({ email, password });
		if (!datos.success) {
			error = 'Ingresá un email válido y tu contraseña';
			return;
		}
		enviando = true;
		try {
			await login(datos.data);
			await refrescarSesion();
			const estado = $sesion;
			if (estado.estado === 'activa' && estado.usuario.debe_cambiar_password) {
				modo = 'cambio';
				error = null;
			} else if (estado.estado === 'activa') {
				await goto('/');
			} else {
				error = 'Credenciales inválidas';
			}
		} catch (e) {
			error = mensajeError(e, 'No se pudo iniciar sesión');
		} finally {
			enviando = false;
		}
	}

	async function enviarCambio() {
		error = null;
		if (passwordNueva !== repetir) {
			error = 'Las contraseñas no coinciden';
			return;
		}
		const datos = CambioPasswordSchema.safeParse({
			password_actual: passwordActual,
			password_nueva: passwordNueva
		});
		if (!datos.success) {
			const primero = datos.error.issues[0];
			error = primero ? `${primero.message}` : 'La contraseña no cumple la política';
			return;
		}
		enviando = true;
		try {
			await cambiarPassword(datos.data);
			await refrescarSesion();
			await goto('/');
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar la contraseña');
		} finally {
			enviando = false;
		}
	}
</script>

<svelte:head>
	<title>{modo === 'login' ? 'Iniciar sesión' : 'Cambio de contraseña'} · Stockmante</title>
</svelte:head>

<div class="acceso-panel">
	<Tarjeta>
		<div class="acceso-contenido">
			<div class="acceso-encabezado">
				<span class="acceso-marca"><LogoSimbolo alto={48} /></span>
				<div>
					<p class="acceso-kicker">Sistema de inventario</p>
					<h1 class="acceso-titulo">{modo === 'login' ? 'Iniciar sesión' : 'Cambiar contraseña'}</h1>
				</div>
			</div>

			{#if modo === 'login'}
				<p class="acceso-subtitulo">Ingresá tus credenciales para continuar</p>

				<form
					class="acceso-form"
					onsubmit={(e) => {
						e.preventDefault();
						void enviarLogin();
					}}
				>
					<Campo
						etiqueta="Email"
						nombre="email"
						tipo="email"
						autocomplete="username"
						bind:valor={email}
						requerido={true}
					/>
					<Campo
						etiqueta="Contraseña"
						nombre="password"
						tipo="password"
						autocomplete="current-password"
						bind:valor={password}
						requerido={true}
					/>

					{#if error}<Aviso variante="error">{error}</Aviso>{/if}

					<Boton tipo="submit" cargando={enviando} anchoCompleto={true}>
						{enviando ? 'Ingresando…' : 'Ingresar'}
					</Boton>
				</form>

				<div class="acceso-secundarios">
					{#if ssoConfig?.google_disponible}
						<Boton href={urlInicioSso('google')} variante="secundario" anchoCompleto={true}>
							Continuar con Google
						</Boton>
					{/if}
					{#if ssoConfig?.microsoft_disponible}
						<Boton href={urlInicioSso('microsoft')} variante="secundario" anchoCompleto={true}>
							Continuar con Microsoft
						</Boton>
					{/if}
					{#if ssoConfig === null || (!ssoConfig.google_disponible && !ssoConfig.microsoft_disponible)}
						<p class="acceso-sin-sso">Sin proveedores SSO configurados</p>
					{/if}
				</div>
			{:else}
				<p class="acceso-subtitulo">Definí una contraseña nueva para continuar</p>
				<Aviso variante="advertencia">
					Tu contraseña es temporal: creá una nueva para continuar.
				</Aviso>

				<form
					class="acceso-form"
					onsubmit={(e) => {
						e.preventDefault();
						void enviarCambio();
					}}
				>
					<Campo
						etiqueta="Contraseña actual (temporal)"
						nombre="password_actual"
						tipo="password"
						autocomplete="current-password"
						bind:valor={passwordActual}
						requerido={true}
					/>
					<Campo
						etiqueta="Contraseña nueva"
						nombre="password_nueva"
						tipo="password"
						autocomplete="new-password"
						ayuda="Mínimo 8 caracteres, 1 mayúscula y 1 dígito"
						bind:valor={passwordNueva}
						requerido={true}
					/>
					<Campo
						etiqueta="Repetir contraseña nueva"
						nombre="repetir"
						tipo="password"
						autocomplete="new-password"
						bind:valor={repetir}
						requerido={true}
					/>

					{#if error}<Aviso variante="error">{error}</Aviso>{/if}

					<Boton tipo="submit" cargando={enviando} anchoCompleto={true}>
						{enviando ? 'Guardando…' : 'Guardar contraseña'}
					</Boton>
				</form>
			{/if}
		</div>
	</Tarjeta>

	<p class="acceso-pie">
		{#if modo === 'login'}
			¿No tenés cuenta? <a class="acceso-enlace" href="/registro">Registrate</a>
		{:else}
			<a class="acceso-enlace" href="/login">Volver al inicio de sesión</a>
		{/if}
	</p>
</div>
