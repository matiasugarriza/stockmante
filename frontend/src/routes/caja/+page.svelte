<script lang="ts">
	import { onMount } from 'svelte';
	import { listarCategorias } from '#lib/api/categorias';
	import { ApiError } from '#lib/api/client';
	import { buscarProductos, type CampoBusqueda } from '#lib/api/productos';
	import type { Categoria } from '#lib/schemas/categorias';
	import type { Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';

	const CAMPOS: { valor: CampoBusqueda; etiqueta: string }[] = [
		{ valor: 'todos', etiqueta: 'Todo' },
		{ valor: 'sku', etiqueta: 'SKU exacto' },
		{ valor: 'codigo', etiqueta: 'Código exacto' },
		{ valor: 'nombre', etiqueta: 'Nombre' },
		{ valor: 'categoria', etiqueta: 'Categoría' }
	];

	const formatoMoneda = new Intl.NumberFormat('es-AR', {
		style: 'currency',
		currency: 'ARS'
	});

	let q = $state('');
	let campo = $state<CampoBusqueda>('todos');
	let categoriaId = $state('');
	let incluirInactivos = $state(false);
	let categorias = $state<Categoria[]>([]);
	let items = $state<Producto[]>([]);
	let buscando = $state(false);
	let consultado = $state(false);
	let error = $state<string | null>(null);
	let errorCategorias = $state<string | null>(null);
	let ms = $state<number | null>(null);

	let secuencia = 0;
	let temporizador: ReturnType<typeof setTimeout> | undefined;
	let buscador: HTMLInputElement | undefined;

	const esGestion = $derived(
		$sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'
	);
	const hayFiltro = $derived(q.trim().length > 0 || categoriaId !== '');
	const total = $derived(items.length);

	onMount(() => {
		buscador?.focus();
		void cargarCategorias();
	});

	// El checkbox solo existe para GESTION; si la sesión cambia, se apaga.
	$effect(() => {
		if (!esGestion && incluirInactivos) {
			incluirInactivos = false;
		}
	});

	// Debounce de 250 ms: una consulta por pausa de tipeo, descartando respuestas viejas.
	$effect(() => {
		const texto = q;
		const campoActual = campo;
		const categoria = categoriaId;
		const inactivos = incluirInactivos;
		const filtrando = hayFiltro;

		if (!filtrando) {
			items = [];
			consultado = false;
			ms = null;
			error = null;
			buscando = false;
			return;
		}

		const turno = ++secuencia;
		if (temporizador) clearTimeout(temporizador);
		buscando = true;
		temporizador = setTimeout(() => {
			void ejecutar(turno, texto, campoActual, categoria, inactivos);
		}, 250);

		return () => {
			if (temporizador) clearTimeout(temporizador);
		};
	});

	async function cargarCategorias() {
		try {
			categorias = await listarCategorias('all');
		} catch (e) {
			errorCategorias = mensajeError(e, 'No se pudieron cargar las categorías');
		}
	}

	async function ejecutar(
		turno: number,
		texto: string,
		campoActual: CampoBusqueda,
		categoria: string,
		inactivos: boolean
	) {
		const inicio = performance.now();
		try {
			const resultado = await buscarProductos({
				q: texto,
				campo: campoActual,
				categoriaId: categoria ? Number(categoria) : undefined,
				incluirInactivos: inactivos,
				limite: 20
			});
			if (turno !== secuencia) return;
			items = resultado.items;
			consultado = true;
			error = null;
			ms = Math.round(performance.now() - inicio);
		} catch (e) {
			if (turno !== secuencia) return;
			items = [];
			consultado = true;
			ms = null;
			error = mensajeError(e, 'No se pudo realizar la búsqueda');
		} finally {
			if (turno === secuencia) buscando = false;
		}
	}

	function mensajeError(e: unknown, fallback: string): string {
		if (e instanceof ApiError) return e.message;
		if (e instanceof Error) return e.message;
		return fallback;
	}

	function limpiar() {
		q = '';
		campo = 'todos';
		categoriaId = '';
		incluirInactivos = false;
		error = null;
	}
</script>

<svelte:head>
	<title>Caja · Stockmante</title>
</svelte:head>

<div class="flex flex-col gap-4">
	<div>
		<h1 class="text-xl font-semibold">Caja</h1>
		<p class="text-sm text-slate-500">
			Buscá por SKU, código de barras, nombre o categoría. Los productos inactivos no se
			muestran salvo que los incluyas como gestión.
		</p>
	</div>

	<div class="rounded border border-slate-200 bg-white p-3">
		<div class="flex flex-col gap-3 md:flex-row md:items-end">
			<label class="flex-1 text-sm">
				<span class="mb-1 block font-medium text-slate-600">Buscar</span>
				<input
					type="search"
					bind:value={q}
					bind:this={buscador}
					placeholder="SKU, código, nombre o categoría"
					class="w-full rounded border border-slate-300 px-3 py-2 text-base focus:border-slate-500 focus:outline-none"
				/>
			</label>

			<label class="text-sm md:w-44">
				<span class="mb-1 block font-medium text-slate-600">Campo</span>
				<select
					bind:value={campo}
					class="w-full rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
				>
					{#each CAMPOS as opcion (opcion.valor)}
						<option value={opcion.valor}>{opcion.etiqueta}</option>
					{/each}
				</select>
			</label>

			<label class="text-sm md:w-56">
				<span class="mb-1 block font-medium text-slate-600">Categoría</span>
				<select
					bind:value={categoriaId}
					class="w-full rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
				>
					<option value="">Todas</option>
					{#each categorias as categoria (categoria.id)}
						<option value={String(categoria.id)}>
							{categoria.nombre}{categoria.activo ? '' : ' (inactiva)'}
						</option>
					{/each}
				</select>
			</label>

			<div class="flex items-center gap-2 md:pb-2">
				{#if esGestion}
					<label class="flex items-center gap-2 text-sm text-slate-600">
						<input type="checkbox" bind:checked={incluirInactivos} class="h-4 w-4" />
						Incluir inactivos
					</label>
				{/if}
				{#if hayFiltro}
					<button
						class="rounded border border-slate-300 px-3 py-1 text-sm text-slate-600 hover:bg-slate-100"
						onclick={limpiar}
					>
						Limpiar
					</button>
				{/if}
			</div>
		</div>

		{#if errorCategorias}
			<p class="mt-3 rounded border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-700">
				{errorCategorias}
			</p>
		{/if}
	</div>

	{#if error}
		<p class="rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
	{/if}

	<div aria-live="polite" class="min-h-5 text-sm text-slate-500">
		{#if buscando}
			Buscando…
		{:else if consultado}
			{total} resultado{total === 1 ? '' : 's'}{#if ms !== null}
				· {ms} ms{/if}
		{:else if !hayFiltro}
			Escribí al menos 2 caracteres o elegí una categoría.
		{/if}
	</div>

	{#if consultado && !buscando && items.length === 0 && !error}
		<p class="rounded border border-slate-200 bg-white px-3 py-6 text-center text-sm text-slate-500">
			Sin resultados. Probá con otro término o revisá si el producto está inactivo.
		</p>
	{:else if items.length > 0}
		<div class="overflow-x-auto rounded border border-slate-200 bg-white">
			<table class="min-w-full text-sm">
				<thead class="bg-slate-100 text-left text-xs uppercase tracking-wide text-slate-500">
					<tr>
						<th class="px-3 py-2">Producto</th>
						<th class="px-3 py-2">SKU</th>
						<th class="px-3 py-2">Código</th>
						<th class="px-3 py-2">Categoría</th>
						<th class="px-3 py-2">Unidad</th>
						<th class="px-3 py-2 text-right">Precio</th>
						<th class="px-3 py-2 text-right">Stock mín.</th>
						<th class="px-3 py-2">Estado</th>
					</tr>
				</thead>
				<tbody>
					{#each items as producto (producto.id)}
						<tr class="border-t border-slate-100">
							<td class="px-3 py-2 font-medium">{producto.nombre}</td>
							<td class="px-3 py-2 font-mono text-xs">{producto.sku}</td>
							<td class="px-3 py-2 font-mono text-xs">{producto.codigo_barras ?? '—'}</td>
							<td class="px-3 py-2">{producto.categoria.nombre}</td>
							<td class="px-3 py-2">{producto.unidad_medida}</td>
							<td class="px-3 py-2 text-right tabular-nums">
								{formatoMoneda.format(producto.precio_venta)}
							</td>
							<td class="px-3 py-2 text-right tabular-nums">{producto.stock_minimo}</td>
							<td class="px-3 py-2">
								<span
									class="rounded px-1.5 py-0.5 text-xs {producto.activo
										? 'bg-emerald-50 text-emerald-700'
										: 'bg-slate-100 text-slate-500'}"
								>
									{producto.activo ? 'Activo' : 'Inactivo'}
								</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
