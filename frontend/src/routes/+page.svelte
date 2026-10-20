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
	{/if}
	<p class="mt-4 text-sm text-slate-500">
		Módulo 1 en curso: usuarios, autenticación y catálogo base.
	</p>
</div>
