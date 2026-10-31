<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '#lib/api/client';
	import { z } from 'zod';
	const Item = z.object({ id: z.number().int(), nombre: z.string(), cuit: z.string().nullable(), email: z.string().nullable(), telefono: z.string().nullable(), direccion: z.string().nullable(), activo: z.boolean() });
	const Lista = z.object({ items: z.array(Item), total: z.number().int(), pagina: z.number().int(), por_pagina: z.number().int() });
	let items = $state<z.infer<typeof Item>[]>([]);
	let error = $state('');
	async function cargar() { try { const b = await api<unknown>('/api/proveedores'); items = Lista.parse(b).items; } catch (e) { error = 'No se pudo cargar'; } }
	onMount(cargar);
</script>
<div class="p-4 space-y-4">
	<h1 class="text-2xl font-bold">Proveedores</h1>
	{#if error}<div class="text-red-700 text-sm">{error}</div>{/if}
	<table class="w-full text-sm border">
		<thead class="bg-gray-100"><tr><th class="p-2 text-left">ID</th><th class="p-2 text-left">Nombre</th><th class="p-2 text-left">CUIT</th><th class="p-2 text-left">Email</th><th class="p-2 text-left">Activo</th></tr></thead>
		<tbody>{#each items as it}<tr class="border-t"><td class="p-2">{it.id}</td><td class="p-2">{it.nombre}</td><td class="p-2">{it.cuit ?? ''}</td><td class="p-2">{it.email ?? ''}</td><td class="p-2">{it.activo ? 'Sí' : 'No'}</td></tr>{/each}</tbody>
	</table>
</div>
