<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import { crearTransferencia, listarTransferencias } from '#lib/api/transferencias';
	import { listarDepositos, listarUbicaciones } from '#lib/api/depositos';
	import { buscarProductos } from '#lib/api/productos';
	import type { Deposito, Ubicacion } from '#lib/schemas/depositos';
	import type { Producto, ListaBusqueda } from '#lib/schemas/productos';
	import type { Transferencia } from '#lib/schemas/transferencias';
	import { sesion } from '#lib/stores/sesion';

	const formatoFecha = new Intl.DateTimeFormat('es-AR', {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	let depositos = $state<Deposito[]>([]);
	let ubicacionesOrigen = $state<Ubicacion[]>([]);
	let ubicacionesDestino = $state<Ubicacion[]>([]);

	let productoQuery = $state('');
	let productoResultados = $state<Producto[]>([]);
	let productoSel = $state<Producto | null>(null);

	let depositoOrigenId = $state('');
	let depositoDestinoId = $state('');
	let ubicacionOrigenId = $state('');
	let ubicacionDestinoId = $state('');
	let cantidad = $state('1');
	let observacion = $state('');

	let transferencias = $state<Transferencia[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;

	let cargando = $state(false);
	let error = $state('');
	let exito = $state('');

	async function cargarDepositos() {
		try {
			depositos = await listarDepositos('true');
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron cargar los depósitos';
		}
	}

	async function cargarUbicaciones() {
		if (depositoOrigenId) {
			try {
				ubicacionesOrigen = await listarUbicaciones(Number(depositoOrigenId));
			} catch {
				ubicacionesOrigen = [];
			}
		} else {
			ubicacionesOrigen = [];
		}
		if (depositoDestinoId) {
			try {
				ubicacionesDestino = await listarUbicaciones(Number(depositoDestinoId));
			} catch {
				ubicacionesDestino = [];
			}
		} else {
			ubicacionesDestino = [];
		}
	}

	async function buscarProducto() {
		if (productoQuery.trim().length < 2) {
			productoResultados = [];
			return;
		}
		try {
			const res: ListaBusqueda = await buscarProductos({ q: productoQuery.trim(), limite: 8 });
			productoResultados = res.items;
		} catch {
			productoResultados = [];
		}
	}

	async function cargarTransferencias() {
		cargando = true;
		try {
			const res = await listarTransferencias({ pagina, porPagina });
			transferencias = res.items;
			total = res.total;
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron cargar las transferencias';
		} finally {
			cargando = false;
		}
	}

	async function enviar() {
		error = '';
		exito = '';
		if (!productoSel) {
			error = 'Elegí un producto';
			return;
		}
		if (!depositoOrigenId || !depositoDestinoId) {
			error = 'Elegí origen y destino';
			return;
		}
		if (depositoOrigenId === depositoDestinoId) {
			error = 'El depósito de origen y el de destino deben ser distintos.';
			return;
		}
		const cant = Number(cantidad);
		if (!Number.isInteger(cant) || cant <= 0) {
			error = 'La cantidad debe ser mayor que cero';
			return;
		}
		cargando = true;
		try {
			const t = await crearTransferencia({
				producto_id: productoSel.id,
				deposito_origen_id: Number(depositoOrigenId),
				deposito_destino_id: Number(depositoDestinoId),
				cantidad: cant,
				ubicacion_origen_id: ubicacionOrigenId ? Number(ubicacionOrigenId) : null,
				ubicacion_destino_id: ubicacionDestinoId ? Number(ubicacionDestinoId) : null,
				observacion: observacion.trim() || null
			});
			exito = `Transferencia #${t.id}: ${t.cantidad} unidades. Stock origen ${t.stock_origen_resultante}, destino ${t.stock_destino_resultante}.`;
			// reset
			cantidad = '1';
			observacion = '';
			await cargarTransferencias();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo registrar la transferencia';
		} finally {
			cargando = false;
		}
	}

	onMount(async () => {
		await cargarDepositos();
		await cargarTransferencias();
	});
</script>

<div class="p-4 space-y-6">
	<h1 class="text-2xl font-bold">Transferencias entre depósitos</h1>

	{#if error}
		<div class="rounded bg-red-100 text-red-700 px-3 py-2 text-sm">{error}</div>
	{/if}
	{#if exito}
		<div class="rounded bg-green-100 text-green-700 px-3 py-2 text-sm">{exito}</div>
	{/if}

	<section class="border rounded p-4 space-y-3">
		<h2 class="font-semibold">Nueva transferencia</h2>

		<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
			<label class="flex flex-col gap-1">
				<span class="text-sm">Producto</span>
				<input
					class="border rounded px-2 py-1"
					bind:value={productoQuery}
					oninput={buscarProducto}
					placeholder="Buscar por nombre o SKU"
				/>
				{#if productoResultados.length}
					<ul class="border rounded max-h-40 overflow-auto">
						{#each productoResultados as p}
							<li>
								<button
									class="w-full text-left px-2 py-1 hover:bg-gray-100"
									onclick={() => {
										productoSel = p;
										productoQuery = `${p.nombre} (${p.sku})`;
										productoResultados = [];
									}}
								>
									{p.nombre} — {p.sku}
								</button>
							</li>
						{/each}
					</ul>
				{/if}
				{#if productoSel}
					<span class="text-xs text-gray-500">Seleccionado: {productoSel.nombre}</span>
				{/if}
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm">Cantidad</span>
				<input class="border rounded px-2 py-1" type="number" min="1" bind:value={cantidad} />
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm">Depósito origen</span>
				<select
					class="border rounded px-2 py-1"
					bind:value={depositoOrigenId}
					onchange={() => {
						ubicacionOrigenId = '';
						cargarUbicaciones();
					}}
				>
					<option value="">Seleccionar</option>
					{#each depositos as d}
						<option value={d.id}>{d.nombre}</option>
					{/each}
				</select>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm">Ubicación origen (opcional)</span>
				<select class="border rounded px-2 py-1" bind:value={ubicacionOrigenId}>
					<option value="">Sin ubicación</option>
					{#each ubicacionesOrigen as u}
						<option value={u.id}>{u.nombre}</option>
					{/each}
				</select>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm">Depósito destino</span>
				<select
					class="border rounded px-2 py-1"
					bind:value={depositoDestinoId}
					onchange={() => {
						ubicacionDestinoId = '';
						cargarUbicaciones();
					}}
				>
					<option value="">Seleccionar</option>
					{#each depositos as d}
						<option value={d.id}>{d.nombre}</option>
					{/each}
				</select>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm">Ubicación destino (opcional)</span>
				<select class="border rounded px-2 py-1" bind:value={ubicacionDestinoId}>
					<option value="">Sin ubicación</option>
					{#each ubicacionesDestino as u}
						<option value={u.id}>{u.nombre}</option>
					{/each}
				</select>
			</label>
		</div>

		<label class="flex flex-col gap-1">
			<span class="text-sm">Observación (opcional)</span>
			<textarea class="border rounded px-2 py-1" rows="2" bind:value={observacion}></textarea>
		</label>

		<button
			class="bg-blue-600 text-white rounded px-4 py-2 disabled:opacity-50"
			disabled={cargando}
			onclick={enviar}
		>
			Transferir
		</button>
	</section>

	<section class="space-y-3">
		<h2 class="font-semibold">Historial</h2>
		{#if cargando}
			<p class="text-sm text-gray-500">Cargando…</p>
		{/if}
		<table class="w-full text-sm border">
			<thead class="bg-gray-100">
				<tr>
					<th class="p-2 text-left">ID</th>
					<th class="p-2 text-left">Producto</th>
					<th class="p-2 text-left">Origen</th>
					<th class="p-2 text-left">Destino</th>
					<th class="p-2 text-right">Cantidad</th>
					<th class="p-2 text-right">Stock origen</th>
					<th class="p-2 text-right">Stock destino</th>
					<th class="p-2 text-left">Autor</th>
					<th class="p-2 text-left">Fecha</th>
				</tr>
			</thead>
			<tbody>
				{#each transferencias as t}
					<tr class="border-t">
						<td class="p-2">{t.id}</td>
						<td class="p-2">{t.producto_nombre} ({t.sku})</td>
						<td class="p-2">{t.deposito_origen.nombre}</td>
						<td class="p-2">{t.deposito_destino.nombre}</td>
						<td class="p-2 text-right">{t.cantidad}</td>
						<td class="p-2 text-right">{t.stock_origen_resultante}</td>
						<td class="p-2 text-right">{t.stock_destino_resultante}</td>
						<td class="p-2">{t.usuario.nombre}</td>
						<td class="p-2">{formatoFecha.format(new Date(t.creado_en))}</td>
					</tr>
				{/each}
			</tbody>
		</table>
		<div class="flex justify-between text-sm">
			<span>Total: {total}</span>
			<div class="space-x-2">
				<button
					class="border rounded px-2 py-1"
					disabled={pagina <= 1}
					onclick={() => {
						pagina--;
						cargarTransferencias();
					}}>Anterior</button
				>
				<button
					class="border rounded px-2 py-1"
					disabled={pagina * porPagina >= total}
					onclick={() => {
						pagina++;
						cargarTransferencias();
					}}>Siguiente</button
				>
			</div>
		</div>
	</section>
</div>
