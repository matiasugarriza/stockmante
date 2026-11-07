<script lang="ts">
	import LogoSimbolo from '#lib/components/marco/LogoSimbolo.svelte';
	import { ApiError } from '#lib/api/client';
	import { registro } from '#lib/api/auth';
	import { RegistroSchema } from '#lib/schemas/auth';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	let nombre = $state('');
	let email = $state('');
	let password = $state('');
	let error = $state<string | null>(null);
	let exito = $state<string | null>(null);
	let enviando = $state(false);

	async function enviar() {
		error = null;
		exito = null;
		const datos = RegistroSchema.safeParse({ nombre, email, password });
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

<div class="acceso-panel">
	<Tarjeta>
		<div class="acceso-contenido">
			<div class="acceso-encabezado">
				<span class="acceso-marca"><LogoSimbolo alto={48} /></span>
				<div>
					<p class="acceso-kicker">Sistema de inventario</p>
					<h1 class="acceso-titulo">Creá tu cuenta</h1>
				</div>
			</div>

			<p class="acceso-subtitulo">Completá tus datos para empezar a operar.</p>

			<form
				class="acceso-form"
				onsubmit={(e) => {
					e.preventDefault();
					void enviar();
				}}
			>
				<Campo etiqueta="Nombre" nombre="nombre" bind:valor={nombre} requerido={true} />
				<Campo
					etiqueta="Email"
					nombre="email"
					tipo="email"
					autocomplete="email"
					bind:valor={email}
					requerido={true}
				/>
				<Campo
					etiqueta="Contraseña"
					nombre="password"
					tipo="password"
					autocomplete="new-password"
					ayuda="Mínimo 8 caracteres, 1 mayúscula y 1 dígito"
					bind:valor={password}
					requerido={true}
				/>

				{#if error}<Aviso variante="error">{error}</Aviso>{/if}
				{#if exito}<Aviso variante="exito">{exito}</Aviso>{/if}

				<Boton tipo="submit" cargando={enviando} anchoCompleto={true}>
					{enviando ? 'Registrando…' : 'Registrarse'}
				</Boton>
			</form>
		</div>
	</Tarjeta>

	<p class="acceso-pie">
		¿Ya tenés cuenta? <a class="acceso-enlace" href="/login">Ingresá</a>
	</p>
</div>
