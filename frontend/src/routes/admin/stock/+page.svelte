<script lang="ts">
	import { onMount } from 'svelte';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import ListaTarjetas from '#lib/components/movil/ListaTarjetas.svelte';
	import TarjetaInventario from '#lib/components/movil/TarjetaInventario.svelte';
	import FiltrosMovil from '#lib/components/movil/FiltrosMovil.svelte';
	import BotonFlotante from '#lib/components/movil/BotonFlotante.svelte';
	import HojaMovimiento from '#lib/components/movimientos/HojaMovimiento.svelte';
	import { page } from '$app/state';
	import { ApiError } from '#lib/api/client';
	import { listarDepositos } from '#lib/api/depositos';
	import { listarCategorias } from '#lib/api/categorias';
	import { listarStock, obtenerStockProducto } from '#lib/api/stock';
	import type { Categoria } from '#lib/schemas/categorias';
	import type { Deposito } from '#lib/schemas/depositos';
	import type { StockItem, StockProducto } from '#lib/schemas/movimientos';
	import { formatearNumero } from '#lib/utils/formato';
	import { textoValorizado } from '#lib/utils/stock';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import EtiquetaEstado from '#lib/components/ui/EtiquetaEstado.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import TablaDatos from '#lib/components/ui/TablaDatos.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	import type { Columna } from '#lib/components/ui/tabla';
	import { enviarAlertas, listarAlertas } from '#lib/api/alertas';
	import type { AlertaItem } from '#lib/schemas/alertas';

	// 011: sección destacada "Bajo mínimo" (independiente del filtro de 010).
	let alertas = $state<AlertaItem[]>([]);
	let totalAlertas = $state(0);
	let errorAlertas = $state('');
	let resultadoEnvio = $state('');
	let enviando = $state(false);

	async function cargarAlertasBajoMinimo() {
		errorAlertas = '';
		try {
			const res = await listarAlertas({ porPagina: 10 });
			alertas = res.items;
			totalAlertas = res.total;
		} catch (e) {
			errorAlertas = `No se pudieron cargar las alertas: ${e instanceof ApiError ? e.message : 'sin respuesta'}.`;
		}
	}

	async function enviarDigest() {
		enviando = true;
		resultadoEnvio = '';
		try {
			const r = await enviarAlertas();
			resultadoEnvio =
				r.fallidos > 0
					? `Se enviaron ${r.enviados} avisos; ${r.fallidos} fallaron y se reintentan en el próximo ciclo.`
					: `Se enviaron ${r.enviados} avisos a ${r.destinatarios} gestores (${r.productos} productos bajo mínimo).`;
		} catch (e) {
			resultadoEnvio = `No se pudo disparar el envío: ${e instanceof ApiError ? e.message : 'sin respuesta'}.`;
		} finally {
			enviando = false;
		}
	}

	const OPCIONES_ACTIVO = [
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' },
		{ valor: 'all', etiqueta: 'Todos' }
	];

	let depositos = $state<Deposito[]>([]);
	let categorias = $state<Categoria[]>([]);
	let depositoId = $state('');
	let categoriaId = $state('');
	let activo = $state('true');
	let q = $state(page.url.searchParams.get('q') ?? '');
	let bajoMinimo = $state(false);
	let errorFiltros = $state('');
	let detalle = $state<StockProducto | null>(null);
	let errorDetalle = $state('');
	let itemMovil = $state<StockItem | null>(null);
	let hojaAbierta = $state(false);

	function abrirHoja(item: StockItem) {
		itemMovil = item;
		hojaAbierta = true;
	}

	function recargarTrasMovimiento() {
		void cargar();
		void cargarAlertasBajoMinimo();
	}

	// 010: stock global, en depósito (si hay uno elegido), mínimo, faltante y valorizado.
	const COLUMNAS: Columna<StockItem>[] = $derived([
		{ clave: 'nombre', encabezado: 'Producto' },
		{ clave: 'sku', encabezado: 'SKU' },
		{ clave: 'stock_actual', encabezado: 'Stock global', alineacion: 'derecha' },
		...(depositoId
			? [{ clave: 'stock_deposito', encabezado: 'En depósito', alineacion: 'derecha' as const }]
			: []),
		{ clave: 'stock_minimo', encabezado: 'Mínimo', alineacion: 'derecha' },
		{ clave: 'faltante', encabezado: 'Faltante', alineacion: 'derecha' },
		{ clave: 'costo', encabezado: 'Costo', alineacion: 'derecha' },
		{ clave: 'valorizado', encabezado: 'Valorizado', alineacion: 'derecha' },
		{ clave: 'bajo_minimo', encabezado: 'Bajo mínimo', alineacion: 'centro' }
	]);
	let items = $state<StockItem[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let cargando = $state(false);
	let error = $state('');

	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Todos' },
		...depositos.map((deposito) => ({ valor: String(deposito.id), etiqueta: deposito.nombre }))
	]);

	const opcionesCategorias = $derived([
		{ valor: '', etiqueta: 'Todas' },
		...categorias.map((categoria) => ({ valor: String(categoria.id), etiqueta: categoria.nombre }))
	]);

	const chipsStock = $derived([
		{ etiqueta: 'Buscar', valor: q.trim() },
		{
			etiqueta: 'Depósito',
			valor: opcionesDepositos.find((opcion) => opcion.valor === depositoId)?.etiqueta ?? ''
		},
		{
			etiqueta: 'Categoría',
			valor: opcionesCategorias.find((opcion) => opcion.valor === categoriaId)?.etiqueta ?? ''
		},
		{
			etiqueta: 'Estado',
			valor:
				activo === 'true'
					? ''
					: (OPCIONES_ACTIVO.find((opcion) => opcion.valor === activo)?.etiqueta ?? '')
		},
		{ etiqueta: 'Bajo mínimo', valor: bajoMinimo ? 'Sí' : '' }
	]);

	function motivo(e: unknown, generico: string): string {
		return e instanceof ApiError ? e.message : generico;
	}

	async function cargar() {
		cargando = true;
		error = '';
		try {
			const res = await listarStock({
				q: q || undefined,
				depositoId: depositoId ? Number(depositoId) : undefined,
				categoriaId: categoriaId ? Number(categoriaId) : undefined,
				activo: activo as 'true' | 'false' | 'all',
				bajoMinimo,
				pagina,
				porPagina
			});
			items = res.items;
			total = res.total;
		} catch (e) {
			error = `No se pudo cargar el stock: ${motivo(e, 'el servidor no respondió')}. Revisá la conexión y reintentá.`;
		} finally {
			cargando = false;
		}
	}

	/** Depósitos y categorías para los filtros; un fallo se informa, no se descarta. */
	async function cargarFiltros() {
		errorFiltros = '';
		try {
			[depositos, categorias] = await Promise.all([
				listarDepositos('true'),
				listarCategorias('all')
			]);
		} catch (e) {
			errorFiltros = `No se pudieron cargar los depósitos o las categorías: ${motivo(e, 'sin respuesta')}. Los filtros quedan incompletos hasta reintentar.`;
		}
	}

	async function verDesglose(item: StockItem) {
		errorDetalle = '';
		try {
			detalle = await obtenerStockProducto(item.producto_id);
		} catch (e) {
			detalle = null;
			errorDetalle = `No se pudo abrir el desglose de ${item.nombre}: ${motivo(e, 'sin respuesta')}.`;
		}
	}

	function aplicarFiltros() {
		pagina = 1;
		void cargar();
	}

	function cambiarPagina(nueva: number) {
		pagina = nueva;
		void cargar();
	}

	function limpiarFiltros() {
		q = '';
		depositoId = '';
		categoriaId = '';
		activo = 'true';
		bajoMinimo = false;
		pagina = 1;
		void cargar();
	}

	onMount(async () => {
		await Promise.all([cargarFiltros(), cargar(), cargarAlertasBajoMinimo()]);
	});
</script>

<svelte:head>
	<title>Stock · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Stock"
		descripcion="Consultá el stock por producto y depósito, y detectá los casos por debajo del mínimo."
	/>

	<Tarjeta>
		<div class="detalle-cabecera">
			<h2>Bajo mínimo ({totalAlertas})</h2>
			<Boton onclick={() => void enviarDigest()} cargando={enviando}>Enviar aviso por correo</Boton>
		</div>
		{#if resultadoEnvio}<p class="detalle-resumen" aria-live="polite">{resultadoEnvio}</p>{/if}
		{#if errorAlertas}
			<Aviso variante="error">
				{errorAlertas}
				{#snippet accion()}<Boton onclick={() => void cargarAlertasBajoMinimo()}>Reintentar</Boton>{/snippet}
			</Aviso>
		{:else if alertas.length === 0}
			<p class="detalle-resumen">Ningún producto activo está por debajo de su mínimo.</p>
		{:else}
			<ul class="desglose">
				{#each alertas as a (a.producto_id)}
					<li>
						<span>{a.nombre} <span class="mono">{a.sku}</span></span>
						<span class="numero">
							stock {formatearNumero(a.stock_actual)} · mínimo {formatearNumero(a.stock_minimo)} ·
							<strong class="alerta">faltan {formatearNumero(a.faltante)}</strong>
						</span>
					</li>
				{/each}
			</ul>
			{#if totalAlertas > alertas.length}
				<p class="detalle-resumen">
					Y {totalAlertas - alertas.length} más: usá el filtro "Bajo mínimo" para verlos todos.
				</p>
			{/if}
		{/if}
	</Tarjeta>

	{#if error}
		<Aviso variante="error">
			{error}
			{#snippet accion()}<Boton onclick={() => void cargar()}>Reintentar</Boton>{/snippet}
		</Aviso>
	{/if}
	{#if errorFiltros}
		<Aviso variante="advertencia">
			{errorFiltros}
			{#snippet accion()}<Boton onclick={() => void cargarFiltros()}>Reintentar</Boton>{/snippet}
		</Aviso>
	{/if}

	{#snippet tarjetaFiltros()}
		<Tarjeta>
			<div class="filtros">
			<Campo
				etiqueta="Buscar"
				nombre="q"
				tipo="search"
				marcador="Producto o SKU"
				bind:valor={q}
				onkeydown={(event) => {
					if (event.key === 'Enter') aplicarFiltros();
				}}
			/>
			<Campo
				etiqueta="Depósito"
				nombre="deposito"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={depositoId}
				onchange={aplicarFiltros}
			/>
			<Campo
				etiqueta="Categoría"
				nombre="categoria"
				tipo="select"
				opciones={opcionesCategorias}
				bind:valor={categoriaId}
				onchange={aplicarFiltros}
			/>
			<Campo
				etiqueta="Estado"
				nombre="activo"
				tipo="select"
				opciones={OPCIONES_ACTIVO}
				bind:valor={activo}
				onchange={aplicarFiltros}
			/>
			<label class="checkbox">
				<input type="checkbox" bind:checked={bajoMinimo} onchange={aplicarFiltros} />
				Bajo mínimo
			</label>
			<div class="accion-filtro">
				<Boton onclick={aplicarFiltros}>Buscar</Boton>
			</div>
			</div>
		</Tarjeta>
	{/snippet}

	<div id="filtros-stock">
	{#if $dispositivo.esMovil}
		<FiltrosMovil chips={chipsStock} onlimpiar={limpiarFiltros}>
			{@render tarjetaFiltros()}
		</FiltrosMovil>
	{:else}
		{@render tarjetaFiltros()}
	{/if}
	</div>

	{#if $dispositivo.esMovil}
		<ListaTarjetas items={items} etiqueta="Stock por producto y depósito" {cargando}>
			{#snippet tarjeta(item)}
				<TarjetaInventario
					nombre={item.nombre}
					sku={item.sku}
					stockActual={item.stock_actual}
					bajoMinimo={item.bajo_minimo}
					contexto={`${item.categoria_nombre}${depositoId ? ` · en depósito: ${formatearNumero(item.stock_deposito ?? 0)}` : ''}`}
					onabrir={() => void verDesglose(item)}
				>
					{#snippet acciones()}
						<Boton variante="secundario" tamano="sm" onclick={() => void verDesglose(item)}>
							Ver desglose
						</Boton>
						<Boton variante="primario" tamano="sm" onclick={() => abrirHoja(item)}>
							Ajustar
						</Boton>
					{/snippet}
				</TarjetaInventario>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio
					icono="stock"
					titulo="Sin resultados de stock"
					descripcion="Probá con otro producto, depósito o quitá el filtro de bajo mínimo."
				/>
			{/snippet}
		</ListaTarjetas>
	{:else}
	<Tarjeta densa={true}>
		<TablaDatos
			columnas={COLUMNAS}
			filas={items}
			etiqueta="Stock por producto y depósito"
			cargando={cargando}
		>
			{#snippet fila(item)}
				<td class="principal">
					<button
						type="button"
						class="enlace"
						onclick={() => void verDesglose(item)}
						aria-label="Ver desglose de {item.nombre}"
					>
						{item.nombre}
					</button>
				</td>
				<td class="mono">{item.sku}</td>
				<td class="numero" class:alerta={item.bajo_minimo}>
					{formatearNumero(item.stock_actual)}
				</td>
				{#if depositoId}
					<td class="numero">{formatearNumero(item.stock_deposito ?? 0)}</td>
				{/if}
				<td class="numero">{formatearNumero(item.stock_minimo)}</td>
				<td class="numero" class:alerta={item.faltante > 0}>{formatearNumero(item.faltante)}</td>
				<td class="numero">{item.costo === null ? '—' : textoValorizado({ valorizado: item.costo })}</td>
				<td class="numero">{textoValorizado(item)}</td>
				<td class="centro">
					<EtiquetaEstado variante={item.bajo_minimo ? 'aviso' : 'exito'}>
						{item.bajo_minimo ? 'Sí' : 'No'}
					</EtiquetaEstado>
				</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio
					icono="stock"
					titulo="Sin resultados de stock"
					descripcion="Probá con otro producto, depósito o quitá el filtro de bajo mínimo."
				/>
			{/snippet}
		</TablaDatos>
	</Tarjeta>
	{/if}

	<div class="pie">
		<Paginacion {pagina} {total} {porPagina} onCambiar={cambiarPagina} />
	</div>

	{#if errorDetalle}<Aviso variante="error">{errorDetalle}</Aviso>{/if}
	{#if detalle}
		<Tarjeta>
			<div class="detalle-cabecera">
				<h2>Desglose de {detalle.nombre}</h2>
				<Boton onclick={() => (detalle = null)}>Cerrar</Boton>
			</div>
			<p class="detalle-resumen">
				Stock global {formatearNumero(detalle.stock_actual)} · mínimo
				{formatearNumero(detalle.stock_minimo)} · faltante {formatearNumero(detalle.faltante)}
			</p>
			{#if detalle.desglose.length === 0}
				<p class="detalle-resumen">Sin existencias registradas en ningún depósito.</p>
			{:else}
				<ul class="desglose">
					{#each detalle.desglose as fila (`${fila.deposito_id}-${fila.ubicacion_id}`)}
						<li>
							<span>{fila.deposito_nombre}{fila.ubicacion_nombre ? ` / ${fila.ubicacion_nombre}` : ''}</span>
							<span class="numero">{formatearNumero(fila.cantidad)}</span>
						</li>
					{/each}
				</ul>
			{/if}
		</Tarjeta>
	{/if}

	{#if $dispositivo.esMovil}
		<BotonFlotante
			etiqueta="Buscar producto para ajustar"
			icono="buscar"
			onclick={() =>
				document.getElementById('filtros-stock')?.scrollIntoView({ behavior: 'smooth' })}
		/>
	{/if}

	<HojaMovimiento
		abierta={hojaAbierta}
		item={itemMovil}
		oncerrar={() => (hojaAbierta = false)}
		onregistrado={recargarTrasMovimiento}
	/>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filtros {
		display: flex;
		align-items: flex-end;
		gap: 12px;
		flex-wrap: wrap;
	}

	.filtros :global(.campo) {
		min-width: 190px;
	}

	.checkbox {
		display: flex;
		align-items: center;
		gap: 6px;
		padding-bottom: 10px;
		color: var(--texto-suave);
		font-size: 13px;
		white-space: nowrap;
	}

	.checkbox input {
		width: 15px;
		height: 15px;
		accent-color: var(--primario);
	}

	.accion-filtro {
		padding-bottom: 1px;
	}

	.principal {
		font-weight: 500;
	}

	.mono {
		color: var(--texto-suave);
		font-family: var(--fuente-mono);
		font-size: 11.5px;
	}

	.numero {
		font-variant-numeric: tabular-nums;
	}

	/* Las celdas del snippet no heredan .alinear-* de TablaDatos (CSS con alcance). */
	td.numero {
		text-align: right;
		white-space: nowrap;
	}

	td.centro {
		text-align: center;
	}

	strong.alerta,
	.numero.alerta {
		color: var(--ambar-texto);
		font-weight: 700;
	}

	.pie {
		padding: 0 2px;
	}

	.enlace {
		border: 0;
		background: none;
		padding: 0;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.enlace:hover,
	.enlace:focus-visible {
		color: var(--primario);
		text-decoration: underline;
	}

	.detalle-cabecera {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.detalle-cabecera h2 {
		margin: 0;
		font-size: 15px;
	}

	.detalle-resumen {
		margin: 8px 0;
		color: var(--texto-suave);
		font-size: 13px;
	}

	.desglose {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.desglose li {
		display: flex;
		justify-content: space-between;
		padding: 6px 0;
		border-top: 1px solid var(--borde);
		font-size: 13px;
	}

	@media (max-width: 720px) {
		.filtros {
			flex-direction: column;
			align-items: stretch;
		}

		.filtros :global(.campo) {
			min-width: 100%;
		}

		.checkbox {
			padding-bottom: 0;
		}
	}
</style>
