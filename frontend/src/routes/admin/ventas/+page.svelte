<script lang="ts">
	import { onMount } from 'svelte';
	import { listarDepositosActivos, type DepositoActivo } from '#lib/api/depositos';
	import { listarClientes } from '#lib/api/clientes';
	import { anularVenta, listarVentas, obtenerVenta } from '#lib/api/ventas';
	import type { Cliente } from '#lib/schemas/clientes';
	import type { EstadoVenta, Venta, VentaDetalle } from '#lib/schemas/ventas';
	import { mensajeError } from '#lib/utils/errores';
	import { formatearMoneda } from '#lib/utils/formato';
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

	const COLUMNAS: Columna<Venta>[] = [
		{ clave: 'numero', encabezado: 'Número' },
		{ clave: 'creado_en', encabezado: 'Fecha' },
		{ clave: 'deposito_nombre', encabezado: 'Depósito' },
		{ clave: 'cliente_nombre', encabezado: 'Cliente' },
		{ clave: 'medio_pago', encabezado: 'Medio de pago' },
		{ clave: 'total', encabezado: 'Total', alineacion: 'derecha' },
		{ clave: 'estado', encabezado: 'Estado' }
	];
	const ESTADOS = [
		{ valor: '', etiqueta: 'Todos' },
		{ valor: 'CONFIRMADA', etiqueta: 'Confirmadas' },
		{ valor: 'ANULADA', etiqueta: 'Anuladas' }
	];
	const porPagina = 20;

	let depositos = $state<DepositoActivo[]>([]);
	let clientes = $state<Cliente[]>([]);
	let depositoId = $state('');
	let clienteId = $state('');
	let estado = $state('');
	let desde = $state('');
	let hasta = $state('');
	let items = $state<Venta[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	let cargando = $state(false);
	let error = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let detalle = $state<VentaDetalle | null>(null);

	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Todos' },
		...depositos.map((d) => ({ valor: String(d.id), etiqueta: d.nombre }))
	]);
	const opcionesClientes = $derived([
		{ valor: '', etiqueta: 'Todos' },
		...clientes.map((c) => ({ valor: String(c.id), etiqueta: c.nombre }))
	]);

	async function cargar() {
		cargando = true;
		error = null;
		try {
			const res = await listarVentas({
				depositoId: depositoId ? Number(depositoId) : undefined,
				clienteId: clienteId ? Number(clienteId) : undefined,
				estado: (estado || undefined) as EstadoVenta | undefined,
				desde: desde || undefined,
				hasta: hasta || undefined,
				pagina,
				porPagina
			});
			items = res.items;
			total = res.total;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar las ventas.');
		} finally {
			cargando = false;
		}
	}

	function aplicar() {
		pagina = 1;
		void cargar();
	}

	async function ver(venta: Venta) {
		try {
			detalle = await obtenerVenta(venta.id);
		} catch (e) {
			error = mensajeError(e, 'No se pudo abrir la venta.');
		}
	}

	async function anular(venta: Venta) {
		if (!confirm(`¿Anular la venta ${venta.numero}? Se repondrá el stock vendido.`)) return;
		error = null;
		aviso = null;
		try {
			const r = await anularVenta(venta.id);
			aviso = `Venta ${r.venta.numero} anulada. ${r.lineas
				.map((l) => `${l.producto_nombre}: quedan ${l.stock_resultante ?? '—'}`)
				.join(' · ')}`;
			detalle = r;
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo anular la venta.');
		}
	}

	onMount(async () => {
		try {
			[depositos, clientes] = await Promise.all([
				listarDepositosActivos(),
				listarClientes({ porPagina: 100 }).then((r) => r.items)
			]);
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los filtros.');
		}
		await cargar();
	});
</script>

<svelte:head>
	<title>Ventas · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina titulo="Ventas" descripcion="Consultá las ventas de caja y anulá las que correspondan." />

	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}
	{#if error}<Aviso variante="error">{error}</Aviso>{/if}

	<Tarjeta>
		<div class="filtros">
			<Campo etiqueta="Depósito" nombre="deposito" tipo="select" opciones={opcionesDepositos} bind:valor={depositoId} onchange={aplicar} />
			<Campo etiqueta="Cliente" nombre="cliente" tipo="select" opciones={opcionesClientes} bind:valor={clienteId} onchange={aplicar} />
			<Campo etiqueta="Estado" nombre="estado" tipo="select" opciones={ESTADOS} bind:valor={estado} onchange={aplicar} />
			<Campo etiqueta="Desde" nombre="desde" tipo="date" bind:valor={desde} onchange={aplicar} />
			<Campo etiqueta="Hasta" nombre="hasta" tipo="date" bind:valor={hasta} onchange={aplicar} />
		</div>
	</Tarjeta>

	<Tarjeta densa={true}>
		<TablaDatos columnas={COLUMNAS} filas={items} etiqueta="Ventas" {cargando}>
			{#snippet fila(venta)}
				<td>
					<button type="button" class="enlace" onclick={() => void ver(venta)}>{venta.numero}</button>
				</td>
				<td>{new Date(venta.creado_en).toLocaleString()}</td>
				<td>{venta.deposito_nombre}</td>
				<td>{venta.cliente_nombre ?? 'Consumidor final'}</td>
				<td>{venta.medio_pago}</td>
				<td class="numero">{formatearMoneda(venta.total)}</td>
				<td>
					<EtiquetaEstado variante={venta.estado === 'CONFIRMADA' ? 'exito' : 'neutro'}>
						{venta.estado === 'CONFIRMADA' ? 'Confirmada' : 'Anulada'}
					</EtiquetaEstado>
					{#if venta.estado === 'CONFIRMADA'}
						<Boton tamano="sm" variante="peligro" onclick={() => void anular(venta)}>Anular</Boton>
					{/if}
				</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio icono="buscar" titulo="Sin ventas" descripcion="No hay ventas con esos filtros." />
			{/snippet}
		</TablaDatos>
	</Tarjeta>

	<Paginacion {pagina} {total} {porPagina} onCambiar={(n) => { pagina = n; void cargar(); }} />

	{#if detalle}
		<Tarjeta>
			<div class="cabecera">
				<h2>Venta {detalle.venta.numero}</h2>
				<Boton tamano="sm" variante="fantasma" onclick={() => (detalle = null)}>Cerrar</Boton>
			</div>
			<ul class="lineas">
				{#each detalle.lineas as l (l.id)}
					<li>
						<span>{l.cantidad} × {l.producto_nombre}</span>
						<span class="numero">{formatearMoneda(l.subtotal)}</span>
					</li>
				{/each}
			</ul>
			<p class="numero">
				Subtotal {formatearMoneda(detalle.venta.subtotal)} · descuento {formatearMoneda(detalle.venta.descuento)} ·
				<strong>total {formatearMoneda(detalle.venta.total)}</strong>
			</p>
		</Tarjeta>
	{/if}
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filtros {
		display: grid;
		grid-template-columns: repeat(5, minmax(140px, 1fr));
		gap: 12px;
	}

	.enlace {
		border: 0;
		background: none;
		padding: 0;
		color: var(--primario);
		font: inherit;
		cursor: pointer;
	}

	.numero {
		font-variant-numeric: tabular-nums;
	}

	.cabecera {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.cabecera h2 {
		margin: 0;
		font-size: 15px;
	}

	.lineas {
		margin: 8px 0;
		padding: 0;
		list-style: none;
	}

	.lineas li {
		display: flex;
		justify-content: space-between;
		padding: 6px 0;
		border-top: 1px solid var(--borde);
	}

	@media (max-width: 900px) {
		.filtros {
			grid-template-columns: 1fr 1fr;
		}
	}
</style>
