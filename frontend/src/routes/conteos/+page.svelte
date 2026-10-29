<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import {
		cancelarConteo,
		cargarLineas,
		cerrarConteo,
		crearConteo,
		listarConteos,
		obtenerConteo
	} from '#lib/api/conteos';
	import { listarDepositos } from '#lib/api/depositos';
	import type { Conteo, ConteoLinea } from '#lib/schemas/conteos';
	import type { Deposito } from '#lib/schemas/depositos';

	const formatoFecha = new Intl.DateTimeFormat('es-AR', {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	let depositos = $state<Deposito[]>([]);
	let depositoId = $state('');
	let observacion = $state('');

	let conteos = $state<Conteo[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let filtroEstado = $state('');

	let conteoActivo = $state<Conteo | null>(null);
	let conteosCargando = $state(false);
	let error = $state('');
	let exito = $state('');

	async function cargarDepositos() {
		try {
			depositos = await listarDepositos('true');
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron cargar los depósitos';
		}
	}

	async function cargarConteos() {
		conteosCargando = true;
		try {
			const res = await listarConteos({
				estado: filtroEstado || undefined,
				pagina,
				porPagina
			});
			conteos = res.items;
			total = res.total;
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron cargar los conteos';
		} finally {
			conteosCargando = false;
		}
	}

	async function abrir() {
		error = '';
		exito = '';
		if (!depositoId) {
			error = 'Elegí un depósito';
			return;
		}
		try {
			const c = await crearConteo({
				deposito_id: Number(depositoId),
				observacion: observacion.trim() || null
			});
			conteoActivo = c;
			exito = `Conteo #${c.id} abierto con ${c.lineas.length} líneas.`;
			observacion = '';
			await cargarConteos();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo abrir el conteo';
		}
	}

	async function guardarLineas() {
		if (!conteoActivo) return;
		error = '';
		exito = '';
		try {
			const payload = {
				lineas: conteoActivo.lineas.map((l: ConteoLinea) => ({
					linea_id: l.id,
					cantidad_contada: l.cantidad_contada ?? 0
				}))
			};
			conteoActivo = await cargarLineas(conteoActivo.id, payload);
			exito = 'Cantidades guardadas.';
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron guardar las cantidades';
		}
	}

	async function cerrar() {
		if (!conteoActivo) return;
		error = '';
		exito = '';
		try {
			conteoActivo = await cerrarConteo(conteoActivo.id);
			exito = `Conteo #${conteoActivo.id} cerrado.`;
			await cargarConteos();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cerrar el conteo';
		}
	}

	async function cancelar() {
		if (!conteoActivo) return;
		error = '';
		exito = '';
		try {
			conteoActivo = await cancelarConteo(conteoActivo.id);
			exito = `Conteo #${conteoActivo.id} cancelado.`;
			await cargarConteos();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cancelar el conteo';
		}
	}

	async function verDetalle(id: number) {
		try {
			conteoActivo = await obtenerConteo(id);
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cargar el conteo';
		}
	}

	onMount(async () => {
		await cargarDepositos();
		await cargarConteos();
	});
</script>

<div class="p-4 space-y-6">
	<h1 class="text-2xl font-bold">Conteos de inventario</h1>

	{#if error}
		<div class="rounded bg-red-100 text-red-700 px-3 py-2 text-sm">{error}</div>
	{/if}
	{#if exito}
		<div class="rounded bg-green-100 text-green-700 px-3 py-2 text-sm">{exito}</div>
	{/if}

	<section class="border rounded p-4 space-y-3">
		<h2 class="font-semibold">Abrir conteo</h2>
		<div class="flex flex-wrap gap-3 items-end">
			<label class="flex flex-col gap-1">
				<span class="text-sm">Depósito</span>
				<select class="border rounded px-2 py-1" bind:value={depositoId}>
					<option value="">Seleccionar</option>
					{#each depositos as d}
						<option value={d.id}>{d.nombre}</option>
					{/each}
				</select>
			</label>
			<label class="flex flex-col gap-1 flex-1">
				<span class="text-sm">Observación (opcional)</span>
				<input class="border rounded px-2 py-1" bind:value={observacion} />
			</label>
			<button class="bg-blue-600 text-white rounded px-4 py-2" onclick={abrir}>
				Abrir conteo
			</button>
		</div>
	</section>

	{#if conteoActivo}
		<section class="border rounded p-4 space-y-3">
			<div class="flex items-center justify-between">
				<h2 class="font-semibold">
					Conteo #{conteoActivo.id} — {conteoActivo.deposito_nombre} ({conteoActivo.estado})
				</h2>
				<div class="space-x-2">
					{#if conteoActivo.estado === 'ABIERTO'}
						<button class="bg-green-600 text-white rounded px-3 py-1" onclick={guardarLineas}>
							Guardar cantidades
						</button>
						<button class="bg-blue-600 text-white rounded px-3 py-1" onclick={cerrar}>
							Cerrar
						</button>
						<button class="bg-red-600 text-white rounded px-3 py-1" onclick={cancelar}>
							Cancelar
						</button>
					{/if}
				</div>
			</div>
			<table class="w-full text-sm border">
				<thead class="bg-gray-100">
					<tr>
						<th class="p-2 text-left">Producto</th>
						<th class="p-2 text-left">Ubicación</th>
						<th class="p-2 text-right">Sistema</th>
						<th class="p-2 text-right">Contado</th>
						<th class="p-2 text-right">Diferencia</th>
					</tr>
				</thead>
				<tbody>
					{#each conteoActivo.lineas as linea}
						<tr class="border-t">
							<td class="p-2">{linea.producto_nombre} ({linea.sku})</td>
							<td class="p-2">{linea.ubicacion_nombre ?? 'Depósito'}</td>
							<td class="p-2 text-right">{linea.cantidad_sistema}</td>
							<td class="p-2 text-right">
								{#if conteoActivo.estado === 'ABIERTO'}
									<input
										class="border rounded px-1 py-0.5 w-20 text-right"
										type="number"
										min="0"
										value={linea.cantidad_contada ?? 0}
										oninput={(e) => {
											const v = Number((e.target as HTMLInputElement).value);
											linea.cantidad_contada = Number.isNaN(v) ? 0 : v;
										}}
									/>
								{:else}
									{linea.cantidad_contada ?? '-'}
								{/if}
							</td>
							<td class="p-2 text-right">{linea.diferencia ?? '-'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
			{#if conteoActivo.movimiento_ajuste_id}
				<p class="text-xs text-gray-500">
					Movimiento de ajuste generado: #{conteoActivo.movimiento_ajuste_id}
				</p>
			{/if}
		</section>
	{/if}

	<section class="space-y-3">
		<h2 class="font-semibold">Historial</h2>
		<div class="flex gap-3 items-center">
			<label class="text-sm">
				Estado:
				<select class="border rounded px-2 py-1" bind:value={filtroEstado} onchange={cargarConteos}>
					<option value="">Todos</option>
					<option value="ABIERTO">Abierto</option>
					<option value="CERRADO">Cerrado</option>
					<option value="CANCELADO">Cancelado</option>
				</select>
			</label>
		</div>
		{#if conteosCargando}
			<p class="text-sm text-gray-500">Cargando…</p>
		{/if}
		<table class="w-full text-sm border">
			<thead class="bg-gray-100">
				<tr>
					<th class="p-2 text-left">ID</th>
					<th class="p-2 text-left">Depósito</th>
					<th class="p-2 text-left">Estado</th>
					<th class="p-2 text-left">Autor</th>
					<th class="p-2 text-left">Fecha</th>
					<th class="p-2 text-left">Acción</th>
				</tr>
			</thead>
			<tbody>
				{#each conteos as c}
					<tr class="border-t">
						<td class="p-2">{c.id}</td>
						<td class="p-2">{c.deposito_nombre}</td>
						<td class="p-2">{c.estado}</td>
						<td class="p-2">{c.usuario.nombre}</td>
						<td class="p-2">{formatoFecha.format(new Date(c.creado_en))}</td>
						<td class="p-2">
							<button class="text-blue-600 underline" onclick={() => verDetalle(c.id)}>
								Ver
							</button>
						</td>
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
						cargarConteos();
					}}>Anterior</button
				>
				<button
					class="border rounded px-2 py-1"
					disabled={pagina * porPagina >= total}
					onclick={() => {
						pagina++;
						cargarConteos();
					}}>Siguiente</button
				>
			</div>
		</div>
	</section>
</div>
