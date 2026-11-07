<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { ApiError } from '#lib/api/client';
	import { cambiarPassword, login, obtenerSsoConfig, urlInicioSso } from '#lib/api/auth';
	import { CambioPasswordSchema, LoginSchema } from '#lib/schemas/auth';
	import { refrescarSesion, sesion } from '#lib/stores/sesion';
	import { elegirPreferencia, iniciarTema, preferencia } from '#lib/tema/tema';
	import type { PreferenciaTema } from '#lib/tema/tema';
	type SsoConfig = Awaited<ReturnType<typeof obtenerSsoConfig>>;

	const opcionesTema: { valor: PreferenciaTema; etiqueta: string; titulo: string }[] = [
		{ valor: 'light', etiqueta: 'Light', titulo: 'Tema claro' },
		{ valor: 'dark', etiqueta: 'Dark', titulo: 'Tema oscuro' },
		{ valor: 'system', etiqueta: 'System', titulo: 'Seguir al sistema' }
	];

	let modo = $state<'login' | 'cambio'>('login');
	let email = $state('');
	let password = $state('');
	let passwordActual = $state('');
	let passwordNueva = $state('');
	let repetir = $state('');
	let error = $state<string | null>(null);
	let enviando = $state(false);
	let ssoConfig = $state<SsoConfig | null>(null);

	// Reaplica el tema al hidratar y escucha cambios del sistema mientras la preferencia sea
	// "system"; el script de app.html ya lo aplicó antes del primer pintado (FR-080).
	onMount(iniciarTema);

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

<div class="login-lienzo">
	<div class="login-panel">
		<div class="login-barra-tema">
			<span class="login-campo-etiqueta mb-0">Tema</span>
			<div class="login-tema" role="group" aria-label="Tema de la interfaz">
				{#each opcionesTema as opcion (opcion.valor)}
					<button
						type="button"
						class="login-tema-opcion"
						aria-pressed={$preferencia === opcion.valor}
						title={opcion.titulo}
						onclick={() => elegirPreferencia(opcion.valor)}
					>
						{#if opcion.valor === 'light'}
							<svg
								width="13"
								height="13"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.8"
								stroke-linecap="round"
								aria-hidden="true"
							>
								<circle cx="12" cy="12" r="4" />
								<path
									d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"
								/>
							</svg>
						{:else if opcion.valor === 'dark'}
							<svg
								width="13"
								height="13"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.8"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<path d="M20 14.5A8.5 8.5 0 0 1 9.5 4 7 7 0 1 0 20 14.5Z" />
							</svg>
						{:else}
							<svg
								width="13"
								height="13"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.8"
								stroke-linecap="round"
								aria-hidden="true"
							>
								<rect x="2.5" y="4" width="19" height="13" rx="2" />
								<path d="M8 21h8M12 17v4" />
							</svg>
						{/if}
						{opcion.etiqueta}
					</button>
				{/each}
			</div>
		</div>

		<div class="login-tarjeta">
			<div class="mb-4 flex items-center gap-3">
				<span class="login-marca" aria-hidden="true">
					<svg
						width="17"
						height="17"
						viewBox="0 0 16 16"
						fill="none"
						stroke="currentColor"
						stroke-width="1.3"
						stroke-linejoin="round"
					>
						<path d="M8 1.6 13.6 4.8v6.4L8 14.4 2.4 11.2V4.8L8 1.6Z" />
						<path d="M2.6 4.9 8 8l5.4-3.1M8 8v6.2" />
					</svg>
				</span>
				<div>
					<p class="login-kicker">Sistema de inventario</p>
					<h1 class="login-titulo">Stockmante</h1>
				</div>
			</div>

			{#if modo === 'login'}
				<p class="login-subtitulo">Iniciá sesión para continuar</p>

				<form
					class="flex flex-col gap-3.5"
					onsubmit={(e) => {
						e.preventDefault();
						void enviarLogin();
					}}
				>
					<label class="block">
						<span class="login-campo-etiqueta">Email</span>
						<input
							class="login-campo"
							type="email"
							autocomplete="username"
							bind:value={email}
							required
						/>
					</label>
					<label class="block">
						<span class="login-campo-etiqueta">Contraseña</span>
						<input
							class="login-campo"
							type="password"
							autocomplete="current-password"
							bind:value={password}
							required
						/>
					</label>

					{#if error}
						<p class="login-error" role="alert">{error}</p>
					{/if}

					<button
						type="submit"
						disabled={enviando}
						aria-busy={enviando}
						class="login-primario mt-1"
					>
						{enviando ? 'Ingresando…' : 'Ingresar'}
					</button>
				</form>

				<div class="mt-5 flex flex-col gap-2">
					{#if ssoConfig?.google_disponible}
						<a class="login-secundario" href={urlInicioSso('google')}>Continuar con Google</a>
					{/if}
					{#if ssoConfig?.microsoft_disponible}
						<a class="login-secundario" href={urlInicioSso('microsoft')}>
							Continuar con Microsoft
						</a>
					{/if}
					{#if ssoConfig === null || (!ssoConfig.google_disponible && !ssoConfig.microsoft_disponible)}
						<p class="login-sin-sso">Sin proveedores SSO configurados</p>
					{/if}
					<p class="login-pie mt-2">
						¿No tenés cuenta? <a class="login-enlace" href="/registro">Registrate</a>
					</p>
				</div>
			{:else}
				<p class="login-aviso mb-5">Tu contraseña es temporal: creá una nueva para continuar.</p>

				<form
					class="flex flex-col gap-3.5"
					onsubmit={(e) => {
						e.preventDefault();
						void enviarCambio();
					}}
				>
					<label class="block">
						<span class="login-campo-etiqueta">Contraseña actual (temporal)</span>
						<input
							class="login-campo"
							type="password"
							autocomplete="current-password"
							bind:value={passwordActual}
							required
						/>
					</label>
					<label class="block">
						<span class="login-campo-etiqueta">Contraseña nueva</span>
						<input
							class="login-campo"
							type="password"
							autocomplete="new-password"
							bind:value={passwordNueva}
							required
						/>
					</label>
					<p class="login-ayuda -mt-1.5">Mínimo 8 caracteres, 1 mayúscula y 1 dígito</p>
					<label class="block">
						<span class="login-campo-etiqueta">Repetir contraseña nueva</span>
						<input
							class="login-campo"
							type="password"
							autocomplete="new-password"
							bind:value={repetir}
							required
						/>
					</label>

					{#if error}
						<p class="login-error" role="alert">{error}</p>
					{/if}

					<button
						type="submit"
						disabled={enviando}
						aria-busy={enviando}
						class="login-primario mt-1"
					>
						{enviando ? 'Guardando…' : 'Guardar contraseña'}
					</button>
				</form>
			{/if}
		</div>
	</div>
</div>
