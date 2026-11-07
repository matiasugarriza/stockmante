<script lang="ts">
	import { onMount } from 'svelte';
	import { listarCategorias } from '#lib/api/categorias';
	import { ApiError } from '#lib/api/client';
	import { listarDepositosActivos, type DepositoActivo } from '#lib/api/depositos';
	import { buscarProductos, type CampoBusqueda } from '#lib/api/productos';
	import { listarStock } from '#lib/api/stock';
	import { elegirDeposito, indexarPorProducto } from '#lib/utils/stock';
	import { crearVenta } from '#lib/api/ventas';
	import { listarClientes } from '#lib/api/clientes';
	import type { Cliente } from '#lib/schemas/clientes';
	import type { MedioPago, VentaDetalle } from '#lib/schemas/ventas';
	import {
		agregarAlCarrito,
		cambiarCantidad,
		errorDescuento,
		lineasParaVenta,
		quitarDelCarrito,
		totalesCarrito,
		type ItemCarrito
	} from '#lib/utils/carrito';
	import type { Categoria } from '#lib/schemas/categorias';
	import type { Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';
	import {
		crearBusquedaDiferida,
		LIMITE_CAJA,
		mensajeTruncados
	} from '#lib/utils/busquedaCaja';
	import { formatearMoneda } from '#lib/utils/formato';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import EtiquetaEstado from '#lib/components/ui/EtiquetaEstado.svelte';
	import TablaDatos from '#lib/components/ui/TablaDatos.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	import type { Columna } from '#lib/components/ui/tabla';

	const CAMPOS: { valor: CampoBusqueda; etiqueta: string }[] = [
		{ valor: 'todos', etiqueta: 'Todo' },
		{ valor: 'sku', etiqueta: 'SKU exacto' },
		{ valor: 'codigo', etiqueta: 'Código exacto' },
		{ valor: 'nombre', etiqueta: 'Nombre' },
		{ valor: 'categoria', etiqueta: 'Categoría' }
	];

	// 013: punto de venta (carrito, descuento, medio de pago y cliente opcional).
	const MEDIOS: { valor: MedioPago; etiqueta: string }[] = [
		{ valor: 'EFECTIVO', etiqueta: 'Efectivo' },
		{ valor: 'TARJETA', etiqueta: 'Tarjeta' },
		{ valor: 'TRANSFERENCIA', etiqueta: 'Transferencia' }
	];
	let carrito = $state<ItemCarrito[]>([]);
	let descuentoTexto = $state('0');
	let medioPago = $state<string>('EFECTIVO');
	let clienteId = $state('');
	let clientes = $state<Cliente[]>([]);
	let vendiendo = $state(false);
	let errorVenta = $state<string | null>(null);
	let ultimaVenta = $state<VentaDetalle | null>(null);

	const descuento = $derived(Number(descuentoTexto.replace(',', '.') || '0'));
	const totales = $derived(totalesCarrito(carrito, Number.isNaN(descuento) ? 0 : descuento));
	const problemaDescuento = $derived(errorDescuento(descuento, totales.subtotal));
	const opcionesClientes = $derived([
		{ valor: '', etiqueta: 'Consumidor final' },
		...clientes.map((c) => ({ valor: String(c.id), etiqueta: c.nombre }))
	]);

	function agregar(producto: Producto) {
		ultimaVenta = null;
		carrito = agregarAlCarrito(carrito, {
			producto_id: producto.id,
			nombre: producto.nombre,
			precio: producto.precio_venta
		});
	}

	async function cargarClientes() {
		try {
			clientes = (await listarClientes({ activo: true, porPagina: 100 })).items;
		} catch {
			clientes = [];
		}
	}

	async function confirmarVenta() {
		if (depositoId === null || carrito.length === 0 || problemaDescuento) return;
		vendiendo = true;
		errorVenta = null;
		try {
			ultimaVenta = await crearVenta({
				deposito_id: depositoId,
				cliente_id: clienteId ? Number(clienteId) : null,
				medio_pago: medioPago as MedioPago,
				descuento,
				lineas: lineasParaVenta(carrito)
			});
			carrito = [];
			descuentoTexto = '0';
			void cargarStock(items);
		} catch (e) {
			errorVenta = mensajeError(e, 'No se pudo registrar la venta');
		} finally {
			vendiendo = false;
		}
	}

	/** Preferencia de pantalla (010, R1): depósito de la caja. */
	const CLAVE_DEPOSITO = 'stockmante.caja.deposito';

	let depositos = $state<DepositoActivo[]>([]);
	let depositoId = $state<number | null>(null);
	let errorDepositos = $state<string | null>(null);
	let stockPorProducto = $state<Map<number, number>>(new Map());
	let errorStock = $state<string | null>(null);
	let turnoStock = 0;

	const depositoActual = $derived(depositos.find((d) => d.id === depositoId) ?? null);
	const opcionesDepositos = $derived(
		depositos.map((d) => ({ valor: String(d.id), etiqueta: d.nombre }))
	);

	const COLUMNAS: Columna<Producto>[] = $derived([
		{ clave: 'nombre', encabezado: 'Producto' },
		{ clave: 'sku', encabezado: 'SKU' },
		{ clave: 'codigo_barras', encabezado: 'Código' },
		{ clave: 'categoria', encabezado: 'Categoría' },
		{ clave: 'unidad_medida', encabezado: 'Unidad' },
		{ clave: 'precio_venta', encabezado: 'Precio', alineacion: 'derecha' },
		{
			clave: 'stock_deposito',
			encabezado: depositoActual ? `Stock en ${depositoActual.nombre}` : 'Stock',
			alineacion: 'derecha'
		},
		{ clave: 'stock_minimo', encabezado: 'Stock mín.', alineacion: 'derecha' },
		{ clave: 'activo', encabezado: 'Estado' },
		{ clave: 'acciones', encabezado: '' }
	]);

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

	const busqueda = crearBusquedaDiferida(250);

	const esGestion = $derived(
		$sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'
	);
	const hayFiltro = $derived(q.trim().length > 0 || categoriaId !== '');
	const total = $derived(items.length);
	const avisoTruncados = $derived(mensajeTruncados(total, LIMITE_CAJA));
	const opcionesCategorias = $derived([
		{ valor: '', etiqueta: 'Todas' },
		...categorias.map((categoria) => ({
			valor: String(categoria.id),
			etiqueta: categoria.activo ? categoria.nombre : `${categoria.nombre} (inactiva)`
		}))
	]);

	onMount(() => {
		(document.getElementById('q') as HTMLInputElement | null)?.focus();
		void cargarCategorias();
		void cargarDepositos();
		void cargarClientes();
	});

	function leerPreferencia(): number | null {
		try {
			const valor = Number(localStorage.getItem(CLAVE_DEPOSITO));
			return Number.isInteger(valor) && valor > 0 ? valor : null;
		} catch {
			return null;
		}
	}

	function guardarPreferencia(id: number | null) {
		try {
			if (id === null) localStorage.removeItem(CLAVE_DEPOSITO);
			else localStorage.setItem(CLAVE_DEPOSITO, String(id));
		} catch {
			// Sin almacenamiento (modo privado): la preferencia dura lo que la pantalla.
		}
	}

	async function cargarDepositos() {
		errorDepositos = null;
		try {
			depositos = await listarDepositosActivos();
			depositoId = elegirDeposito(leerPreferencia(), depositos);
			guardarPreferencia(depositoId);
		} catch (e) {
			errorDepositos = `${mensajeError(e, 'No se pudieron cargar los depósitos')}. El stock por depósito no se muestra hasta reintentar.`;
		}
	}

	function cambiarDeposito(valor: string) {
		depositoId = valor ? Number(valor) : null;
		guardarPreferencia(depositoId);
		void cargarStock(items);
	}

	/** Stock del depósito elegido para los resultados visibles; descarta respuestas viejas. */
	async function cargarStock(productos: Producto[]) {
		const turno = ++turnoStock;
		errorStock = null;
		if (depositoId === null || productos.length === 0) {
			stockPorProducto = new Map();
			return;
		}
		try {
			const res = await listarStock({
				productoIds: productos.map((p) => p.id).slice(0, 50),
				depositoId,
				activo: 'all',
				porPagina: 50
			});
			if (turno !== turnoStock) return;
			stockPorProducto = indexarPorProducto(res.items);
		} catch (e) {
			if (turno !== turnoStock) return;
			stockPorProducto = new Map();
			errorStock = `${mensajeError(e, 'No se pudo consultar el stock')}. Reintentá la búsqueda.`;
		}
	}

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

		buscando = true;
		busqueda.programar((esVigente) =>
			ejecutar(esVigente, texto, campoActual, categoria, inactivos)
		);

		return () => busqueda.cancelar();
	});

	async function cargarCategorias() {
		try {
			categorias = await listarCategorias('all');
		} catch (e) {
			errorCategorias = mensajeError(e, 'No se pudieron cargar las categorías');
		}
	}

	async function ejecutar(
		esVigente: () => boolean,
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
				limite: LIMITE_CAJA
			});
			if (!esVigente()) return;
			items = resultado.items;
			consultado = true;
			error = null;
			ms = Math.round(performance.now() - inicio);
			void cargarStock(resultado.items);
		} catch (e) {
			if (!esVigente()) return;
			items = [];
			consultado = true;
			ms = null;
			error = mensajeError(e, 'No se pudo realizar la búsqueda');
		} finally {
			if (esVigente()) buscando = false;
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

<div class="pagina">
	<EncabezadoPagina
		titulo="Caja"
		descripcion="Buscá por SKU, código de barras, nombre o categoría. Los productos inactivos no se muestran salvo que los incluyas como gestión."
	/>

	<Tarjeta>
		<div class="filtros">
			<Campo
				etiqueta="Buscar"
				nombre="q"
				tipo="search"
				marcador="SKU, código, nombre o categoría"
				bind:valor={q}
			/>
			<Campo
				etiqueta="Campo"
				nombre="campo"
				tipo="select"
				opciones={CAMPOS}
				bind:valor={campo}
			/>
			<Campo
				etiqueta="Categoría"
				nombre="categoria"
				tipo="select"
				opciones={opcionesCategorias}
				bind:valor={categoriaId}
			/>
			{#if depositos.length > 0}
				<Campo
					etiqueta="Depósito"
					nombre="deposito"
					tipo="select"
					opciones={opcionesDepositos}
					valor={depositoId === null ? '' : String(depositoId)}
					onchange={cambiarDeposito}
				/>
			{/if}
			<div class="acciones-filtro">
				{#if esGestion}
					<label class="checkbox">
						<input type="checkbox" bind:checked={incluirInactivos} />
						Incluir inactivos
					</label>
				{/if}
				{#if hayFiltro}
					<Boton variante="fantasma" tamano="sm" onclick={limpiar}>Limpiar</Boton>
				{/if}
			</div>
		</div>

		{#if errorCategorias}
			<Aviso variante="advertencia">{errorCategorias}</Aviso>
		{/if}
		{#if errorDepositos}
			<Aviso variante="advertencia">
				{errorDepositos}
				{#snippet accion()}<Boton tamano="sm" onclick={() => void cargarDepositos()}>Reintentar</Boton>{/snippet}
			</Aviso>
		{:else if depositos.length === 0 && !errorDepositos}
			<p class="resumen">No hay depósitos activos: el stock por depósito no se puede mostrar.</p>
		{/if}
		{#if errorStock}
			<Aviso variante="advertencia">{errorStock}</Aviso>
		{/if}
	</Tarjeta>

	{#if error}
		<Aviso variante="error">{error}</Aviso>
	{/if}

	<p class="resumen" aria-live="polite">
		{#if buscando}
			Buscando…
		{:else if consultado}
			{total} resultado{total === 1 ? '' : 's'}{#if ms !== null}
				· {ms} ms{/if}
			{#if avisoTruncados}
				<span class="truncados">{avisoTruncados}</span>
			{/if}
		{:else if !hayFiltro}
			Escribí al menos 2 caracteres o elegí una categoría.
		{/if}
	</p>

	{#if consultado && !buscando && items.length === 0 && !error}
		<Tarjeta>
			<EstadoVacio
				icono="buscar"
				titulo="Sin resultados"
				descripcion="Probá con otro término o revisá si el producto está inactivo."
			/>
		</Tarjeta>
	{:else if items.length > 0}
		<Tarjeta densa={true}>
			<TablaDatos columnas={COLUMNAS} filas={items} etiqueta="Resultados de búsqueda de caja">
				{#snippet fila(producto)}
					<td class="principal">{producto.nombre}</td>
					<td class="mono">{producto.sku}</td>
					<td class="mono">{producto.codigo_barras ?? '—'}</td>
					<td>{producto.categoria.nombre}</td>
					<td>{producto.unidad_medida}</td>
					<td class="numero">{formatearMoneda(producto.precio_venta)}</td>
					<td class="numero">
						{depositoId === null ? '—' : (stockPorProducto.get(producto.id) ?? '…')}
					</td>
					<td class="numero">{producto.stock_minimo}</td>
					<td>
						<EtiquetaEstado variante={producto.activo ? 'exito' : 'neutro'}>
							{producto.activo ? 'Activo' : 'Inactivo'}
						</EtiquetaEstado>
					</td>
					<td>
						<Boton
							tamano="sm"
							variante="fantasma"
							deshabilitado={!producto.activo}
							onclick={() => agregar(producto)}
						>
							Agregar
						</Boton>
					</td>
				{/snippet}
			</TablaDatos>
		</Tarjeta>
	{/if}

	<Tarjeta>
		<h2 class="titulo-venta">Venta</h2>
		{#if ultimaVenta}
			<Aviso variante="exito">
				Venta {ultimaVenta.venta.numero} registrada en {ultimaVenta.venta.deposito_nombre} · total
				{formatearMoneda(ultimaVenta.venta.total)}.
				{ultimaVenta.lineas
					.map((l) => `${l.producto_nombre}: quedan ${l.stock_resultante ?? '—'}`)
					.join(' · ')}
			</Aviso>
		{/if}
		{#if errorVenta}<Aviso variante="error">{errorVenta}</Aviso>{/if}
		{#if carrito.length === 0}
			<p class="resumen">Agregá productos desde los resultados de búsqueda.</p>
		{:else}
			<ul class="carrito">
				{#each carrito as item (item.producto_id)}
					<li>
						<span class="principal">{item.nombre}</span>
						<input
							class="cantidad"
							type="number"
							min="1"
							aria-label="Cantidad de {item.nombre}"
							value={item.cantidad}
							onchange={(e) =>
								(carrito = cambiarCantidad(
									carrito,
									item.producto_id,
									Number((e.currentTarget as HTMLInputElement).value)
								))}
						/>
						<span class="numero">{formatearMoneda(item.precio * item.cantidad)}</span>
						<Boton
							tamano="sm"
							variante="fantasma"
							onclick={() => (carrito = quitarDelCarrito(carrito, item.producto_id))}
						>
							Quitar
						</Boton>
					</li>
				{/each}
			</ul>
			<div class="cierre">
				<Campo etiqueta="Descuento" nombre="descuento" bind:valor={descuentoTexto} />
				<Campo etiqueta="Medio de pago" nombre="medio_pago" tipo="select" opciones={MEDIOS} bind:valor={medioPago} />
				<Campo etiqueta="Cliente" nombre="cliente" tipo="select" opciones={opcionesClientes} bind:valor={clienteId} />
			</div>
			{#if problemaDescuento}<p class="error-campo">{problemaDescuento}</p>{/if}
			<p class="totales">
				Subtotal {formatearMoneda(totales.subtotal)} · descuento {formatearMoneda(totales.descuento)} ·
				<strong>total {formatearMoneda(totales.total)}</strong>
			</p>
			<Boton
				onclick={() => void confirmarVenta()}
				cargando={vendiendo}
				deshabilitado={depositoId === null || Boolean(problemaDescuento)}
			>
				Confirmar venta
			</Boton>
			{#if depositoId === null}
				<p class="resumen">Elegí un depósito para vender.</p>
			{/if}
		{/if}
	</Tarjeta>
</div>

<style>
	.titulo-venta {
		margin: 0 0 8px;
		font-size: 15px;
	}

	.carrito {
		margin: 0 0 10px;
		padding: 0;
		list-style: none;
	}

	.carrito li {
		display: grid;
		grid-template-columns: 1fr 80px 120px auto;
		align-items: center;
		gap: 10px;
		padding: 6px 0;
		border-top: 1px solid var(--borde);
	}

	.cantidad {
		width: 80px;
		padding: 4px 6px;
	}

	.cierre {
		display: grid;
		grid-template-columns: repeat(3, minmax(160px, 1fr));
		gap: 12px;
	}

	.totales {
		margin: 10px 0;
		font-variant-numeric: tabular-nums;
	}

	.error-campo {
		margin: 6px 0 0;
		color: var(--rojo-texto, #b42318);
		font-size: 13px;
	}

	@media (max-width: 720px) {
		.cierre,
		.carrito li {
			grid-template-columns: 1fr;
		}
	}

	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filtros {
		display: grid;
		grid-template-columns: minmax(220px, 2fr) 160px 200px 180px auto;
		gap: 12px;
		align-items: end;
	}

	.acciones-filtro {
		display: flex;
		align-items: center;
		gap: 10px;
		padding-bottom: 2px;
	}

	.checkbox {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--texto-suave);
		font-size: 13px;
		white-space: nowrap;
	}

	.checkbox input {
		width: 15px;
		height: 15px;
		accent-color: var(--primario);
	}

	.resumen {
		min-height: 18px;
		margin: 0;
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	.truncados {
		display: block;
		color: var(--ambar-texto);
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
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	@media (max-width: 980px) {
		.filtros {
			grid-template-columns: 1fr 1fr;
		}
	}

	@media (max-width: 560px) {
		.filtros {
			grid-template-columns: 1fr;
		}

		.acciones-filtro {
			padding-bottom: 0;
		}
	}
</style>
