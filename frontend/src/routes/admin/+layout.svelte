<script lang="ts">
	import { goto } from '$app/navigation';
	import type { Snippet } from 'svelte';
	import { sesion } from '#lib/stores/sesion';

	let { children }: { children: Snippet } = $props();

	// El backend ya deniega con 403; este guard evita que OPERACION vea
	// formularios de gestión (UX, no autoridad).
	$effect(() => {
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol !== 'GESTION') {
			void goto('/caja');
		}
	});
</script>

{#if $sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'}
	{@render children()}
{:else}
	<p class="verificando">Verificando permisos…</p>
{/if}

<style>
	.verificando {
		padding: 24px 16px;
		color: var(--texto-suave);
		font-size: 13px;
	}
</style>
