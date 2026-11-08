<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import { crearTransferencia, listarTransferencias } from '#lib/api/transferencias';
	import { listarDepositosActivos, listarUbicaciones, type DepositoActivo } from '#lib/api/depositos';
	import { buscarProductos } from '#lib/api/productos';
	import type { Ubicacion } from '#lib/schemas/depositos';
	import type { Producto, ListaBusqueda } from '#lib/schemas/productos';
	import type { Transferencia } from '#lib/schemas/transferencias';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import Icono from '#lib/components/ui/Icono.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import TablaDatos from '#lib/components/ui/TablaDatos.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	import type { Columna } from '#lib/components/ui/tabla';
	import BotonEscanear from '#lib/components/codigos/BotonEscanear.svelte';
	import AvisoLectura from '#lib/components/codigos/AvisoLectura.svelte';
	import { leerProducto, type AvisoLectura as AvisoLecturaTipo } from '#lib/codigos/lectura';
	import { sesion } from '#lib/stores/sesion';

	const formatoFecha = new Intl.DateTimeFormat('es-AR', {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	const COLUMNAS: Columna<Transferencia>[] = [
		{ clave: 'id', encabezado: 'ID' },
		{ clave: 'producto_nombre', encabezado: 'Producto' },
		{ clave: 'deposito_origen', encabezado: 'Origen' },
		{ clave: 'deposito_destino', encabezado: 'Destino' },
		{ clave: 'cantidad', encabezado: 'Cantidad', alineacion: 'derecha' },
		{ clave: 'stock_origen_resultante', encabezado: 'Stock origen', alineacion: 'derecha' },
		{ clave: 'stock_destino_resultante', encabezado: 'Stock destino', alineacion: 'derecha' },
		{ clave: 'usuario', encabezado: 'Autor' },
		{ clave: 'creado_en', encabezado: 'Fecha' }
	];

	let depositos = $state<DepositoActivo[]>([]);
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

	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Seleccionar' },
		...depositos.map((deposito) => ({ valor: String(deposito.id), etiqueta: deposito.nombre }))
	]);
	const opcionesOrigen = $derived([
		{ valor: '', etiqueta: 'Sin ubicación' },
		...ubicacionesOrigen.map((ubicacion) => ({
			valor: String(ubicacion.id),
			etiqueta: ubicacion.nombre
		}))
	]);
	const opcionesDestino = $derived([
		{ valor: '', etiqueta: 'Sin ubicación' },
		...ubicacionesDestino.map((ubicacion) => ({
			valor: String(ubicacion.id),
			etiqueta: ubicacion.nombre
		}))
	]);

	async function cargarDepositos() {
		try {
			depositos = await listarDepositosActivos();
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

	async function buscarProducto(texto = productoQuery) {
		if (texto.trim().length < 2) {
			productoResultados = [];
			return;
		}
		try {
			const res: ListaBusqueda = await buscarProductos({ q: texto.trim(), limite: 8 });
			productoResultados = res.items;
		} catch {
			productoResultados = [];
		}
	}

	function elegirProducto(producto: Producto) {
		productoSel = producto;
		productoQuery = `${producto.nombre} (${producto.sku})`;
		productoResultados = [];
	}

	// 020: la lectura elige el producto a transferir; no se escribe hasta confirmar (FR-150).
	let avisoLectura = $state<AvisoLecturaTipo | null>(null);
	const esGestion = $derived(
		$sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'
	);

	async function alLeer(valor: string) {
		avisoLectura = null;
		const { resultado, aviso } = await leerProducto(valor, esGestion);
		avisoLectura = aviso;
		if (resultado?.estado === 'encontrado') {
			elegirProducto(resultado.producto);
		} else {
			productoSel = null;
			productoResultados = [];
			productoQuery = valor;
		}
	}

	function cambiarOrigen() {
		ubicacionOrigenId = '';
		void cargarUbicaciones();
	}

	function cambiarDestino() {
		ubicacionDestinoId = '';
		void cargarUbicaciones();
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
			cantidad = '1';
			observacion = '';
			await cargarTransferencias();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo registrar la transferencia';
		} finally {
			cargando = false;
		}
	}

	function cambiarPagina(nueva: number) {
		pagina = nueva;
		void cargarTransferencias();
	}

	onMount(async () => {
		await cargarDepositos();
		await cargarTransferencias();
	});
</script>

<svelte:head>
	<title>Transferencias · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Transferencias entre depósitos"
		descripcion="Mové stock entre depósitos de forma inmediata y atómica. El stock origen y destino se actualiza en la misma operación."
	/>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if exito}<Aviso variante="exito">{exito}</Aviso>{/if}

	<Tarjeta titulo="Nueva transferencia">
		<div class="grilla-campos">
			<div class="campo-producto">
				<Campo
					etiqueta="Producto"
					nombre="producto"
					marcador="Buscar por nombre o SKU"
					bind:valor={productoQuery}
					oninput={(valor) => {
						productoQuery = valor;
						void buscarProducto(valor);
					}}
				/>
				<BotonEscanear
					tamano="sm"
					etiqueta="Escanear"
					onLectura={(lectura) => void alLeer(lectura.valor)}
				/>
				<AvisoLectura aviso={avisoLectura} onReintentar={(valor) => void alLeer(valor)} />
				{#if productoResultados.length > 0 && !productoSel}
					<ul class="resultados">
						{#each productoResultados as producto (producto.id)}
							<li>
								<button type="button" onclick={() => elegirProducto(producto)}>
									<span>{producto.nombre}</span>
									<span class="resultado-sku">{producto.sku}</span>
								</button>
							</li>
						{/each}
					</ul>
				{/if}
				{#if productoSel}
					<span class="chip">
						{productoSel.nombre}
						<span class="chip-sku">{productoSel.sku}</span>
						<button
							type="button"
							class="chip-quitar"
							aria-label="Quitar producto"
							onclick={() => {
								productoSel = null;
								productoResultados = [];
							}}
						>
							<Icono nombre="cerrar" tamano={12} />
						</button>
					</span>
				{/if}
			</div>

			<Campo
				etiqueta="Cantidad"
				nombre="cantidad"
				tipo="number"
				min={1}
				step={1}
				bind:valor={cantidad}
			/>

			<Campo
				etiqueta="Depósito origen"
				nombre="deposito_origen"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={depositoOrigenId}
				onchange={cambiarOrigen}
			/>
			<Campo
				etiqueta="Ubicación origen (opcional)"
				nombre="ubicacion_origen"
				tipo="select"
				opciones={opcionesOrigen}
				bind:valor={ubicacionOrigenId}
			/>

			<Campo
				etiqueta="Depósito destino"
				nombre="deposito_destino"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={depositoDestinoId}
				onchange={cambiarDestino}
			/>
			<Campo
				etiqueta="Ubicación destino (opcional)"
				nombre="ubicacion_destino"
				tipo="select"
				opciones={opcionesDestino}
				bind:valor={ubicacionDestinoId}
			/>
		</div>

		<Campo
			etiqueta="Observación (opcional)"
			nombre="observacion"
			tipo="textarea"
			maxlength={500}
			bind:valor={observacion}
		/>

		<div class="acciones">
			<Boton cargando={cargando} onclick={() => void enviar()}>Transferir</Boton>
		</div>
	</Tarjeta>

	<Tarjeta titulo="Historial">
		<TablaDatos
			columnas={COLUMNAS}
			filas={transferencias}
			etiqueta="Historial de transferencias"
			cargando={cargando}
		>
			{#snippet fila(t)}
				<td>{t.id}</td>
				<td>
					{t.producto_nombre}
					<span class="sku">{t.sku}</span>
				</td>
				<td>{t.deposito_origen.nombre}</td>
				<td>{t.deposito_destino.nombre}</td>
				<td class="numero">{t.cantidad}</td>
				<td class="numero">{t.stock_origen_resultante}</td>
				<td class="numero">{t.stock_destino_resultante}</td>
				<td>{t.usuario.nombre}</td>
				<td class="fecha">{formatoFecha.format(new Date(t.creado_en))}</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio
					icono="transferencias"
					titulo="No hay transferencias registradas"
					descripcion="Registrá la primera con el formulario de arriba."
				/>
			{/snippet}
		</TablaDatos>

		<div class="pie-tabla">
			<Paginacion {pagina} {total} {porPagina} onCambiar={cambiarPagina} />
		</div>
	</Tarjeta>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.grilla-campos {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
		margin-bottom: 14px;
	}

	.campo-producto {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.campo-producto :global(.boton-escanear) {
		align-self: flex-start;
	}

	.resultados {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		z-index: 15;
		max-height: 180px;
		overflow-y: auto;
		margin: 0;
		padding: 4px;
		list-style: none;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		box-shadow: var(--sombra-elevada);
	}

	.resultados button {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		padding: 7px 8px;
		border: 0;
		border-radius: 4px;
		background: transparent;
		color: var(--texto);
		font-size: 12.5px;
		text-align: left;
		cursor: pointer;
	}

	.resultados button:hover {
		background: var(--superficie-2);
	}

	.resultado-sku {
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
		font-size: 11px;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 6px;
		padding: 2px 6px;
		border-radius: var(--radio-pildora);
		background: var(--primario-suave);
		color: var(--primario-texto);
		font-size: 11.5px;
	}

	.chip-sku {
		font-family: var(--fuente-mono);
		font-size: 10.5px;
		opacity: 0.8;
	}

	.chip-quitar {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border: 0;
		background: transparent;
		color: inherit;
		cursor: pointer;
		padding: 0;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		margin-top: 14px;
	}

	.sku {
		display: block;
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
		font-size: 11px;
	}

	.numero {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	.fecha {
		white-space: nowrap;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.pie-tabla {
		padding: 12px 16px 0;
	}

	@media (max-width: 720px) {
		.grilla-campos {
			grid-template-columns: 1fr;
		}
	}
</style>
