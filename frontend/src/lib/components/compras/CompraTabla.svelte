<script lang="ts">
	import type { Compra } from '#lib/schemas/compras';
	import { formatearFechaCorta, formatearMoneda } from '#lib/utils/formato';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import Boton from '../ui/Boton.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';
	import TablaDatos from '../ui/TablaDatos.svelte';
	import ListaTarjetas from '../movil/ListaTarjetas.svelte';
	import TarjetaRegistro from '../movil/TarjetaRegistro.svelte';
	import type { Columna } from '../ui/tabla';

	interface Props {
		compras: Compra[];
		cargando?: boolean;
		onEditar: (compra: Compra) => void;
		onConfirmar: (compra: Compra) => void;
		onAnular: (compra: Compra) => void;
		onEliminar: (compra: Compra) => void;
	}

	let {
		compras,
		cargando = false,
		onEditar,
		onConfirmar,
		onAnular,
		onEliminar
	}: Props = $props();

	const COLUMNAS: Columna<Compra>[] = [
		{ clave: 'id', encabezado: 'Compra' },
		{ clave: 'creado_en', encabezado: 'Fecha' },
		{ clave: 'proveedor_nombre', encabezado: 'Proveedor' },
		{ clave: 'deposito_nombre', encabezado: 'Depósito' },
		{ clave: 'numero', encabezado: 'Nº factura/remito' },
		{ clave: 'estado', encabezado: 'Estado' },
		{ clave: 'total_costo', encabezado: 'Total', alineacion: 'derecha' },
		{ clave: 'acciones', encabezado: 'Acciones', alineacion: 'derecha' }
	];

	const VARIANTES_ESTADO: Record<string, 'aviso' | 'exito' | 'neutro'> = {
		BORRADOR: 'aviso',
		CONFIRMADA: 'exito',
		ANULADA: 'neutro'
	};
</script>

{#if $dispositivo.esMovil}
	<ListaTarjetas items={compras} etiqueta="Compras" {cargando}>
		{#snippet tarjeta(compra)}
			<TarjetaRegistro
				titulo={`Compra #${compra.id}`}
				subtitulo={compra.proveedor_nombre}
				badge={{
					texto: compra.estado.charAt(0) + compra.estado.slice(1).toLowerCase(),
					variante: VARIANTES_ESTADO[compra.estado] ?? 'neutro'
				}}
				campos={[
					{ etiqueta: 'Fecha', valor: formatearFechaCorta(compra.creado_en) },
					{ etiqueta: 'Depósito', valor: compra.deposito_nombre },
					{ etiqueta: 'Factura', valor: compra.numero ?? '—' },
					{ etiqueta: 'Total', valor: formatearMoneda(compra.total_costo) }
				]}
			>
				{#snippet acciones()}
					{#if compra.estado === 'BORRADOR'}
						<Boton variante="secundario" tamano="sm" onclick={() => onEditar(compra)}>
							Editar
						</Boton>
						<Boton variante="secundario" tamano="sm" onclick={() => onConfirmar(compra)}>
							Confirmar
						</Boton>
						<Boton variante="fantasma" tamano="sm" onclick={() => onEliminar(compra)}>
							Eliminar
						</Boton>
					{:else if compra.estado === 'CONFIRMADA'}
						<Boton variante="fantasma" tamano="sm" onclick={() => onAnular(compra)}>
							Anular
						</Boton>
					{:else}
						<span class="sin-acciones">Sin acciones</span>
					{/if}
				{/snippet}
			</TarjetaRegistro>
		{/snippet}
		{#snippet vacio()}
			<EstadoVacio
				icono="compras"
				titulo="No hay compras en este filtro"
				descripcion="Creá una compra en borrador y confirmá la recepción cuando llegue la mercadería."
			/>
		{/snippet}
	</ListaTarjetas>
{:else}
	<TablaDatos columnas={COLUMNAS} filas={compras} etiqueta="Compras" {cargando}>
	{#snippet fila(compra)}
		<td class="id">Compra #{compra.id}</td>
		<td class="fecha">{formatearFechaCorta(compra.creado_en)}</td>
		<td class="principal">{compra.proveedor_nombre}</td>
		<td>{compra.deposito_nombre}</td>
		<td>{compra.numero ?? '—'}</td>
		<td>
			<EtiquetaEstado variante={VARIANTES_ESTADO[compra.estado] ?? 'neutro'}>
				{compra.estado}
			</EtiquetaEstado>
		</td>
		<td class="numero">{formatearMoneda(compra.total_costo)}</td>
		<td>
			<div class="acciones">
				{#if compra.estado === 'BORRADOR'}
					<Boton variante="secundario" tamano="sm" onclick={() => onEditar(compra)}>
						Editar
					</Boton>
					<Boton variante="secundario" tamano="sm" onclick={() => onConfirmar(compra)}>
						Confirmar
					</Boton>
					<Boton variante="fantasma" tamano="sm" onclick={() => onEliminar(compra)}>
						Eliminar
					</Boton>
				{:else if compra.estado === 'CONFIRMADA'}
					<Boton variante="fantasma" tamano="sm" onclick={() => onAnular(compra)}>
						Anular
					</Boton>
				{:else}
					<span class="sin-acciones">Sin acciones</span>
				{/if}
			</div>
		</td>
	{/snippet}
	{#snippet vacio()}
		<EstadoVacio
			icono="compras"
			titulo="No hay compras en este filtro"
			descripcion="Creá una compra en borrador y confirmá la recepción cuando llegue la mercadería."
		/>
	{/snippet}
</TablaDatos>
{/if}

<style>
	.id {
		color: var(--texto-tenue);
		font-variant-numeric: tabular-nums;
	}

	.fecha {
		white-space: nowrap;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.principal {
		font-weight: 500;
	}

	.numero {
		text-align: right;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 4px;
		flex-wrap: wrap;
	}

	.sin-acciones {
		color: var(--texto-tenue);
		font-size: 11.5px;
	}
</style>
