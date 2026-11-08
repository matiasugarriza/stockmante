<script lang="ts">
	import { ApiError } from '#lib/api/client';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import ListaTarjetas from '#lib/components/movil/ListaTarjetas.svelte';
	import TarjetaRegistro from '#lib/components/movil/TarjetaRegistro.svelte';
	import BotonFlotante from '#lib/components/movil/BotonFlotante.svelte';
	import { crearMovimiento, listarMovimientos } from '#lib/api/movimientos';
	import { listarDepositosActivos, listarUbicaciones, type DepositoActivo } from '#lib/api/depositos';
	import { buscarProductos } from '#lib/api/productos';
	import {
		type LineaNuevaInput,
		type MovimientoDetalle,
		type MotivoMovimiento,
		type TipoMovimiento
	} from '#lib/schemas/movimientos';
	import { MOTIVOS_POR_TIPO, motivoPorDefecto } from '#lib/schemas/movimientos';
	import type { Ubicacion } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import EtiquetaEstado from '#lib/components/ui/EtiquetaEstado.svelte';
	import Icono from '#lib/components/ui/Icono.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import TablaDatos from '#lib/components/ui/TablaDatos.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	import type { Columna } from '#lib/components/ui/tabla';
	import BotonEscanear from '#lib/components/codigos/BotonEscanear.svelte';
	import AvisoLectura from '#lib/components/codigos/AvisoLectura.svelte';
	import { leerProducto, type AvisoLectura as AvisoLecturaTipo } from '#lib/codigos/lectura';

	const TIPOS: { valor: TipoMovimiento; etiqueta: string }[] = [
		{ valor: 'ENTRADA', etiqueta: 'Entrada' },
		{ valor: 'SALIDA', etiqueta: 'Salida' },
		{ valor: 'MERMA', etiqueta: 'Merma' },
		{ valor: 'AJUSTE', etiqueta: 'Ajuste' }
	];

	const etiquetaMotivo = (valor: MotivoMovimiento) =>
		valor.charAt(0) + valor.slice(1).toLowerCase();

	const VARIANTES_TIPO: Record<
		TipoMovimiento,
		'exito' | 'info' | 'aviso' | 'peligro' | 'neutro'
	> = {
		ENTRADA: 'exito',
		SALIDA: 'info',
		MERMA: 'peligro',
		AJUSTE: 'neutro'
	};

	const COLUMNAS: Columna<MovimientoDetalle>[] = [
		{ clave: 'id', encabezado: '#' },
		{ clave: 'creado_en', encabezado: 'Fecha' },
		{ clave: 'tipo', encabezado: 'Tipo' },
		{ clave: 'motivo', encabezado: 'Motivo' },
		{ clave: 'deposito', encabezado: 'Depósito' },
		{ clave: 'usuario', encabezado: 'Autor' },
		{ clave: 'lineas', encabezado: 'Líneas' }
	];

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

	let depositos = $state<DepositoActivo[]>([]);
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
	const pideObservacion = $derived(motivo === 'OTRO' || tipo === 'AJUSTE');

	const opcionesTipo = $derived(TIPOS.filter((opcion) => opcion.valor !== 'AJUSTE' || esGestion));
	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Elegí…' },
		...depositos.map((deposito) => ({ valor: String(deposito.id), etiqueta: deposito.nombre }))
	]);
	const opcionesUbicaciones = $derived([
		{ valor: '', etiqueta: 'Sin ubicación' },
		...ubicaciones.map((ubicacion) => ({
			valor: String(ubicacion.id),
			etiqueta: ubicacion.nombre
		}))
	]);
	const opcionesFiltroTipo = $derived([{ valor: '', etiqueta: 'Todos' }, ...TIPOS]);

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

	// El selector solo ofrece los motivos admitidos para el tipo (matriz de 007).
	const motivos = $derived(
		MOTIVOS_POR_TIPO[tipo].map((valor) => ({ valor, etiqueta: etiquetaMotivo(valor) }))
	);

	// Al cambiar el tipo, el motivo vuelve a una opción coherente.
	$effect(() => {
		if (!MOTIVOS_POR_TIPO[tipo].includes(motivo)) {
			motivo = motivoPorDefecto(tipo);
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
			depositos = await listarDepositosActivos();
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

	// 020: la lectura elige el producto de esa línea; no registra nada hasta confirmar (FR-150).
	let avisoLectura = $state<{ index: number; aviso: AvisoLecturaTipo } | null>(null);

	async function leerEnLinea(index: number, valor: string) {
		avisoLectura = null;
		const { resultado, aviso } = await leerProducto(valor, esGestion);
		if (aviso) avisoLectura = { index, aviso };
		if (!lineas[index]) return;
		if (resultado?.estado === 'encontrado') {
			elegirProducto(index, resultado.producto);
		} else {
			lineas[index] = { ...lineas[index], producto: null, resultados: [], busqueda: valor };
		}
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
			return tipo === 'AJUSTE'
				? 'Un ajuste requiere una justificación en la observación.'
				: 'Con motivo OTRO hace falta una observación.';
		for (const [i, linea] of lineas.entries()) {
			if (!linea.producto) return `Elegí un producto en la línea ${i + 1}.`;
			if (deltaDe(linea) === null)
				return `La cantidad de la línea ${i + 1} debe ser un entero mayor que cero.`;
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

<div class="pagina">
	<EncabezadoPagina
		titulo="Movimientos de stock"
		descripcion="Registrá entradas, salidas, mermas y ajustes. El stock se actualiza en una única transacción."
	/>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}

	<Tarjeta titulo="Registrar movimiento">
		<div class="formulario">
			<div class="grilla-campos">
				<Campo
					etiqueta="Tipo"
					nombre="tipo"
					tipo="select"
					opciones={opcionesTipo}
					bind:valor={tipo}
				/>
				<Campo
					etiqueta="Depósito"
					nombre="deposito"
					tipo="select"
					opciones={opcionesDepositos}
					bind:valor={depositoId}
				/>
				<Campo
					etiqueta="Motivo"
					nombre="motivo"
					tipo="select"
					opciones={motivos}
					bind:valor={motivo}
				/>
				<Campo
					etiqueta="Observación"
					nombre="observacion"
					marcador={pideObservacion ? 'Obligatoria para AJUSTE u OTRO' : 'Opcional'}
					maxlength={500}
					bind:valor={observacion}
				/>
			</div>

			<div class="lineas">
				{#each lineas as linea, index (index)}
					<div class="linea">
						<div class="linea-campos">
							<div class="campo-producto">
								<Campo
									etiqueta={`Producto${lineas.length > 1 ? ` ${index + 1}` : ''}`}
									nombre={`producto-${index}`}
									marcador="Nombre o SKU"
									bind:valor={linea.busqueda}
									oninput={(valor) => buscarEnLinea(index, valor)}
								/>
								<BotonEscanear
									tamano="sm"
									etiqueta="Escanear"
									onLectura={(lectura) => void leerEnLinea(index, lectura.valor)}
								/>
								{#if linea.resultados.length > 0 && !linea.producto}
									<ul class="resultados">
										{#each linea.resultados as resultado (resultado.id)}
											<li>
												<button
													type="button"
													onclick={() => elegirProducto(index, resultado)}
												>
													<span class="resultado-nombre">{resultado.nombre}</span>
													<span class="resultado-sku">{resultado.sku}</span>
												</button>
											</li>
										{/each}
									</ul>
								{/if}
								{#if linea.producto}
									<span class="chip">
										{linea.producto.nombre}
										<span class="chip-sku">{linea.producto.sku}</span>
										<button
											type="button"
											class="chip-quitar"
											aria-label="Quitar producto"
											onclick={() => (lineas[index] = { ...nuevaLinea(), busqueda: linea.busqueda })}
										>
											<Icono nombre="cerrar" tamano={12} />
										</button>
									</span>
								{/if}
								{#if avisoLectura?.index === index}
									<AvisoLectura
										aviso={avisoLectura.aviso}
										onReintentar={(valor) => void leerEnLinea(index, valor)}
									/>
								{/if}
							</div>

							<Campo
								etiqueta="Cantidad"
								nombre={`cantidad-${index}`}
								tipo="number"
								min={1}
								step={1}
								bind:valor={linea.cantidad}
							/>

							<Campo
								etiqueta="Ubicación (opcional)"
								nombre={`ubicacion-${index}`}
								tipo="select"
								opciones={opcionesUbicaciones}
								bind:valor={linea.ubicacionId}
							/>

							{#if lineas.length > 1}
								<Boton
									variante="fantasma"
									tamano="sm"
									icono="basura"
									onclick={() => quitarLinea(index)}
								>
									Quitar
								</Boton>
							{/if}
						</div>

						{#if linea.producto && deltaDe(linea) !== null}
							<p class="vista-previa">
								{tipo}: <strong>{signoDelta(deltaDe(linea)!)}</strong> × {linea.producto.nombre}
							</p>
						{/if}
					</div>
				{/each}
			</div>

			<div class="acciones">
				<Boton
					variante="secundario"
					icono="mas"
					deshabilitado={lineas.length >= 200}
					onclick={agregarLinea}
				>
					Agregar línea
				</Boton>
				<Boton cargando={guardando} onclick={() => void enviar()}>
					{guardando ? 'Registrando…' : 'Registrar movimiento'}
				</Boton>
			</div>
		</div>
	</Tarjeta>

	<Tarjeta titulo="Historial">
		<div class="filtros">
			<Campo
				etiqueta="Tipo"
				nombre="filtro-tipo"
				tipo="select"
				opciones={opcionesFiltroTipo}
				bind:valor={filtroTipo}
			/>
			<Campo
				etiqueta="Depósito"
				nombre="filtro-deposito"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={filtroDeposito}
			/>
			<Boton variante="secundario" tamano="sm" onclick={aplicarFiltros}>Filtrar</Boton>
		</div>

		{#if $dispositivo.esMovil}
			<ListaTarjetas items={movimientos} etiqueta="Historial de movimientos" {cargando}>
				{#snippet tarjeta(movimiento)}
					<TarjetaRegistro
						titulo={`${movimiento.tipo} #${movimiento.id}`}
						subtitulo={formatoFecha.format(new Date(movimiento.creado_en))}
						badge={{ texto: movimiento.tipo, variante: VARIANTES_TIPO[movimiento.tipo] }}
						campos={[
							{ etiqueta: 'Motivo', valor: movimiento.motivo },
							{ etiqueta: 'Depósito', valor: movimiento.deposito.nombre },
							{ etiqueta: 'Autor', valor: movimiento.usuario.nombre },
							{ etiqueta: 'Líneas', valor: String(movimiento.lineas.length) }
						]}
					/>
				{/snippet}
				{#snippet vacio()}
					<EstadoVacio
						icono="movimientos"
						titulo="No hay movimientos registrados"
						descripcion="Registrá el primero con el formulario de arriba."
					/>
				{/snippet}
			</ListaTarjetas>
		{:else}
		<TablaDatos
			columnas={COLUMNAS}
			filas={movimientos}
			etiqueta="Historial de movimientos"
			cargando={cargando}
		>
			{#snippet fila(movimiento)}
				<td>{movimiento.id}</td>
				<td class="fecha">{formatoFecha.format(new Date(movimiento.creado_en))}</td>
				<td>
					<EtiquetaEstado variante={VARIANTES_TIPO[movimiento.tipo]}>
						{movimiento.tipo}
					</EtiquetaEstado>
				</td>
				<td>
					{movimiento.motivo}
					{#if movimiento.observacion}
						<span class="observacion">{movimiento.observacion}</span>
					{/if}
				</td>
				<td>{movimiento.deposito.nombre}</td>
				<td>{movimiento.usuario.nombre}</td>
				<td>
					<ul class="lineas-tabla">
						{#each movimiento.lineas as linea (linea.producto_id)}
							<li>
								<span>{linea.producto_nombre}</span>
								<span class="sku">({linea.sku})</span>
								<strong class:positivo={linea.cantidad_delta > 0}>
									{signoDelta(linea.cantidad_delta)}
								</strong>
								<span class="stock">→ {linea.stock_resultante}</span>
							</li>
						{/each}
					</ul>
				</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio
					icono="movimientos"
					titulo="No hay movimientos registrados"
					descripcion="Registrá el primero con el formulario de arriba."
				/>
			{/snippet}
		</TablaDatos>
		{/if}

		{#if totalPaginas > 1}
			<div class="pie-tabla">
				<Paginacion {pagina} {total} {porPagina} onCambiar={cambiarPagina} />
			</div>
		{/if}
	</Tarjeta>

	{#if depositoActual && ubicaciones.length === 0}
		<Aviso variante="info">
			{depositoActual.nombre} no tiene ubicaciones cargadas: los movimientos se imputan al depósito
			hasta que se asignen ubicaciones al producto.
		</Aviso>
	{/if}
	{#if $dispositivo.esMovil}
		<BotonFlotante etiqueta="Nuevo movimiento" icono="movimientos" onclick={() => window.scrollTo({ top: 0, behavior: 'smooth' })} />
	{/if}
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.formulario {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.grilla-campos {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 12px;
	}

	.lineas {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.linea {
		display: flex;
		flex-direction: column;
		gap: 8px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		padding: 12px;
		background: var(--superficie-2);
	}

	.linea-campos {
		display: flex;
		align-items: flex-end;
		gap: 10px;
		flex-wrap: wrap;
	}

	.campo-producto {
		flex: 1;
		min-width: 240px;
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

	.vista-previa {
		margin: 0;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.vista-previa strong {
		color: var(--texto);
		font-variant-numeric: tabular-nums;
	}

	.acciones {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.filtros {
		display: flex;
		align-items: flex-end;
		gap: 10px;
		flex-wrap: wrap;
		margin-bottom: 12px;
	}

	.fecha {
		white-space: nowrap;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.observacion {
		display: block;
		color: var(--texto-tenue);
		font-size: 11.5px;
	}

	.lineas-tabla {
		margin: 0;
		padding: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.lineas-tabla li {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		font-size: 11.5px;
	}

	.lineas-tabla .sku {
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
	}

	.lineas-tabla strong {
		color: var(--peligro-texto);
		font-variant-numeric: tabular-nums;
	}

	.lineas-tabla strong.positivo {
		color: var(--exito-texto);
	}

	.lineas-tabla .stock {
		color: var(--texto-tenue);
	}

	.pie-tabla {
		padding: 12px 16px 0;
	}

	@media (max-width: 1000px) {
		.grilla-campos {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	@media (max-width: 640px) {
		.grilla-campos {
			grid-template-columns: 1fr;
		}

		.campo-producto {
			min-width: 100%;
		}
	}
</style>
