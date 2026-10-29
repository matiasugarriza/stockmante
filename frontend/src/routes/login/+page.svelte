<script lang="ts">
	import { goto } from '$app/navigation';
	import { ApiError } from '#lib/api/client';
	import { cambiarPassword, login } from '#lib/api/auth';
	import {
		CambioPasswordSchema,
		LoginSchema
	} from '#lib/schemas/auth';
	import { refrescarSesion, sesion } from '#lib/stores/sesion';

	let modo = $state<'login' | 'cambio'>('login');
	let email = $state('');
	let password = $state('');
	let passwordActual = $state('');
	let passwordNueva = $state('');
	let repetir = $state('');
	let error = $state<string | null>(null);
	let enviando = $state(false);

	// Si llegamos con sesión activa y password temporal, directo al cambio.
	$effect(() => {
		const estado = $sesion;
		if (estado.estado === 'activa' && estado.usuario.debe_cambiar_password) {
			modo = 'cambio';
		} else if (estado.estado === 'activa' && modo === 'login') {
			void goto('/');
		}
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

<div class="flex min-h-[80vh] items-center justify-center">
	<div class="w-full max-w-sm rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
		<h1 class="mb-1 text-xl font-semibold">Stockmante</h1>

		{#if modo === 'login'}
			<p class="mb-5 text-sm text-slate-500">Iniciá sesión para continuar</p>

			<form
				class="flex flex-col gap-3"
				onsubmit={(e) => {
					e.preventDefault();
					void enviarLogin();
				}}
			>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Email</span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						type="email"
						autocomplete="username"
						bind:value={email}
						required
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Contraseña</span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						type="password"
						autocomplete="current-password"
						bind:value={password}
						required
					/>
				</label>

				{#if error}
					<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
				{/if}

				<button
					type="submit"
					disabled={enviando}
					class="mt-1 rounded bg-slate-800 px-4 py-2 font-medium text-white hover:bg-slate-700 disabled:opacity-60"
				>
					{enviando ? 'Ingresando…' : 'Ingresar'}
				</button>
			</form>

			<div class="mt-4 flex flex-col gap-2">
				<a
					class="rounded border border-slate-300 px-4 py-2 text-center text-sm hover:bg-slate-100"
					href="/api/auth/sso/google/iniciar"
				>
					Continuar con Google
				</a>
				<a
					class="rounded border border-slate-300 px-4 py-2 text-center text-sm hover:bg-slate-100"
					href="/api/auth/sso/microsoft/iniciar"
				>
					Continuar con Microsoft
				</a>
				<p class="text-center text-sm text-slate-500">
					¿No tenés cuenta? <a class="text-blue-600 underline" href="/registro">Registrate</a>
				</p>
			</div>
		{:else}
			<p class="mb-5 text-sm text-amber-700">
				Tu contraseña es temporal: creá una nueva para continuar.
			</p>

			<form
				class="flex flex-col gap-3"
				onsubmit={(e) => {
					e.preventDefault();
					void enviarCambio();
				}}
			>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Contraseña actual (temporal)</span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						type="password"
						autocomplete="current-password"
						bind:value={passwordActual}
						required
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Contraseña nueva</span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						type="password"
						autocomplete="new-password"
						bind:value={passwordNueva}
						required
					/>
					<span class="text-xs text-slate-500">Mínimo 8 caracteres, 1 mayúscula y 1 dígito</span>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Repetir contraseña nueva</span>
					<input
						class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
						type="password"
						autocomplete="new-password"
						bind:value={repetir}
						required
					/>
				</label>

				{#if error}
					<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
				{/if}

				<button
					type="submit"
					disabled={enviando}
					class="mt-1 rounded bg-slate-800 px-4 py-2 font-medium text-white hover:bg-slate-700 disabled:opacity-60"
				>
					{enviando ? 'Guardando…' : 'Guardar contraseña'}
				</button>
			</form>
		{/if}
	</div>
</div>
