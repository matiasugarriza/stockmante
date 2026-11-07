<script lang="ts">
	import type { Deposito, Ubicacion } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import { formatearMoneda } from '#lib/utils/formato';
	import { textoUbicacion, textoUbicaciones } from '#lib/utils/ubicaciones';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import Boton from '../ui/Boton.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';
	import TablaDatos from '../ui/TablaDatos.svelte';
	import ListaTarjetas from '../movil/ListaTarjetas.svelte';
	import TarjetaRegistro from '../movil/TarjetaRegistro.svelte';
	import type { Columna } from '../ui/tabla';

	interface Props {
		productos: Producto[];
		ubicacionesPorProducto: Record<number, Ubicacion[]>;
		depositos: Deposito[];
		cargando?: boolean;
		onEditar: (producto: Producto) => void;
		onAlternarEstado: (producto: Producto) => void;
	}

	let {
		productos,
		ubicacionesPorProducto,
		depositos,
		cargando = false,
		onEditar,
		onAlternarEstado
	}: Props = $props();

	const COLUMNAS: Columna<Producto>[] = [
		{ clave: 'nombre', encabezado: 'Nombre' },
		{ clave: 'sku', encabezado: 'SKU' },
		{ clave: 'categoria', encabezado: 'Categoría' },
		{ clave: 'ubicaciones', encabezado: 'Ubicaciones' },
		{ clave: 'precio_venta', encabezado: 'Precio', alineacion: 'derecha' },
		{ clave: 'stock_minimo', encabezado: 'Stock mín.', alineacion: 'derecha' },
		{ clave: 'activo', encabezado: 'Estado' },
		{ clave: 'acciones', encabezado: 'Acciones', alineacion: 'derecha' }
	];
</script>

{#if $dispositivo.esMovil}
	<ListaTarjetas items={productos} etiqueta="Productos" {cargando}>
		{#snippet tarjeta(producto)}
			{@const listaUbicaciones = ubicacionesPorProducto[producto.id]}
			{@const contexto =
				listaUbicaciones === undefined
					? 'Ubicaciones no disponibles'
					: listaUbicaciones.length === 0
						? 'Sin ubicación'
						: textoUbicaciones(listaUbicaciones, depositos)}
			<TarjetaRegistro
				titulo={producto.nombre}
				subtitulo={producto.codigo_barras
					? `${producto.sku} · ${producto.codigo_barras}`
					: producto.sku}
				mono
				badge={{
					texto: producto.activo ? 'Activo' : 'Inactivo',
					variante: producto.activo ? 'exito' : 'neutro'
				}}
				campos={[
					{ etiqueta: 'Categoría', valor: producto.categoria.nombre },
					{
						etiqueta: 'Precio',
						valor: `${formatearMoneda(producto.precio_venta)} ${producto.unidad_medida}`
					},
					{ etiqueta: 'Stock mín.', valor: String(producto.stock_minimo) },
					{ etiqueta: 'Ubicaciones', valor: contexto }
				]}
			>
				{#snippet acciones()}
					<Boton variante="secundario" tamano="sm" onclick={() => onEditar(producto)}>
						Editar
					</Boton>
					<Boton variante="fantasma" tamano="sm" onclick={() => onAlternarEstado(producto)}>
						{producto.activo ? 'Desactivar' : 'Reactivar'}
					</Boton>
				{/snippet}
			</TarjetaRegistro>
		{/snippet}
		{#snippet vacio()}
			<EstadoVacio
				icono="productos"
				titulo="No hay productos para estos filtros"
				descripcion="Probá con otra búsqueda o creá el primer producto."
			/>
		{/snippet}
	</ListaTarjetas>
{:else}
	<TablaDatos columnas={COLUMNAS} filas={productos} etiqueta="Productos" {cargando}>
	{#snippet fila(producto)}
		{@const listaUbicaciones = ubicacionesPorProducto[producto.id]}
		<td>
			<p class="nombre">{producto.nombre}</p>
			{#if producto.codigo_barras}<small>{producto.codigo_barras}</small>{/if}
		</td>
		<td class="mono">{producto.sku}</td>
		<td>{producto.categoria.nombre}</td>
		<td class="ubicaciones">
			{#if listaUbicaciones === undefined}
				<span class="tenue" title="No se pudieron cargar las ubicaciones">No disponible</span>
			{:else if listaUbicaciones.length === 0}
				<span class="sin-ubicacion">Sin ubicación</span>
			{:else}
				<span
					title={listaUbicaciones
						.map((ubicacion) => textoUbicacion(ubicacion, depositos))
						.join('\n')}
				>
					{textoUbicaciones(listaUbicaciones, depositos)}
				</span>
			{/if}
		</td>
		<td class="numero">
			{formatearMoneda(producto.precio_venta)}
			<span class="unidad">{producto.unidad_medida}</span>
		</td>
		<td class="numero">{producto.stock_minimo}</td>
		<td>
			<EtiquetaEstado variante={producto.activo ? 'exito' : 'neutro'}>
				{producto.activo ? 'Activo' : 'Inactivo'}
			</EtiquetaEstado>
		</td>
		<td>
			<div class="acciones">
				<Boton variante="secundario" tamano="sm" onclick={() => onEditar(producto)}>
					Editar
				</Boton>
				<Boton variante="fantasma" tamano="sm" onclick={() => onAlternarEstado(producto)}>
					{producto.activo ? 'Desactivar' : 'Reactivar'}
				</Boton>
			</div>
		</td>
	{/snippet}
	{#snippet vacio()}
		<EstadoVacio
			icono="productos"
			titulo="No hay productos para estos filtros"
			descripcion="Probá con otra búsqueda o creá el primer producto."
		/>
	{/snippet}
</TablaDatos>
{/if}

<style>
	.nombre {
		margin: 0;
		font-weight: 500;
	}

	small {
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
		font-size: 10.5px;
	}

	.mono {
		color: var(--texto-suave);
		font-family: var(--fuente-mono);
		font-size: 11.5px;
		white-space: nowrap;
	}

	.ubicaciones {
		max-width: 240px;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.tenue {
		color: var(--texto-tenue);
	}

	.sin-ubicacion {
		color: var(--ambar-texto);
	}

	.numero {
		text-align: right;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.unidad {
		display: block;
		color: var(--texto-tenue);
		font-size: 10.5px;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 4px;
		flex-wrap: wrap;
	}
</style>
