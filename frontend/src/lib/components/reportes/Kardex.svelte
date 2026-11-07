<script lang="ts">
	import { onMount } from 'svelte';
	import { listarDepositosActivos, type DepositoActivo } from '#lib/api/depositos';
	import { buscarProductos } from '#lib/api/productos';
	import { obtenerKardex, urlCsv } from '#lib/api/reportes';
	import type { Producto } from '#lib/schemas/productos';
	import type { Kardex } from '#lib/schemas/reportes';
	import { mensajeError } from '#lib/utils/errores';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';

	let busqueda = $state('');
	let candidatos = $state<Producto[]>([]);
	let producto = $state<Producto | null>(null);
	let depositos = $state<DepositoActivo[]>([]);
	let depositoId = $state('');
	let desde = $state('');
	let hasta = $state('');
	let reporte = $state<Kardex | null>(null);
	let error = $state<string | null>(null);
	let cargando = $state(false);

	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Todos' },
		...depositos.map((d) => ({ valor: String(d.id), etiqueta: d.nombre }))
	]);
	const parametros = $derived({
		productoId: producto?.id ?? 0,
		depositoId: depositoId ? Number(depositoId) : undefined,
		desde: desde || undefined,
		hasta: hasta || undefined
	});

	async function buscar() {
		if (busqueda.trim().length < 2) return;
		try {
			candidatos = (await buscarProductos({ q: busqueda, campo: 'todos', limite: 10 })).items;
		} catch (e) {
			error = mensajeError(e, 'No se pudo buscar el producto.');
		}
	}

	async function consultar() {
		if (!producto) return;
		cargando = true;
		error = null;
		try {
			reporte = await obtenerKardex(parametros);
		} catch (e) {
			reporte = null;
			error = mensajeError(e, 'No se pudo obtener el kardex.');
		} finally {
			cargando = false;
		}
	}

	onMount(async () => {
		try {
			depositos = await listarDepositosActivos();
		} catch {
			depositos = [];
		}
	});
</script>

<div class="filtros">
	<Campo
		etiqueta="Producto"
		nombre="producto"
		tipo="search"
		marcador="Nombre o SKU"
		bind:valor={busqueda}
		onkeydown={(e) => {
			if (e.key === 'Enter') void buscar();
		}}
	/>
	<Campo etiqueta="Depósito" nombre="deposito" tipo="select" opciones={opcionesDepositos} bind:valor={depositoId} />
	<Campo etiqueta="Desde" nombre="desde" tipo="date" bind:valor={desde} />
	<Campo etiqueta="Hasta" nombre="hasta" tipo="date" bind:valor={hasta} />
	<Boton onclick={() => void consultar()} cargando={cargando} deshabilitado={!producto}>Ver kardex</Boton>
</div>

{#if candidatos.length > 0 && !producto}
	<ul class="candidatos">
		{#each candidatos as c (c.id)}
			<li>
				<button type="button" onclick={() => { producto = c; candidatos = []; }}>{c.nombre} · {c.sku}</button>
			</li>
		{/each}
	</ul>
{/if}
{#if producto}
	<p class="elegido">
		Producto: <strong>{producto.nombre}</strong>
		<button type="button" class="cambiar" onclick={() => { producto = null; reporte = null; }}>cambiar</button>
	</p>
{/if}

{#if error}<Aviso variante="error">{error}</Aviso>{/if}

{#if reporte}
	<p class="resumen">
		Saldo inicial <strong>{reporte.saldo_inicial}</strong> · saldo final <strong>{reporte.saldo_final}</strong>
		· <a href={urlCsv('kardex', parametros)}>Descargar CSV</a>
	</p>
	<table class="tabla">
		<thead>
			<tr><th>Fecha</th><th>Tipo</th><th>Motivo</th><th>Depósito</th><th class="num">Cantidad</th><th class="num">Saldo</th></tr>
		</thead>
		<tbody>
			{#each reporte.lineas as l (`${l.movimiento_id}-${l.saldo}`)}
				<tr>
					<td>{new Date(l.creado_en).toLocaleString()}</td>
					<td>{l.tipo}</td>
					<td>{l.motivo}</td>
					<td>{l.deposito_nombre}</td>
					<td class="num">{l.cantidad}</td>
					<td class="num">{l.saldo}</td>
				</tr>
			{:else}
				<tr><td colspan="6">Sin movimientos en el período.</td></tr>
			{/each}
		</tbody>
	</table>
{/if}

<style>
	.filtros {
		display: grid;
		grid-template-columns: 2fr 1fr 1fr 1fr auto;
		gap: 12px;
		align-items: end;
	}

	.candidatos {
		margin: 8px 0;
		padding: 0;
		list-style: none;
	}

	.candidatos button,
	.cambiar {
		border: 0;
		background: none;
		color: var(--primario);
		cursor: pointer;
		font: inherit;
		padding: 2px 0;
	}

	.tabla {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	.tabla th,
	.tabla td {
		padding: 6px 8px;
		border-top: 1px solid var(--borde);
		text-align: left;
	}

	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	@media (max-width: 900px) {
		.filtros {
			grid-template-columns: 1fr;
		}
	}
</style>
