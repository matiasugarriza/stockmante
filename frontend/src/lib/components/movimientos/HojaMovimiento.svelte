<script lang="ts">
	import { get } from 'svelte/store';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';
	import { listarDepositosActivos, type DepositoActivo } from '#lib/api/depositos';
	import { crearMovimiento } from '#lib/api/movimientos';
	import { obtenerStockProducto } from '#lib/api/stock';
	import {
		MOTIVOS_POR_TIPO,
		motivoPorDefecto,
		type StockItem,
		type StockProducto,
		type TipoMovimiento
	} from '#lib/schemas/movimientos';
	import { sesion } from '#lib/stores/sesion';
	import { mensajeMovimiento, mostrarAviso } from '#lib/stores/avisos';
	import { mensajeError } from '#lib/utils/errores';
	import { validarCantidadMovil } from '#lib/utils/inventario';

	interface Props {
		abierta: boolean;
		item: StockItem | null;
		oncerrar: () => void;
		onregistrado: () => void;
	}

	let { abierta, item, oncerrar, onregistrado }: Props = $props();

	let tipo = $state<TipoMovimiento>('ENTRADA');
	let depositoId = $state('');
	let cantidad = $state('');
	let observacion = $state('');
	let depositos = $state<DepositoActivo[]>([]);
	let desglose = $state<StockProducto | null>(null);
	let error = $state<string | null>(null);
	let enviando = $state(false);

	const esGestion = $derived(
		$sesion.estado === 'activa' && $sesion.usuario.familia_rol === 'GESTION'
	);
	const tipos = $derived(
		esGestion
			? [
					{ valor: 'ENTRADA', etiqueta: 'Entrada' },
					{ valor: 'SALIDA', etiqueta: 'Salida' },
					{ valor: 'MERMA', etiqueta: 'Merma' },
					{ valor: 'AJUSTE', etiqueta: 'Ajuste' }
				]
			: [
					{ valor: 'ENTRADA', etiqueta: 'Entrada' },
					{ valor: 'SALIDA', etiqueta: 'Salida' },
					{ valor: 'MERMA', etiqueta: 'Merma' }
				]
	);
	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Elegí un depósito' },
		...depositos.map((deposito) => ({ valor: String(deposito.id), etiqueta: deposito.nombre }))
	]);
	const disponible = $derived.by(() => {
		if (!desglose || !depositoId) return null;
		const id = Number(depositoId);
		return desglose.desglose
			.filter((fila) => fila.deposito_id === id)
			.reduce((total, fila) => total + fila.cantidad, 0);
	});

	$effect(() => {
		if (!abierta) return;
		error = null;
		enviando = false;
		cantidad = '';
		observacion = '';
		tipo = 'ENTRADA';
		void cargarDepositos();
		if (item) void cargarDesglose(item.producto_id);
	});

	async function cargarDepositos() {
		try {
			depositos = await listarDepositosActivos();
			depositoId = depositos[0] ? String(depositos[0].id) : '';
		} catch (fallo) {
			error = mensajeError(fallo, 'No se pudieron cargar los depósitos.');
		}
	}

	async function cargarDesglose(productoId: number) {
		try {
			desglose = await obtenerStockProducto(productoId);
		} catch {
			desglose = null;
		}
	}

	function cambiarTipo(valor: string) {
		tipo = valor as TipoMovimiento;
	}

	function confirmar() {
		if (!item) return;
		error = validarCantidadMovil({
			tipo,
			cantidad,
			disponible,
			esGestion
		});
		if (error) return;
		void enviar();
	}

	async function enviar() {
		if (!item || !depositoId) return;
		enviando = true;
		error = null;
		const valor = Math.abs(Number(cantidad));
		const signo = tipo === 'SALIDA' || tipo === 'MERMA' ? -1 : 1;
		try {
			const resultado = await crearMovimiento({
				tipo,
				deposito_id: Number(depositoId),
				motivo: motivoPorDefecto(tipo),
				observacion: observacion.trim() ? observacion.trim() : null,
				lineas: [{ producto_id: item.producto_id, cantidad_delta: signo * valor }]
			});
			const linea = resultado.lineas[0];
			if (linea) {
				mostrarAviso(
					mensajeMovimiento(
						tipo,
						linea.producto_nombre,
						resultado.deposito.nombre,
						linea.cantidad_delta,
						linea.stock_resultante
					),
					'exito'
				);
			}
			onregistrado();
			oncerrar();
		} catch (fallo) {
			error = mensajeError(fallo, 'No se pudo registrar el movimiento.');
		} finally {
			enviando = false;
		}
	}
</script>

<Modal
	abierto={abierta}
	titulo={item ? `Movimiento: ${item.nombre}` : 'Nuevo movimiento'}
	descripcion={item ? item.sku : undefined}
	variante="hoja"
	{oncerrar}
>
	{#if item}
		<div class="formulario">
			{#if error}
				<Aviso variante="error" titulo="No se pudo registrar">{error}</Aviso>
			{/if}
			<Campo
				etiqueta="Tipo"
				nombre="tipo"
				tipo="select"
				opciones={tipos}
				valor={tipo}
				onchange={cambiarTipo}
			/>
			<Campo
				etiqueta="Depósito"
				nombre="deposito"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={depositoId}
			/>
			<Campo
				etiqueta="Cantidad"
				nombre="cantidad"
				tipo="number"
				min="1"
				step="1"
				marcador="0"
				bind:valor={cantidad}
			/>
			{#if disponible !== null && (tipo === 'SALIDA' || tipo === 'MERMA')}
				<p class="disponible">Disponible en el depósito: {disponible}</p>
			{/if}
			<Campo
				etiqueta="Observación (opcional)"
				nombre="observacion"
				tipo="textarea"
				marcador="Detalle del movimiento"
				bind:valor={observacion}
			/>
		</div>
	{:else}
		<p class="vacio">Elegí un producto de la lista para ajustar su stock.</p>
	{/if}

	{#snippet acciones()}
		<Boton variante="secundario" onclick={oncerrar}>Cancelar</Boton>
		<Boton
			onclick={confirmar}
			cargando={enviando}
			deshabilitado={!item || enviando}
		>
			Confirmar
		</Boton>
	{/snippet}
</Modal>

<style>
	.formulario {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.disponible {
		margin: -6px 0 0;
		color: var(--texto-tenue);
		font-size: 12px;
	}

	.vacio {
		margin: 12px 0;
		color: var(--texto-suave);
		font-size: 13px;
	}
</style>
