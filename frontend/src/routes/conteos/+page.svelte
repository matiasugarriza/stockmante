<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import {
		cancelarConteo,
		cargarLineas,
		cerrarConteo,
		crearConteo,
		listarConteos,
		obtenerConteo
	} from '#lib/api/conteos';
	import { listarDepositos } from '#lib/api/depositos';
	import type { Conteo, ConteoLinea } from '#lib/schemas/conteos';
	import type { Deposito } from '#lib/schemas/depositos';
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

	type EstadoConteo = Conteo['estado'];

	const formatoFecha = new Intl.DateTimeFormat('es-AR', {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	const VARIANTES_ESTADO: Record<EstadoConteo, 'info' | 'exito' | 'neutro'> = {
		ABIERTO: 'info',
		CERRADO: 'exito',
		CANCELADO: 'neutro'
	};

	const ESTADOS = [
		{ valor: '', etiqueta: 'Todos' },
		{ valor: 'ABIERTO', etiqueta: 'Abierto' },
		{ valor: 'CERRADO', etiqueta: 'Cerrado' },
		{ valor: 'CANCELADO', etiqueta: 'Cancelado' }
	];

	const COLUMNAS_LINEAS: Columna<ConteoLinea>[] = [
		{ clave: 'producto_nombre', encabezado: 'Producto' },
		{ clave: 'ubicacion_nombre', encabezado: 'Ubicación' },
		{ clave: 'cantidad_sistema', encabezado: 'Sistema', alineacion: 'derecha' },
		{ clave: 'cantidad_contada', encabezado: 'Contado', alineacion: 'derecha' },
		{ clave: 'diferencia', encabezado: 'Diferencia', alineacion: 'derecha' }
	];

	const COLUMNAS_HISTORIAL: Columna<Conteo>[] = [
		{ clave: 'id', encabezado: 'ID' },
		{ clave: 'deposito_nombre', encabezado: 'Depósito' },
		{ clave: 'estado', encabezado: 'Estado' },
		{ clave: 'usuario', encabezado: 'Autor' },
		{ clave: 'creado_en', encabezado: 'Fecha' },
		{ clave: 'acciones', encabezado: 'Acción' }
	];

	let depositos = $state<Deposito[]>([]);
	let depositoId = $state('');
	let observacion = $state('');

	let conteos = $state<Conteo[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let filtroEstado = $state('');

	let conteoActivo = $state<Conteo | null>(null);
	let conteosCargando = $state(false);
	let error = $state('');
	let exito = $state('');

	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Seleccionar' },
		...depositos.map((deposito) => ({ valor: String(deposito.id), etiqueta: deposito.nombre }))
	]);
	const conteoEditable = $derived(conteoActivo?.estado === 'ABIERTO');

	async function cargarDepositos() {
		try {
			depositos = await listarDepositos('true');
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron cargar los depósitos';
		}
	}

	async function cargarConteos() {
		conteosCargando = true;
		try {
			const res = await listarConteos({
				estado: filtroEstado || undefined,
				pagina,
				porPagina
			});
			conteos = res.items;
			total = res.total;
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron cargar los conteos';
		} finally {
			conteosCargando = false;
		}
	}

	async function abrir() {
		error = '';
		exito = '';
		if (!depositoId) {
			error = 'Elegí un depósito';
			return;
		}
		try {
			const c = await crearConteo({
				deposito_id: Number(depositoId),
				observacion: observacion.trim() || null
			});
			conteoActivo = c;
			exito = `Conteo #${c.id} abierto con ${c.lineas.length} líneas.`;
			observacion = '';
			await cargarConteos();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo abrir el conteo';
		}
	}

	async function guardarLineas() {
		if (!conteoActivo) return;
		error = '';
		exito = '';
		try {
			const payload = {
				lineas: conteoActivo.lineas.map((l: ConteoLinea) => ({
					linea_id: l.id,
					cantidad_contada: l.cantidad_contada ?? 0
				}))
			};
			conteoActivo = await cargarLineas(conteoActivo.id, payload);
			exito = 'Cantidades guardadas.';
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudieron guardar las cantidades';
		}
	}

	async function cerrar() {
		if (!conteoActivo) return;
		error = '';
		exito = '';
		try {
			conteoActivo = await cerrarConteo(conteoActivo.id);
			exito = `Conteo #${conteoActivo.id} cerrado.`;
			await cargarConteos();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cerrar el conteo';
		}
	}

	async function cancelar() {
		if (!conteoActivo) return;
		error = '';
		exito = '';
		try {
			conteoActivo = await cancelarConteo(conteoActivo.id);
			exito = `Conteo #${conteoActivo.id} cancelado.`;
			await cargarConteos();
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cancelar el conteo';
		}
	}

	async function verDetalle(id: number) {
		try {
			conteoActivo = await obtenerConteo(id);
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cargar el conteo';
		}
	}

	function cambiarPagina(nueva: number) {
		pagina = nueva;
		void cargarConteos();
	}

	onMount(async () => {
		await cargarDepositos();
		await cargarConteos();
	});
</script>

<svelte:head>
	<title>Conteos · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Conteos de inventario"
		descripcion="Abrí un conteo por depósito, cargá las cantidades reales y cerralo para ajustar las diferencias."
	/>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if exito}<Aviso variante="exito">{exito}</Aviso>{/if}

	<Tarjeta titulo="Abrir conteo">
		<div class="grilla-apertura">
			<Campo
				etiqueta="Depósito"
				nombre="deposito"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={depositoId}
			/>
			<Campo
				etiqueta="Observación (opcional)"
				nombre="observacion"
				maxlength={500}
				bind:valor={observacion}
			/>
			<div class="accion-apertura">
				<Boton onclick={() => void abrir()}>Abrir conteo</Boton>
			</div>
		</div>
	</Tarjeta>

	{#if conteoActivo}
		<Tarjeta>
			<div class="cabecera-conteo">
				<div>
					<h2>Conteo #{conteoActivo.id}</h2>
					<p>{conteoActivo.deposito_nombre} · {formatoFecha.format(new Date(conteoActivo.creado_en))}</p>
				</div>
				<EtiquetaEstado variante={VARIANTES_ESTADO[conteoActivo.estado]}>
					{conteoActivo.estado}
				</EtiquetaEstado>
			</div>

			{#if conteoEditable}
				<div class="acciones-conteo">
					<Boton variante="secundario" onclick={() => void guardarLineas()}>
						Guardar cantidades
					</Boton>
					<Boton onclick={() => void cerrar()}>Cerrar</Boton>
					<Boton variante="peligro" onclick={() => void cancelar()}>Cancelar</Boton>
				</div>
			{/if}

			<TablaDatos
				columnas={COLUMNAS_LINEAS}
				filas={conteoActivo.lineas}
				etiqueta={`Líneas del conteo ${conteoActivo.id}`}
			>
				{#snippet fila(linea)}
					<td>
						{linea.producto_nombre}
						<span class="sku">{linea.sku}</span>
					</td>
					<td>{linea.ubicacion_nombre ?? 'Depósito'}</td>
					<td class="numero">{linea.cantidad_sistema}</td>
					<td class="numero">
						{#if conteoEditable}
							<input
								class="input-contado"
								type="number"
								min="0"
								value={linea.cantidad_contada ?? 0}
								oninput={(e) => {
									const v = Number((e.target as HTMLInputElement).value);
									linea.cantidad_contada = Number.isNaN(v) ? 0 : v;
								}}
							/>
						{:else}
							{linea.cantidad_contada ?? '—'}
						{/if}
					</td>
					<td
						class="numero"
						class:positivo={(linea.diferencia ?? 0) > 0}
						class:negativo={(linea.diferencia ?? 0) < 0}
					>
						{linea.diferencia ?? '—'}
					</td>
				{/snippet}
			</TablaDatos>

			{#if conteoActivo.movimiento_ajuste_id}
				<Aviso variante="info">
					Movimiento de ajuste generado: #{conteoActivo.movimiento_ajuste_id}
				</Aviso>
			{/if}
		</Tarjeta>
	{/if}

	<Tarjeta titulo="Historial">
		<div class="filtros">
			<Campo
				etiqueta="Estado"
				nombre="filtro-estado"
				tipo="select"
				opciones={ESTADOS}
				bind:valor={filtroEstado}
				onchange={() => void cargarConteos()}
			/>
		</div>

		<TablaDatos
			columnas={COLUMNAS_HISTORIAL}
			filas={conteos}
			etiqueta="Historial de conteos"
			cargando={conteosCargando}
		>
			{#snippet fila(conteo)}
				<td>{conteo.id}</td>
				<td>{conteo.deposito_nombre}</td>
				<td>
					<EtiquetaEstado variante={VARIANTES_ESTADO[conteo.estado]}>
						{conteo.estado}
					</EtiquetaEstado>
				</td>
				<td>{conteo.usuario.nombre}</td>
				<td class="fecha">{formatoFecha.format(new Date(conteo.creado_en))}</td>
				<td>
					<Boton variante="fantasma" tamano="sm" onclick={() => void verDetalle(conteo.id)}>
						Ver
					</Boton>
				</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio
					icono="conteos"
					titulo="No hay conteos registrados"
					descripcion="Abrí el primero con el formulario de arriba."
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

	.grilla-apertura {
		display: grid;
		grid-template-columns: 220px minmax(0, 1fr) auto;
		gap: 12px;
		align-items: end;
	}

	.accion-apertura {
		padding-bottom: 2px;
	}

	.cabecera-conteo {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 12px;
	}

	.cabecera-conteo h2 {
		margin: 0;
		color: var(--texto);
		font-size: 16px;
		font-weight: 650;
	}

	.cabecera-conteo p {
		margin: 3px 0 0;
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	.acciones-conteo {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-bottom: 12px;
	}

	.filtros {
		display: flex;
		gap: 10px;
		margin-bottom: 12px;
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

	.numero.positivo {
		color: var(--exito-texto);
	}

	.numero.negativo {
		color: var(--peligro-texto);
	}

	.input-contado {
		width: 84px;
		min-height: 30px;
		padding: 3px 6px;
		border: 1px solid var(--borde-fuerte);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto);
		font-family: var(--fuente-sans);
		font-size: 13px;
		text-align: right;
	}

	.fecha {
		white-space: nowrap;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.pie-tabla {
		padding: 12px 16px 0;
	}

	@media (max-width: 760px) {
		.grilla-apertura {
			grid-template-columns: 1fr;
		}
	}
</style>
