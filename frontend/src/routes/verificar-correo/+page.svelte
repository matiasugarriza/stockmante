<script lang="ts">
	import { page } from '$app/state';
	import { ApiError } from '#lib/api/client';
	import { verificarCorreo } from '#lib/api/auth';

	let estado = $state<'verificando' | 'ok' | 'error' | 'sin_token'>('verificando');
	let mensaje = $state<string | null>(null);
	// El effect puede re-ejecutarse ante cambios de estado: solo una pasada.
	let intentado = $state(false);

	$effect(() => {
		if (intentado) return;
		intentado = true;
		const token = page.url.searchParams.get('token');
		if (!token) {
			estado = 'sin_token';
			mensaje = 'El enlace de verificación no incluye un token.';
			return;
		}
		verificarCorreo(token)
			.then(() => {
				estado = 'ok';
			})
			.catch((e: unknown) => {
				estado = 'error';
				mensaje =
					e instanceof ApiError ? e.message : 'No se pudo verificar la cuenta. Probá de nuevo.';
			});
	});
</script>

<svelte:head>
	<title>Verificar correo · Stockmante</title>
</svelte:head>

<div class="flex min-h-[80vh] items-center justify-center">
	<div class="w-full max-w-sm rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
		<h1 class="mb-2 text-xl font-semibold">Verificación de correo</h1>

		{#if estado === 'verificando'}
			<p class="text-sm text-slate-500">Verificando tu cuenta...</p>
		{:else if estado === 'ok'}
			<p class="mb-4 text-sm text-emerald-700">
				Cuenta verificada. Ya podés iniciar sesión con tu correo y contraseña.
			</p>
			<a
				class="block rounded bg-slate-800 px-4 py-2 text-center text-sm font-medium text-white hover:bg-slate-700"
				href="/login"
			>
				Iniciar sesión
			</a>
		{:else}
			<p class="mb-4 rounded bg-red-50 px-3 py-2 text-sm text-red-700">{mensaje}</p>
			<div class="flex gap-2">
				<a
					class="rounded border border-slate-300 px-4 py-2 text-sm hover:bg-slate-100"
					href="/registro"
				>
					Registrarme de nuevo
				</a>
				<a
					class="rounded border border-slate-300 px-4 py-2 text-sm hover:bg-slate-100"
					href="/login"
				>
					Ir al login
				</a>
			</div>
		{/if}
	</div>
</div>
