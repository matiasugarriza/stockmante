<script lang="ts">
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	import TarjetaKpi from '#lib/components/ui/TarjetaKpi.svelte';
	import GraficoBarras from '#lib/components/ui/GraficoBarras.svelte';
	import GraficoDona from '#lib/components/ui/GraficoDona.svelte';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Esqueleto from '#lib/components/ui/Esqueleto.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import PanelActividad from '#lib/components/inicio/PanelActividad.svelte';
	import AccesosRapidos from '#lib/components/inicio/AccesosRapidos.svelte';
	import { listarStock } from '#lib/api/stock';
	import { listarMovimientos } from '#lib/api/movimientos';
	import { listarProductos } from '#lib/api/productos';
	import { obtenerStockValorizado } from '#lib/api/reportes';
	import { sesion } from '#lib/stores/sesion';
	import { mensajeError } from '#lib/utils/errores';
	import {
		formatearFechaCorta,
		formatearMoneda,
		formatearNumero
	} from '#lib/utils/formato';
	import type { DatoGrafico } from '#lib/utils/graficos';
	import type { MovimientoDetalle, StockItem, TipoMovimiento } from '#lib/schemas/movimientos';

	const TIPOS: Array<{ tipo: TipoMovimiento; etiqueta: string; color: string }> = [
		{ tipo: 'ENTRADA', etiqueta: 'Entradas', color: 'var(--teal)' },
		{ tipo: 'SALIDA', etiqueta: 'Salidas', color: 'var(--ambar)' },
		{ tipo: 'MERMA', etiqueta: 'Mermas', color: 'var(--peligro)' },
		{ tipo: 'AJUSTE', etiqueta: 'Ajustes', color: 'var(--primario)' }
	];

	const esGestion = $derived(
		$sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'
	);
	const fecha = formatearFechaCorta(new Date().toISOString());

	let bajos = $state<number | null>(null);
	let cargandoBajos = $state(true);
	let errorBajos = $state<string | null>(null);

	let movHoy = $state<number | null>(null);
	let cargandoMovHoy = $state(true);
	let errorMovHoy = $state<string | null>(null);

	let activos = $state<number | null>(null);
	let cargandoActivos = $state(true);
	let errorActivos = $state<string | null>(null);

	let valor = $state<number | null>(null);
	let cargandoValor = $state(true);
	let errorValor = $state<string | null>(null);

	let tipos = $state<DatoGrafico[]>([]);
	let cargandoTipos = $state(true);
	let errorTipos = $state<string | null>(null);

	let estadoStock = $state<DatoGrafico[]>([]);
	let stockTotal = $state<number | null>(null);
	let cargandoEstado = $state(true);
	let errorEstado = $state<string | null>(null);

	let actividad = $state<MovimientoDetalle[]>([]);
	let cargandoActividad = $state(true);
	let errorActividad = $state<string | null>(null);

	let urgentes = $state<StockItem[]>([]);
	let cargandoUrgentes = $state(true);
	let errorUrgentes = $state<string | null>(null);

	const hayBajos = $derived(bajos !== null && bajos > 0);
	const erroresKpi = $derived(
		[errorBajos, errorMovHoy, errorActivos, errorValor].filter(Boolean).join(' ')
	);

	function desdeHoy(): string {
		const inicio = new Date();
		inicio.setHours(0, 0, 0, 0);
		return inicio.toISOString();
	}

	async function cargarBajos() {
		cargandoBajos = true;
		errorBajos = null;
		try {
			const respuesta = await listarStock({ bajoMinimo: true, porPagina: 1 });
			bajos = respuesta.total;
		} catch (error) {
			errorBajos = mensajeError(error, 'No se pudo cargar el stock bajo mínimo.');
		} finally {
			cargandoBajos = false;
		}
	}

	async function cargarMovHoy() {
		cargandoMovHoy = true;
		errorMovHoy = null;
		try {
			const respuesta = await listarMovimientos({ desde: desdeHoy(), porPagina: 1 });
			movHoy = respuesta.total;
		} catch (error) {
			errorMovHoy = mensajeError(error, 'No se pudieron cargar los movimientos de hoy.');
		} finally {
			cargandoMovHoy = false;
		}
	}

	async function cargarActivos() {
		if (!esGestion) return;
		cargandoActivos = true;
		errorActivos = null;
		try {
			const respuesta = await listarProductos({ activo: 'true', porPagina: 1 });
			activos = respuesta.total;
		} catch (error) {
			errorActivos = mensajeError(error, 'No se pudieron cargar los productos activos.');
		} finally {
			cargandoActivos = false;
		}
	}

	async function cargarValor() {
		if (!esGestion) return;
		cargandoValor = true;
		errorValor = null;
		try {
			const reporte = await obtenerStockValorizado();
			valor = reporte.total;
		} catch (error) {
			errorValor = mensajeError(error, 'No se pudo calcular el valor del stock.');
		} finally {
			cargandoValor = false;
		}
	}

	async function cargarTipos() {
		cargandoTipos = true;
		errorTipos = null;
		try {
			const desde = desdeHoy();
			const totales = await Promise.all(
				TIPOS.map((tipo) =>
					listarMovimientos({ tipo: tipo.tipo, desde, porPagina: 1 }).then(
						(respuesta) => respuesta.total
					)
				)
			);
			tipos = TIPOS.map((tipo, indice) => ({
				etiqueta: tipo.etiqueta,
				valor: totales[indice] ?? 0,
				color: tipo.color
			}));
		} catch (error) {
			errorTipos = mensajeError(error, 'No se pudo cargar la distribución de movimientos.');
		} finally {
			cargandoTipos = false;
		}
	}

	async function cargarEstado() {
		cargandoEstado = true;
		errorEstado = null;
		try {
			const [todos, bajoMinimo] = await Promise.all([
				listarStock({ porPagina: 1 }),
				listarStock({ bajoMinimo: true, porPagina: 1 })
			]);
			stockTotal = todos.total;
			const normales = Math.max(0, todos.total - bajoMinimo.total);
			estadoStock = [
				{ etiqueta: 'Bajo mínimo', valor: bajoMinimo.total, color: 'var(--ambar)' },
				{ etiqueta: 'Sobre mínimo', valor: normales, color: 'var(--exito)' }
			];
		} catch (error) {
			errorEstado = mensajeError(error, 'No se pudo cargar el estado del stock.');
		} finally {
			cargandoEstado = false;
		}
	}

	async function cargarActividad() {
		cargandoActividad = true;
		errorActividad = null;
		try {
			const respuesta = await listarMovimientos({ porPagina: 6 });
			actividad = respuesta.items;
		} catch (error) {
			errorActividad = mensajeError(error, 'No se pudieron cargar los movimientos recientes.');
		} finally {
			cargandoActividad = false;
		}
	}

	async function cargarUrgentes() {
		cargandoUrgentes = true;
		errorUrgentes = null;
		try {
			const respuesta = await listarStock({ bajoMinimo: true, porPagina: 5 });
			urgentes = respuesta.items;
		} catch (error) {
			errorUrgentes = mensajeError(error, 'No se pudieron cargar los casos urgentes.');
		} finally {
			cargandoUrgentes = false;
		}
	}

	function cargarTodo() {
		void cargarBajos();
		void cargarMovHoy();
		void cargarTipos();
		void cargarEstado();
		void cargarActividad();
		void cargarUrgentes();
		if (esGestion) {
			void cargarActivos();
			void cargarValor();
		}
	}

	let iniciado = false;
	$effect(() => {
		if (iniciado || $sesion.estado !== 'activa') return;
		iniciado = true;
		cargarTodo();
	});
</script>

<svelte:head>
	<title>Inicio · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Inicio"
		descripcion={`Resumen del inventario al ${fecha}.`}
	>
		<Boton href="/caja" icono="caja">Ir a Caja</Boton>
	</EncabezadoPagina>

	{#if erroresKpi}
		<Aviso variante="error" titulo="Algunos indicadores no se pudieron cargar">
			{erroresKpi}
		</Aviso>
	{/if}

	<div class="grid-kpis" role="group" aria-label="Indicadores del inventario">
		<TarjetaKpi
			etiqueta="Stock bajo mínimo"
			valor={bajos === null ? '—' : formatearNumero(bajos)}
			contexto={hayBajos ? 'Productos que necesitan reposición' : 'Todo el stock sobre el mínimo'}
			icono="alerta"
			variante={hayBajos ? 'aviso' : 'exito'}
			cargando={cargandoBajos}
			plano
		/>
		<TarjetaKpi
			etiqueta="Movimientos de hoy"
			valor={movHoy === null ? '—' : formatearNumero(movHoy)}
			contexto="Entradas, salidas, mermas y ajustes"
			icono="movimientos"
			variante="info"
			cargando={cargandoMovHoy}
			plano
		/>
		{#if esGestion}
			<TarjetaKpi
				etiqueta="Productos activos"
				valor={activos === null ? '—' : formatearNumero(activos)}
				contexto="Catálogo disponible para operar"
				icono="productos"
				variante="neutro"
				cargando={cargandoActivos}
				plano
			/>
			<TarjetaKpi
				etiqueta="Valor del stock"
				valor={valor === null ? '—' : formatearMoneda(valor)}
				contexto="Costo del inventario activo"
				icono="stock"
				variante="exito"
				cargando={cargandoValor}
				plano
			/>
		{/if}
	</div>

	<div class="grid-graficos">
		<Tarjeta titulo="Movimientos por tipo" descripcion="Desde el inicio del día">
			{#if cargandoTipos}
				<Esqueleto alto="230px" />
			{:else if errorTipos}
				<Aviso variante="error" titulo="No se pudo cargar el gráfico">{errorTipos}</Aviso>
			{:else}
				<GraficoBarras datos={tipos} etiqueta="Movimientos del día por tipo" />
			{/if}
		</Tarjeta>

		<Tarjeta titulo="Estado del stock" descripcion="Productos bajo mínimo frente al total">
			{#if cargandoEstado}
				<Esqueleto alto="230px" />
			{:else if errorEstado}
				<Aviso variante="error" titulo="No se pudo cargar el gráfico">{errorEstado}</Aviso>
			{:else}
				<GraficoDona
					datos={estadoStock}
					etiqueta="Distribución de productos según stock mínimo"
					centro={stockTotal === null ? undefined : formatearNumero(stockTotal)}
				/>
			{/if}
		</Tarjeta>
	</div>

	<div class="grid-inferior">
		<Tarjeta titulo="Actividad reciente" descripcion="Últimos movimientos registrados">
			<PanelActividad
				movimientos={actividad}
				cargando={cargandoActividad}
				error={errorActividad}
				onreintentar={() => void cargarActividad()}
			/>
		</Tarjeta>

		<Tarjeta titulo="Casos urgentes" descripcion="Productos por debajo del mínimo">
			{#if cargandoUrgentes}
				<div class="cargando-urgentes">
					{#each Array(4) as _, indice (indice)}
						<Esqueleto alto="34px" />
					{/each}
				</div>
			{:else if errorUrgentes}
				<Aviso variante="error" titulo="No se pudieron cargar los casos urgentes">
					{errorUrgentes}
				</Aviso>
			{:else if urgentes.length === 0}
				<EstadoVacio
					icono="check"
					titulo="Sin alertas de stock"
					descripcion="Todos los productos están sobre su mínimo."
				/>
			{:else}
				<ul class="urgentes">
					{#each urgentes as item (item.producto_id)}
						<li>
							<a href={esGestion ? '/admin/stock' : '/caja'} class="urgente">
								<span class="datos">
									<strong>{item.nombre}</strong>
									<small>{item.sku} · mínimo {formatearNumero(item.stock_minimo)}</small>
								</span>
								<span class="stock">{formatearNumero(item.stock_actual)}</span>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</Tarjeta>
	</div>

	<Tarjeta titulo="Accesos rápidos" descripcion="Atajos a las tareas más frecuentes">
		<AccesosRapidos {esGestion} />
	</Tarjeta>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.grid-kpis {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 14px 24px;
		padding: 4px 0 14px;
		border-bottom: 1px solid var(--borde);
	}

	.grid-graficos,
	.grid-inferior {
		display: grid;
		grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
		gap: 16px;
		align-items: start;
	}

	.urgentes {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.urgente {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 9px 0;
		border-bottom: 1px solid var(--borde);
		color: var(--texto);
		text-decoration: none;
	}

	.urgentes li:last-child .urgente {
		border-bottom: 0;
	}

	.urgente:hover strong {
		color: var(--primario-texto);
	}

	.datos {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.datos strong {
		font-size: 13px;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.datos small {
		color: var(--texto-tenue);
		font-size: 11.5px;
	}

	.stock {
		flex: none;
		padding: 2px 8px;
		border-radius: var(--radio-pildora);
		background: var(--ambar-suave);
		color: var(--ambar-texto);
		font-size: 12px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}

	.cargando-urgentes {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	@media (max-width: 1180px) {
		.grid-kpis {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}

		.grid-graficos,
		.grid-inferior {
			grid-template-columns: minmax(0, 1fr);
		}
	}

	@media (max-width: 560px) {
		.grid-kpis {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
