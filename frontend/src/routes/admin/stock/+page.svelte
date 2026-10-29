<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import { listarDepositos } from '#lib/api/depositos';
	import { listarStock } from '#lib/api/stock';
	import type { Deposito } from '#lib/schemas/depositos';
	import type { StockItem } from '#lib/schemas/movimientos';

	let depositos = $state<Deposito[]>([]);
	let depositoId = $state('');
	let q = $state('');
	let bajoMinimo = $state(false);
	let items = $state<StockItem[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let cargando = $state(false);
	let error = $state('');

	async function cargar() {
		cargando = true;
		error = '';
		try {
			const res = await listarStock({
				q: q || undefined,
				depositoId: depositoId ? Number(depositoId) : undefined,
				bajoMinimo,
				pagina,
				porPagina
			});
			items = res.items;
			total = res.total;
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cargar el stock';
		} finally {
			cargando = false;
		}
	}

	onMount(async () => {
		try {
			depositos = await listarDepositos('true');
		} catch {
			depositos = [];
		}
		await cargar();
	});
</script>

<div class="p-4 space-y-4">
	<h1 class="text-2xl font-bold">Stock</h1>

	{#if error}
		<div class="rounded bg-red-100 text-red-700 px-3 py-2 text-sm">{error}</div>
	{/if}

	<div class="flex flex-wrap gap-3 items-end">
		<label class="flex flex-col gap-1 text-sm">
			<span>Buscar</span>
			<input class="border rounded px-2 py-1" bind:value={q} onkeydown={(e) => e.key === 'Enter' && cargar()} />
		</label>
		<label class="flex flex-col gap-1 text-sm">
			<span>Depósito</span>
			<select class="border rounded px-2 py-1" bind:value={depositoId} onchange={cargar}>
				<option value="">Todos</option>
				{#each depositos as d}
					<option value={d.id}>{d.nombre}</option>
				{/each}
			</select>
		</label>
		<label class="flex items-center gap-2 text-sm">
			<input type="checkbox" bind:checked={bajoMinimo} onchange={cargar} />
			Bajo mínimo
		</label>
		<button class="bg-blue-600 text-white rounded px-4 py-2" onclick={cargar}>Buscar</button>
	</div>

	{#if cargando}
		<p class="text-sm text-gray-500">Cargando…</p>
	{/if}

	<table class="w-full text-sm border">
		<thead class="bg-gray-100">
			<tr>
				<th class="p-2 text-left">Producto</th>
				<th class="p-2 text-left">SKU</th>
				<th class="p-2 text-right">Stock</th>
				<th class="p-2 text-right">Mínimo</th>
				<th class="p-2 text-right">Bajo mínimo</th>
			</tr>
		</thead>
		<tbody>
			{#each items as it}
				<tr class="border-t">
					<td class="p-2">{it.nombre}</td>
					<td class="p-2">{it.sku}</td>
					<td class="p-2 text-right">{it.stock_actual}</td>
					<td class="p-2 text-right">{it.stock_minimo}</td>
					<td class="p-2 text-right">{it.bajo_minimo ? 'Sí' : 'No'}</td>
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
