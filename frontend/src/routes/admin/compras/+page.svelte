<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '#lib/api/client';
	import { z } from 'zod';
	const Item = z.object({ id: z.number().int(), numero: z.string().nullable(), proveedor_nombre: z.string(), deposito_nombre: z.string(), estado: z.string(), total_costo: z.number(), creado_en: z.string() });
	const Lista = z.object({ items: z.array(Item), total: z.number().int(), pagina: z.number().int(), por_pagina: z.number().int() });
	let items = $state<z.infer<typeof Item>[]>([]);
	let error = $state('');
	async function cargar() { try { const b = await api<unknown>('/api/compras'); items = Lista.parse(b).items; } catch (e) { error = 'No se pudo cargar'; } }
	onMount(cargar);
</script>
<div class="p-4 space-y-4">
	<h1 class="text-2xl font-bold">Compras</h1>
	{#if error}<div class="text-red-700 text-sm">{error}</div>{/if}
	<table class="w-full text-sm border">
		<thead class="bg-gray-100"><tr><th class="p-2 text-left">ID</th><th class="p-2 text-left">Proveedor</th><th class="p-2 text-left">Depósito</th><th class="p-2 text-left">Estado</th><th class="p-2 text-left">Total</th></tr></thead>
		<tbody>{#each items as it}<tr class="border-t"><td class="p-2">{it.id}</td><td class="p-2">{it.proveedor_nombre}</td><td class="p-2">{it.deposito_nombre}</td><td class="p-2">{it.estado}</td><td class="p-2">{it.total_costo}</td></tr>{/each}</tbody>
	</table>
</div>
