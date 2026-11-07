<script lang="ts">
	import LogoSimbolo from '#lib/components/marco/LogoSimbolo.svelte';
	import { page } from '$app/state';
	import { ApiError } from '#lib/api/client';
	import { verificarCorreo } from '#lib/api/auth';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Esqueleto from '#lib/components/ui/Esqueleto.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

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

<div class="acceso-panel">
	<Tarjeta>
		<div class="acceso-contenido">
			<div class="acceso-encabezado">
				<span class="acceso-marca"><LogoSimbolo alto={48} /></span>
				<div>
					<p class="acceso-kicker">Cuenta</p>
					<h1 class="acceso-titulo">Verificación de correo</h1>
				</div>
			</div>

			{#if estado === 'verificando'}
				<p class="acceso-subtitulo">Verificando tu cuenta…</p>
				<Esqueleto alto="40px" />
			{:else if estado === 'ok'}
				<Aviso variante="exito">
					Cuenta verificada. Ya podés iniciar sesión con tu correo y contraseña.
				</Aviso>
				<Boton href="/login" anchoCompleto={true}>Iniciar sesión</Boton>
			{:else}
				<Aviso variante="error">{mensaje}</Aviso>
				<div class="acceso-acciones">
					<Boton href="/registro" variante="secundario">Registrarme de nuevo</Boton>
					<Boton href="/login" variante="secundario">Ir al login</Boton>
				</div>
			{/if}
		</div>
	</Tarjeta>
</div>
