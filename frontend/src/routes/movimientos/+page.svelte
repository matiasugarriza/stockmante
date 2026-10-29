<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import { crearMovimiento, listarMovimientos } from '#lib/api/movimientos';
	import { listarDepositos, listarUbicaciones } from '#lib/api/depositos';
	import { buscarProductos } from '#lib/api/productos';
	import {
		TipoMovimientoSchema,
		type LineaNuevaInput,
		type MovimientoDetalle,
		type MotivoMovimiento,
		type TipoMovimiento
	} from '#lib/schemas/movimientos';
	import { MotivoMovimientoSchema } from '#lib/schemas/movimientos';
	import type { Deposito, Ubicacion } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';

	const TIPOS: { valor: TipoMovimiento; etiqueta: string }[] = [
		{ valor: 'ENTRADA', etiqueta: 'Entrada' },
		{ valor: 'SALIDA', etiqueta: 'Salida' },
		{ valor: 'MERMA', etiqueta: 'Merma' },
		{ valor: 'AJUSTE', etiqueta: 'Ajuste' }
	];

	const MOTIVOS: { valor: MotivoMovimiento; etiqueta: string }[] =
		MotivoMovimientoSchema.options.map((valor) => ({
			valor,
			etiqueta: valor.charAt(0) + valor.slice(1).toLowerCase()
		}));

	const formatoFecha = new Intl.DateTimeFormat('es-AR', {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	interface LineaForm {
		producto: Producto | null;
		busqueda: string;
		resultados: Producto[];
		cantidad: string;
		ubicacionId: string;
	}

	let depositos = $state<Deposito[]>([]);
	let depositoId = $state('');
	let ubicaciones = $state<Ubicacion[]>([]);
	let tipo = $state<TipoMovimiento>('ENTRADA');
	let motivo = $state<MotivoMovimiento>('COMPRA');
	let observacion = $state('');
	let lineas = $state<LineaForm[]>([nuevaLinea()]);

	let movimientos = $state<MovimientoDetalle[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let filtroTipo = $state('');
	let filtroDeposito = $state('');

	let cargando = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let cargado = $state(false);

	const esGestion = $derived(
		$sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'
	);
	const totalPaginas = $derived(Math.max(1, Math.ceil(total / porPagina)));
	const depositoActual = $derived(depositos.find((d) => String(d.id) === depositoId) ?? null);

	// El signo de la cantidad depende del tipo; el input lo pide en positivo.
	const restoPositivo = $derived(tipo === 'ENTRADA' || tipo === 'AJUSTE');
	const pideObservacion = $derived(motivo === 'OTRO');

	// Si la sesión cambia a OPERACION, el AJUSTE se cae al primer tipo válido.
	$effect(() => {
		if (!esGestion && tipo === 'AJUSTE') {
			tipo = 'ENTRADA';
		}
	});

	// Al cambiar el depósito se recargan sus ubicaciones.
	$effect(() => {
		const id = depositoId;
		if (!id) {
			ubicaciones = [];
			return;
		}
		void cargarUbicaciones(Number(id));
	});

	// Al cambiar el tipo, el motivo vuelve a una opción coherente.
	$effect(() => {
		if (tipo !== 'AJUSTE' && motivo === 'AJUSTE') {
			motivo = 'COMPRA';
		}
	});

	$effect(() => {
		if (!cargado && $sesion.estado === 'activa') {
			cargado = true;
			void iniciar();
		}
	});

	function nuevaLinea(): LineaForm {
		return { producto: null, busqueda: '', resultados: [], cantidad: '1', ubicacionId: '' };
	}

	function mensajeError(e: unknown, fallback: string): string {
		if (e instanceof ApiError) return e.message;
		if (e instanceof Error) return e.message;
		return fallback;
	}

	async function iniciar() {
		try {
			depositos = await listarDepositos('true');
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los depósitos');
		}
		await cargarMovimientos();
	}

	async function cargarUbicaciones(id: number) {
		try {
			ubicaciones = await listarUbicaciones(id);
		} catch {
			ubicaciones = [];
		}
	}

	async function cargarMovimientos() {
		cargando = true;
		try {
			const resultado = await listarMovimientos({
				tipo: filtroTipo || undefined,
				depositoId: filtroDeposito ? Number(filtroDeposito) : undefined,
				pagina,
				porPagina
			});
			movimientos = resultado.items;
			total = resultado.total;
			error = null;
		} catch (e) {
			error = mensajeError(e, 'No se pudo cargar el historial');
		} finally {
			cargando = false;
		}
	}

	let secuenciaBusqueda = 0;
	let temporizadores = new Map<number, ReturnType<typeof setTimeout>>();

	function buscarEnLinea(index: number, texto: string) {
		const turno = ++secuenciaBusqueda;
		const previo = temporizadores.get(index);
		if (previo) clearTimeout(previo);
		const limpio = texto.trim();
		if (limpio.length < 2) {
			lineas[index] = { ...lineas[index], resultados: [], busqueda: texto };
			return;
		}
		temporizadores.set(
			index,
			setTimeout(() => {
				void (async () => {
					try {
						const r = await buscarProductos({ q: limpio, limite: 8 });
						if (turno !== secuenciaBusqueda) return;
						lineas[index] = { ...lineas[index], resultados: r.items, busqueda: texto };
					} catch {
						if (turno !== secuenciaBusqueda) return;
						lineas[index] = { ...lineas[index], resultados: [], busqueda: texto };
					}
				})();
			}, 250)
		);
	}

	function elegirProducto(index: number, producto: Producto) {
		lineas[index] = {
			...lineas[index],
			producto,
			busqueda: producto.nombre,
			resultados: [],
			ubicacionId: ''
		};
	}

	function agregarLinea() {
		if (lineas.length < 200) lineas = [...lineas, nuevaLinea()];
	}

	function quitarLinea(index: number) {
		if (lineas.length > 1) lineas = lineas.filter((_, i) => i !== index);
	}

	function deltaDe(linea: LineaForm): number | null {
		const bruto = Number(linea.cantidad);
		if (!Number.isInteger(bruto) || bruto <= 0) return null;
		return tipo === 'SALIDA' || tipo === 'MERMA' ? -bruto : bruto;
	}

	function validar(): string | null {
		if (!depositoId) return 'Elegí un depósito.';
		if (pideObservacion && !observacion.trim())
			return 'Con motivo OTRO hace falta una observación.';
		for (const [i, linea] of lineas.entries()) {
			if (!linea.producto) return `Elegí un producto en la línea ${i + 1}.`;
			if (deltaDe(linea) === null) return `La cantidad de la línea ${i + 1} debe ser un entero mayor que cero.`;
		}
		return null;
	}

	async function enviar() {
		aviso = null;
		error = validar();
		if (error) return;

		const payload = {
			tipo,
			deposito_id: Number(depositoId),
			motivo,
			observacion: observacion.trim() || null,
			lineas: lineas.map((linea) => ({
				producto_id: linea.producto!.id,
				ubicacion_id: linea.ubicacionId ? Number(linea.ubicacionId) : null,
				cantidad_delta: deltaDe(linea)!
			})) satisfies LineaNuevaInput[]
		};

		guardando = true;
		try {
			const creado = await crearMovimiento(payload);
			aviso = `Movimiento #${creado.id} registrado: ${creado.lineas
				.map((l) => `${l.producto_nombre} → ${l.stock_resultante} en ${creado.deposito.nombre}`)
				.join('; ')}`;
			lineas = [nuevaLinea()];
			observacion = '';
			pagina = 1;
			await cargarMovimientos();
		} catch (e) {
			error = mensajeError(e, 'No se pudo registrar el movimiento');
		} finally {
			guardando = false;
		}
	}

	function cambiarPagina(nueva: number) {
		pagina = nueva;
		void cargarMovimientos();
	}

	function aplicarFiltros() {
		pagina = 1;
		void cargarMovimientos();
	}

	const signoDelta = (delta: number) => (delta > 0 ? `+${delta}` : `${delta}`);
</script>

<svelte:head>
	<title>Movimientos · Stockmante</title>
</svelte:head>

<div class="space-y-6">
	<h1 class="text-xl font-bold">Movimientos de stock</h1>

	{#if error}
		<div class="rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
			{error}
		</div>
	{/if}
	{#if aviso}
		<div class="rounded border border-green-200 bg-green-50 px-3 py-2 text-sm text-green-700">
			{aviso}
		</div>
	{/if}

	<section class="rounded-lg border border-slate-200 bg-white p-4">
		<h2 class="mb-3 font-semibold">Registrar movimiento</h2>

		<div class="mb-3 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
			<label class="block text-sm">
				<span class="mb-1 block text-slate-600">Tipo</span>
				<select
					class="w-full rounded border border-slate-300 px-2 py-1.5"
					bind:value={tipo}
				>
					{#each TIPOS as opcion}
						{#if opcion.valor !== 'AJUSTE' || esGestion}
							<option value={opcion.valor}>{opcion.etiqueta}</option>
						{/if}
					{/each}
				</select>
			</label>

			<label class="block text-sm">
				<span class="mb-1 block text-slate-600">Depósito</span>
				<select class="w-full rounded border border-slate-300 px-2 py-1.5" bind:value={depositoId}>
					<option value="">Elegí…</option>
					{#each depositos as deposito}
						<option value={String(deposito.id)}>{deposito.nombre}</option>
					{/each}
				</select>
			</label>

			<label class="block text-sm">
				<span class="mb-1 block text-slate-600">Motivo</span>
				<select class="w-full rounded border border-slate-300 px-2 py-1.5" bind:value={motivo}>
					{#each MOTIVOS as opcion}
						<option value={opcion.valor}>{opcion.etiqueta}</option>
					{/each}
				</select>
			</label>

			<label class="block text-sm">
				<span class="mb-1 block text-slate-600">
					Observación {pideObservacion ? '(obligatoria con OTRO)' : '(opcional)'}
				</span>
				<input
					type="text"
					class="w-full rounded border border-slate-300 px-2 py-1.5"
					maxlength="500"
					bind:value={observacion}
				/>
			</label>
		</div>

		<div class="space-y-3">
			{#each lineas as linea, index}
				<div class="rounded border border-slate-200 p-3">
					<div class="flex flex-wrap items-end gap-2">
						<label class="min-w-52 flex-1 text-sm">
							<span class="mb-1 block text-slate-600">Producto (nombre o SKU)</span>
							<input
								type="text"
								class="w-full rounded border border-slate-300 px-2 py-1.5"
								value={linea.busqueda}
								oninput={(e) => buscarEnLinea(index, e.currentTarget.value)}
							/>
							{#if linea.resultados.length > 0 && !linea.producto}
								<ul
									class="mt-1 max-h-44 overflow-auto rounded border border-slate-200 bg-white text-xs shadow"
								>
									{#each linea.resultados as resultado}
										<li>
											<button
												type="button"
												class="block w-full px-2 py-1 text-left hover:bg-slate-50"
												onclick={() => elegirProducto(index, resultado)}
											>
												<span class="font-medium">{resultado.nombre}</span>
												<span class="ml-2 text-slate-500">{resultado.sku}</span>
											</button>
										</li>
									{/each}
								</ul>
							{/if}
							{#if linea.producto}
								<span class="mt-1 inline-block rounded bg-slate-100 px-1.5 py-0.5 text-xs">
									{linea.producto.sku}
									<button
										type="button"
										class="ml-1 text-slate-500 hover:text-slate-800"
										onclick={() =>
											(lineas[index] = { ...nuevaLinea(), busqueda: linea.busqueda })}
									>
										✕
									</button>
								</span>
							{/if}
						</label>

						<label class="w-28 text-sm">
							<span class="mb-1 block text-slate-600">Cantidad</span>
							<input
								type="number"
								min="1"
								step="1"
								class="w-full rounded border border-slate-300 px-2 py-1.5"
								bind:value={linea.cantidad}
							/>
						</label>

						<label class="w-48 text-sm">
							<span class="mb-1 block text-slate-600">Ubicación (opcional)</span>
							<select class="w-full rounded border border-slate-300 px-2 py-1.5" bind:value={linea.ubicacionId}>
								<option value="">Sin ubicación</option>
								{#each ubicaciones as ubicacion}
									<option value={String(ubicacion.id)}>{ubicacion.nombre}</option>
								{/each}
							</select>
						</label>

						{#if lineas.length > 1}
							<button
								type="button"
								class="rounded border border-slate-300 px-2 py-1.5 text-sm hover:bg-slate-50"
								onclick={() => quitarLinea(index)}
							>
								Quitar
							</button>
						{/if}
					</div>
					{#if linea.producto && deltaDe(linea) !== null}
						<p class="mt-2 text-xs text-slate-500">
							{tipo}: {signoDelta(deltaDe(linea)!)} × {linea.producto.nombre}
						</p>
					{/if}
				</div>
			{/each}
		</div>

		<div class="mt-3 flex items-center gap-3">
			<button
				type="button"
				class="rounded border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-50"
				onclick={agregarLinea}
			>
				+ Agregar línea
			</button>
			<button
				type="button"
				class="rounded bg-blue-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-blue-700 disabled:opacity-50"
				disabled={guardando}
				onclick={() => void enviar()}
			>
				{guardando ? 'Registrando…' : 'Registrar movimiento'}
			</button>
		</div>
	</section>

	<section class="rounded-lg border border-slate-200 bg-white p-4">
		<div class="mb-3 flex flex-wrap items-center gap-3">
			<h2 class="mr-auto font-semibold">Historial</h2>
			<label class="text-sm">
				<span class="mr-1 text-slate-600">Tipo</span>
				<select class="rounded border border-slate-300 px-2 py-1" bind:value={filtroTipo}>
					<option value="">Todos</option>
					{#each TIPOS as opcion}
						<option value={opcion.valor}>{opcion.etiqueta}</option>
					{/each}
				</select>
			</label>
			<label class="text-sm">
				<span class="mr-1 text-slate-600">Depósito</span>
				<select class="rounded border border-slate-300 px-2 py-1" bind:value={filtroDeposito}>
					<option value="">Todos</option>
					{#each depositos as deposito}
						<option value={String(deposito.id)}>{deposito.nombre}</option>
					{/each}
				</select>
			</label>
			<button
				type="button"
				class="rounded border border-slate-300 px-3 py-1 text-sm hover:bg-slate-50"
				onclick={aplicarFiltros}
			>
				Filtrar
			</button>
		</div>

		{#if cargando}
			<p class="py-6 text-center text-sm text-slate-500">Cargando…</p>
		{:else if movimientos.length === 0}
			<p class="py-6 text-center text-sm text-slate-500">No hay movimientos registrados.</p>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full text-left text-sm">
					<thead class="border-b border-slate-200 text-slate-500">
						<tr>
							<th class="py-2 pr-3">#</th>
							<th class="py-2 pr-3">Fecha</th>
							<th class="py-2 pr-3">Tipo</th>
							<th class="py-2 pr-3">Motivo</th>
							<th class="py-2 pr-3">Depósito</th>
							<th class="py-2 pr-3">Autor</th>
							<th class="py-2 pr-3">Líneas</th>
						</tr>
					</thead>
					<tbody>
						{#each movimientos as movimiento (movimiento.id)}
							<tr class="border-b border-slate-100 align-top">
								<td class="py-2 pr-3">{movimiento.id}</td>
								<td class="py-2 pr-3 whitespace-nowrap">
									{formatoFecha.format(new Date(movimiento.creado_en))}
								</td>
								<td class="py-2 pr-3">
									<span
										class="rounded px-1.5 py-0.5 text-xs
										{movimiento.tipo === 'ENTRADA'
											? 'bg-green-100 text-green-700'
											: movimiento.tipo === 'AJUSTE'
												? 'bg-purple-100 text-purple-700'
												: 'bg-red-100 text-red-700'}"
									>
										{movimiento.tipo}
									</span>
								</td>
								<td class="py-2 pr-3">
									{movimiento.motivo}
									{#if movimiento.observacion}
										<span class="block text-xs text-slate-500">{movimiento.observacion}</span>
									{/if}
								</td>
								<td class="py-2 pr-3">{movimiento.deposito.nombre}</td>
								<td class="py-2 pr-3">{movimiento.usuario.nombre}</td>
								<td class="py-2 pr-3">
									<ul class="space-y-0.5 text-xs">
										{#each movimiento.lineas as linea}
											<li>
												{linea.producto_nombre}
												<span class="text-slate-500">({linea.sku})</span>
												<span class="font-medium {linea.cantidad_delta > 0 ? 'text-green-700' : 'text-red-700'}">
													{signoDelta(linea.cantidad_delta)}
												</span>
												<span class="text-slate-500">→ {linea.stock_resultante}</span>
											</li>
										{/each}
									</ul>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			{#if totalPaginas > 1}
				<div class="mt-3 flex items-center justify-end gap-2 text-sm">
					<button
						type="button"
						class="rounded border border-slate-300 px-2 py-1 disabled:opacity-40"
						disabled={pagina <= 1}
						onclick={() => cambiarPagina(pagina - 1)}
					>
						Anterior
					</button>
					<span class="text-slate-600">Página {pagina} de {totalPaginas}</span>
					<button
						type="button"
						class="rounded border border-slate-300 px-2 py-1 disabled:opacity-40"
						disabled={pagina >= totalPaginas}
						onclick={() => cambiarPagina(pagina + 1)}
					>
						Siguiente
					</button>
				</div>
			{/if}
		{/if}
	</section>

	{#if depositoActual && ubicaciones.length === 0}
		<p class="text-xs text-slate-500">
			{depositoActual.nombre} no tiene ubicaciones cargadas: los movimientos se imputan al
			depósito hasta que se asignen ubicaciones al producto.
		</p>
	{/if}
</div>
