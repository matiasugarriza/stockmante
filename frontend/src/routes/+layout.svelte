<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { refrescarSesion, sesion } from '#lib/stores/sesion';
	import { listarAlertas } from '#lib/api/alertas';
	import { esRutaPublica } from '#lib/utils/rutas';
	import { iniciarTema } from '#lib/tema/tema';
	import { iniciarDispositivo } from '#lib/dispositivo/dispositivo';
	import { tituloDeRuta } from '#lib/navegacion/navegacion';
	import MarcoApp from '#lib/components/marco/MarcoApp.svelte';
	import LienzoAcceso from '#lib/components/marco/LienzoAcceso.svelte';
	import LogoSimbolo from '#lib/components/marco/LogoSimbolo.svelte';
	import AvisoFlotante from '#lib/components/ui/AvisoFlotante.svelte';
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	let alertasBajoMinimo = $state(0);

	const enAcceso = $derived(esRutaPublica(page.url.pathname));
	const titulo = $derived(tituloDeRuta(page.url.pathname));

	onMount(() => {
		const limpiarDispositivo = iniciarDispositivo();
		const limpiarTema = iniciarTema();
		void refrescarSesion();
		return () => {
			limpiarTema();
			limpiarDispositivo();
		};
	});

	// 011: el contador sale de /api/alertas (solo GESTION) y se refresca al montar y al
	// cambiar de ruta; sin polling.
	$effect(() => {
		void page.url.pathname;
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION') {
			void cargarAlertas();
		} else {
			alertasBajoMinimo = 0;
		}
	});

	async function cargarAlertas() {
		try {
			const res = await listarAlertas({ porPagina: 1 });
			alertasBajoMinimo = res.total;
		} catch {
			alertasBajoMinimo = 0;
		}
	}

	// Sin sesión → a /login (excepto en rutas públicas, p. ej. /registro: FR-070). Con
	// password temporal → cambio obligatorio.
	$effect(() => {
		const estado = $sesion;
		if (estado.estado === 'sin_sesion' && !enAcceso) {
			void goto('/login');
		} else if (estado.estado === 'activa' && estado.usuario.debe_cambiar_password && !enAcceso) {
			void goto('/login');
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if enAcceso}
	<LienzoAcceso>
		{@render children()}
	</LienzoAcceso>
{:else if $sesion.estado === 'activa'}
	<MarcoApp {titulo} alertasStock={alertasBajoMinimo}>
		{@render children()}
	</MarcoApp>
{:else if $sesion.estado === 'cargando'}
	<div class="cargando">
		<LogoSimbolo alto={64} />
		<p>Cargando…</p>
	</div>
{:else}
	<div class="cargando">
		<p>Redirigiendo al inicio de sesión…</p>
	</div>
{/if}

<AvisoFlotante />

<style>
	.cargando {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		min-height: 100dvh;
		background: var(--fondo);
		color: var(--texto-suave);
		font-size: 13px;
	}

</style>
