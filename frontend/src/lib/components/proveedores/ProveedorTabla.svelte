<script lang="ts">
	import type { Proveedor } from '#lib/schemas/proveedores';
	import { dispositivo } from '#lib/dispositivo/dispositivo';
	import Boton from '../ui/Boton.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';
	import TablaDatos from '../ui/TablaDatos.svelte';
	import ListaTarjetas from '../movil/ListaTarjetas.svelte';
	import TarjetaRegistro from '../movil/TarjetaRegistro.svelte';
	import type { Columna } from '../ui/tabla';

	interface Props {
		proveedores: Proveedor[];
		cargando?: boolean;
		onEditar: (proveedor: Proveedor) => void;
		onAlternarEstado: (proveedor: Proveedor) => void;
	}

	let { proveedores, cargando = false, onEditar, onAlternarEstado }: Props = $props();

	const COLUMNAS: Columna<Proveedor>[] = [
		{ clave: 'id', encabezado: 'ID' },
		{ clave: 'nombre', encabezado: 'Nombre' },
		{ clave: 'cuit', encabezado: 'CUIT' },
		{ clave: 'email', encabezado: 'Email' },
		{ clave: 'telefono', encabezado: 'Teléfono' },
		{ clave: 'activo', encabezado: 'Estado' },
		{ clave: 'acciones', encabezado: 'Acciones', alineacion: 'derecha' }
	];
</script>

{#if $dispositivo.esMovil}
	<ListaTarjetas items={proveedores} etiqueta="Proveedores" {cargando}>
		{#snippet tarjeta(proveedor)}
			<TarjetaRegistro
				titulo={proveedor.nombre}
				subtitulo={proveedor.email ?? proveedor.telefono ?? 'Sin contacto'}
				badge={{
					texto: proveedor.activo ? 'Activo' : 'Inactivo',
					variante: proveedor.activo ? 'exito' : 'neutro'
				}}
				campos={[
					{ etiqueta: 'CUIT', valor: proveedor.cuit ?? '—' },
					{ etiqueta: 'Teléfono', valor: proveedor.telefono ?? '—' }
				]}
			>
				{#snippet acciones()}
					<Boton variante="secundario" tamano="sm" onclick={() => onEditar(proveedor)}>
						Editar
					</Boton>
					<Boton variante="fantasma" tamano="sm" onclick={() => onAlternarEstado(proveedor)}>
						{proveedor.activo ? 'Desactivar' : 'Reactivar'}
					</Boton>
				{/snippet}
			</TarjetaRegistro>
		{/snippet}
		{#snippet vacio()}
			<EstadoVacio
				icono="proveedores"
				titulo="Todavía no hay proveedores en este filtro"
				descripcion="Cambiá el filtro o creá el primer proveedor."
			/>
		{/snippet}
	</ListaTarjetas>
{:else}
	<TablaDatos columnas={COLUMNAS} filas={proveedores} etiqueta="Proveedores" {cargando}>
	{#snippet fila(proveedor)}
		<td class="id">{proveedor.id}</td>
		<td class="principal">{proveedor.nombre}</td>
		<td>{proveedor.cuit ?? '—'}</td>
		<td>{proveedor.email ?? '—'}</td>
		<td>{proveedor.telefono ?? '—'}</td>
		<td>
			<EtiquetaEstado variante={proveedor.activo ? 'exito' : 'neutro'}>
				{proveedor.activo ? 'Activo' : 'Inactivo'}
			</EtiquetaEstado>
		</td>
		<td>
			<div class="acciones">
				<Boton variante="secundario" tamano="sm" onclick={() => onEditar(proveedor)}>
					Editar
				</Boton>
				<Boton variante="fantasma" tamano="sm" onclick={() => onAlternarEstado(proveedor)}>
					{proveedor.activo ? 'Desactivar' : 'Reactivar'}
				</Boton>
			</div>
		</td>
	{/snippet}
	{#snippet vacio()}
		<EstadoVacio
			icono="proveedores"
			titulo="Todavía no hay proveedores en este filtro"
			descripcion="Creá el primero para poder registrar compras."
		/>
	{/snippet}
</TablaDatos>
{/if}

<style>
	.id {
		color: var(--texto-tenue);
		font-variant-numeric: tabular-nums;
	}

	.principal {
		font-weight: 500;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 4px;
		flex-wrap: wrap;
	}
</style>
