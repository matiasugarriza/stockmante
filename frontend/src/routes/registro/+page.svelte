<script lang="ts">
	import { ApiError } from '#lib/api/client';
	import { registro } from '#lib/api/auth';
	import { z } from 'zod';

	let nombre = $state('');
	let email = $state('');
	let password = $state('');
	let error = $state<string | null>(null);
	let exito = $state<string | null>(null);
	let enviando = $state(false);

	const Schema = z.object({
		nombre: z.string().min(1, 'El nombre es obligatorio'),
		email: z.string().email('Email inválido'),
		password: z
			.string()
			.min(8, 'Mínimo 8 caracteres')
			.refine((v) => /[A-Z]/.test(v), 'Al menos una mayúscula')
			.refine((v) => /\d/.test(v), 'Al menos un dígito')
	});

	async function enviar() {
		error = null;
		exito = null;
		const datos = Schema.safeParse({ nombre, email, password });
		if (!datos.success) {
			error = datos.error.issues[0]?.message ?? 'Datos inválidos';
			return;
		}
		enviando = true;
		try {
			await registro(datos.data);
			exito = 'Te enviamos un correo para verificar tu cuenta.';
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo registrar';
		} finally {
			enviando = false;
		}
	}
</script>

<svelte:head>
	<title>Registro · Stockmante</title>
</svelte:head>

<div class="flex min-h-[80vh] items-center justify-center">
	<div class="w-full max-w-sm rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
		<h1 class="mb-1 text-xl font-semibold">Stockmante</h1>
		<p class="mb-5 text-sm text-slate-500">Creá tu cuenta</p>

		<form
			class="flex flex-col gap-3"
			onsubmit={(e) => {
				e.preventDefault();
				void enviar();
			}}
		>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Nombre</span>
				<input class="rounded border border-slate-300 px-3 py-2" bind:value={nombre} required />
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Email</span>
				<input
					class="rounded border border-slate-300 px-3 py-2"
					type="email"
					bind:value={email}
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Contraseña</span>
				<input
					class="rounded border border-slate-300 px-3 py-2"
					type="password"
					bind:value={password}
					required
				/>
				<span class="text-xs text-slate-500">Mínimo 8 caracteres, 1 mayúscula y 1 dígito</span>
			</label>

			{#if error}
				<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
			{/if}
			{#if exito}
				<p class="rounded bg-green-50 px-3 py-2 text-sm text-green-700">{exito}</p>
			{/if}

			<button
				type="submit"
				disabled={enviando}
				class="mt-1 rounded bg-slate-800 px-4 py-2 font-medium text-white hover:bg-slate-700 disabled:opacity-60"
			>
				{enviando ? 'Registrando…' : 'Registrarse'}
			</button>
		</form>

		<p class="mt-4 text-center text-sm text-slate-500">
			¿Ya tenés cuenta? <a class="text-blue-600 underline" href="/login">Ingresá</a>
		</p>
	</div>
</div>
