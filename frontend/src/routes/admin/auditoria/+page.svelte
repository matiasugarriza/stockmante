<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import { api } from '#lib/api/client';
	import { z } from 'zod';

	const ItemSchema = z.object({
		id: z.number().int(),
		tabla_ref: z.string(),
		registro_id: z.number().int(),
		accion: z.string(),
		usuario_id: z.number().int(),
		usuario_nombre: z.string(),
		cuando: z.string()
	});
	const ListaSchema = z.object({
		items: z.array(ItemSchema),
		total: z.number().int(),
		pagina: z.number().int(),
		por_pagina: z.number().int()
	});

	let items = $state<z.infer<typeof ItemSchema>[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let tablaRef = $state('');
	let usuarioId = $state('');
	let cargando = $state(false);
	let error = $state('');

	async function cargar() {
		cargando = true;
		error = '';
		try {
			const query = new URLSearchParams();
			if (tablaRef) query.set('tabla_ref', tablaRef);
			if (usuarioId) query.set('usuario_id', usuarioId);
			query.set('pagina', String(pagina));
			query.set('por_pagina', String(porPagina));
			const body = await api<unknown>(`/api/auditoria?${query.toString()}`);
			const res = ListaSchema.parse(body);
			items = res.items;
			total = res.total;
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cargar la auditoría';
		} finally {
			cargando = false;
		}
	}

	onMount(cargar);
</script>

<div class="p-4 space-y-4">
	<h1 class="text-2xl font-bold">Auditoría</h1>
	{#if error}
		<div class="rounded bg-red-100 text-red-700 px-3 py-2 text-sm">{error}</div>
	{/if}
	<div class="flex gap-3 items-end">
		<label class="flex flex-col gap-1 text-sm">
			<span>Tabla</span>
			<input class="border rounded px-2 py-1" bind:value={tablaRef} />
		</label>
		<label class="flex flex-col gap-1 text-sm">
			<span>Usuario ID</span>
			<input class="border rounded px-2 py-1" bind:value={usuarioId} />
		</label>
		<button class="bg-blue-600 text-white rounded px-4 py-2" onclick={cargar}>Filtrar</button>
	</div>
	{#if cargando}
		<p class="text-sm text-gray-500">Cargando…</p>
	{/if}
	<table class="w-full text-sm border">
		<thead class="bg-gray-100">
			<tr>
				<th class="p-2 text-left">ID</th>
				<th class="p-2 text-left">Tabla</th>
				<th class="p-2 text-left">Registro</th>
				<th class="p-2 text-left">Acción</th>
				<th class="p-2 text-left">Usuario</th>
				<th class="p-2 text-left">Cuándo</th>
			</tr>
		</thead>
		<tbody>
			{#each items as it}
				<tr class="border-t">
					<td class="p-2">{it.id}</td>
					<td class="p-2">{it.tabla_ref}</td>
					<td class="p-2">{it.registro_id}</td>
					<td class="p-2">{it.accion}</td>
					<td class="p-2">{it.usuario_nombre}</td>
					<td class="p-2">{new Date(it.cuando).toLocaleString()}</td>
				</tr>
			{/each}
		</tbody>
	</table>
	<div class="flex justify-between text-sm">
		<span>Total: {total}</span>
		<div class="space-x-2">
			<button class="border rounded px-2 py-1" disabled={pagina <= 1} onclick={() => { pagina--; cargar(); }}>Anterior</button>
			<button class="border rounded px-2 py-1" disabled={pagina * porPagina >= total} onclick={() => { pagina++; cargar(); }}>Siguiente</button>
		</div>
	</div>
</div>
