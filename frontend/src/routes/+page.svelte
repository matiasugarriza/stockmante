<script lang="ts">
	import { sesion } from '#lib/stores/sesion';
</script>

<svelte:head>
	<title>Inicio · Stockmante</title>
</svelte:head>

<div class="mx-auto max-w-xl py-12 text-center">
	<h1 class="text-2xl font-semibold">Stockmante</h1>
	{#if $sesion.estado === 'activa'}
		<p class="mt-2 text-slate-600">
			Hola, <span class="font-medium">{$sesion.usuario.nombre}</span>.
		</p>
		<div class="mt-6 flex flex-wrap justify-center gap-3 text-sm">
			<a
				class="rounded bg-slate-800 px-4 py-2 font-medium text-white hover:bg-slate-700"
				href="/caja"
			>
				Ir a Caja
			</a>
			{#if $sesion.usuario.familia_rol === 'GESTION'}
				<a
					class="rounded border border-slate-300 px-4 py-2 font-medium text-slate-700 hover:bg-slate-100"
					href="/admin/productos"
				>
					Administrar catálogo
				</a>
			{/if}
		</div>
	{:else}
		<p class="mt-4 text-sm text-slate-500">Iniciá sesión para operar el inventario.</p>
	{/if}
</div>
