<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { cerrarSesion, refrescarSesion, sesion } from '#lib/stores/sesion';
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	onMount(() => {
		void refrescarSesion();
	});

	// Sin sesión → a /login (excepto la propia /login). Con password temporal → cambio obligatorio.
	$effect(() => {
		const estado = $sesion;
		const enLogin = page.url.pathname === '/login';
		if (estado.estado === 'sin_sesion' && !enLogin) {
			void goto('/login');
		} else if (estado.estado === 'activa' && estado.usuario.debe_cambiar_password && !enLogin) {
			void goto('/login');
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="min-h-screen bg-slate-50 text-slate-800">
	{#if $sesion.estado === 'activa' && page.url.pathname !== '/login'}
		<header class="flex items-center justify-between border-b border-slate-200 bg-white px-4 py-2">
			<span class="text-sm font-semibold">Stockmante</span>
			<div class="flex items-center gap-3 text-sm">
				<a class="rounded px-2 py-1 hover:bg-slate-100" href="/caja">Caja</a>
				<a class="rounded px-2 py-1 hover:bg-slate-100" href="/movimientos">Movimientos</a>
				{#if $sesion.usuario.familia_rol === 'GESTION'}
					<a class="rounded px-2 py-1 hover:bg-slate-100" href="/admin/usuarios">Usuarios</a>
					<a class="rounded px-2 py-1 hover:bg-slate-100" href="/admin/categorias">Categorías</a>
					<a class="rounded px-2 py-1 hover:bg-slate-100" href="/admin/productos">Productos</a>
					<a class="rounded px-2 py-1 hover:bg-slate-100" href="/admin/depositos">Depósitos</a>
				{/if}
				<span class="text-slate-600">
					{$sesion.usuario.nombre}
					<span class="ml-1 rounded bg-slate-100 px-1.5 py-0.5 text-xs text-slate-500">
						{$sesion.usuario.rol}
					</span>
				</span>
				<button
					class="rounded border border-slate-300 px-2 py-1 hover:bg-slate-100"
					onclick={() => void cerrarSesion()}
				>
					Cerrar sesión
				</button>
			</div>
		</header>
	{/if}

	<main class="mx-auto max-w-5xl p-4">
		{@render children()}
	</main>
</div>
