<script lang="ts">
	import { ApiError } from '#lib/api/client';
	import { importarProductos, type ModoImportacion } from '#lib/api/importaciones';
	import type { DetalleImportacion, ResumenImportacion } from '#lib/schemas/importaciones';
	import { mensajeError } from '#lib/utils/errores';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	let archivo = $state<File | null>(null);
	let resumen = $state<ResumenImportacion | null>(null);
	let errores = $state<DetalleImportacion[]>([]);
	let mensaje = $state<string | null>(null);
	let aviso = $state<string | null>(null);
	let trabajando = $state<ModoImportacion | null>(null);

	const puedeAplicar = $derived(
		archivo !== null && resumen !== null && !resumen.aplicado && resumen.con_error === 0
	);

	function elegir(event: Event) {
		const entrada = event.currentTarget as HTMLInputElement;
		archivo = entrada.files?.[0] ?? null;
		resumen = null;
		errores = [];
		mensaje = null;
		aviso = null;
	}

	async function ejecutar(modo: ModoImportacion) {
		if (!archivo) return;
		trabajando = modo;
		mensaje = null;
		aviso = null;
		try {
			resumen = await importarProductos(archivo, modo);
			errores = resumen.errores;
			if (resumen.aplicado) {
				aviso = `Se crearon ${resumen.creados} productos con su stock inicial.`;
			}
		} catch (e) {
			if (e instanceof ApiError && e.detalles) {
				errores = e.detalles;
			}
			mensaje = mensajeError(e, 'No se pudo procesar el archivo.');
		} finally {
			trabajando = null;
		}
	}
</script>

<svelte:head>
	<title>Importar productos · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Importar productos"
		descripcion="Subí un CSV (coma o punto y coma, hasta 5.000 filas y 5 MB). Primero validá: aplicar es todo o nada."
	/>

	<Tarjeta>
		<label class="archivo" for="archivo-csv">Archivo CSV</label>
		<input id="archivo-csv" type="file" accept=".csv,text/csv" onchange={elegir} />
		<p class="ayuda">
			Columnas: <code>nombre, sku, categoria, unidad_medida, costo, precio_venta, stock_minimo</code>
			y opcionales <code>codigo_barras, stock_inicial, deposito</code> (vacío = «Principal»).
		</p>
		<div class="acciones">
			<Boton
				variante="secundario"
				onclick={() => void ejecutar('validar')}
				cargando={trabajando === 'validar'}
				deshabilitado={!archivo}
			>
				Validar
			</Boton>
			<Boton
				onclick={() => void ejecutar('aplicar')}
				cargando={trabajando === 'aplicar'}
				deshabilitado={!puedeAplicar}
			>
				Aplicar
			</Boton>
		</div>
		{#if archivo && !resumen}<p class="ayuda">Validá el archivo para habilitar «Aplicar».</p>{/if}
	</Tarjeta>

	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}
	{#if mensaje}<Aviso variante="error">{mensaje}</Aviso>{/if}

	{#if resumen}
		<Tarjeta>
			<dl class="resumen" aria-live="polite">
				<div><dt>Filas leídas</dt><dd>{resumen.filas_leidas}</dd></div>
				<div><dt>Válidas</dt><dd>{resumen.validas}</dd></div>
				<div><dt>Con error</dt><dd>{resumen.con_error}</dd></div>
				<div><dt>Creados</dt><dd>{resumen.creados}</dd></div>
			</dl>
		</Tarjeta>
	{/if}

	{#if errores.length > 0}
		<Tarjeta densa={true}>
			<table class="tabla" aria-label="Errores de la importación">
				<thead><tr><th>Fila</th><th>Columna</th><th>Error</th></tr></thead>
				<tbody>
					{#each errores as e, i (`${e.fila}-${e.columna}-${i}`)}
						<tr>
							<td>{e.fila === 0 ? 'Archivo' : e.fila}</td>
							<td><code>{e.columna}</code></td>
							<td>{e.mensaje}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</Tarjeta>
	{:else if resumen && resumen.con_error === 0 && !resumen.aplicado}
		<p class="ayuda">Sin errores: podés aplicar la importación.</p>
	{/if}
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.archivo {
		display: block;
		margin-bottom: 6px;
		font-weight: 600;
		font-size: 13px;
	}

	.ayuda {
		margin: 8px 0 0;
		color: var(--texto-suave);
		font-size: 13px;
	}

	.acciones {
		display: flex;
		gap: 8px;
		margin-top: 12px;
	}

	.resumen {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
		margin: 0;
	}

	.resumen dt {
		color: var(--texto-suave);
		font-size: 12px;
	}

	.resumen dd {
		margin: 2px 0 0;
		font-size: 20px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
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

	@media (max-width: 720px) {
		.resumen {
			grid-template-columns: 1fr 1fr;
		}
	}
</style>
