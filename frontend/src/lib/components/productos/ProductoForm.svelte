<script lang="ts">
	import {
		CrearProductoSchema,
		type CrearProductoInput,
		type Producto,
		type UnidadMedida
	} from '#lib/schemas/productos';
	import type { Categoria } from '#lib/schemas/categorias';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';

	interface Props {
		producto: Producto | null;
		categorias: Categoria[];
		guardando: boolean;
		error: string | null;
		/** 020: código leído que no existía; precarga el alta. */
		codigoInicial?: string;
		onGuardar: (datos: CrearProductoInput) => void;
		onCerrar: () => void;
	}

	let { producto, categorias, guardando, error, codigoInicial, onGuardar, onCerrar }: Props =
		$props();

	// El padre monta el formulario al abrirlo, con `producto` ya definido.
	// svelte-ignore state_referenced_locally
	let nombre = $state(producto?.nombre ?? '');
	// svelte-ignore state_referenced_locally
	let sku = $state(producto?.sku ?? '');
	// svelte-ignore state_referenced_locally
	let codigoBarras = $state(producto?.codigo_barras ?? codigoInicial ?? '');
	// svelte-ignore state_referenced_locally
	let categoriaId = $state(producto ? String(producto.categoria.id) : '');
	// svelte-ignore state_referenced_locally
	let unidad = $state<UnidadMedida>(producto?.unidad_medida ?? 'UNIDAD');
	// svelte-ignore state_referenced_locally
	let costo = $state(producto ? String(producto.costo) : '');
	// svelte-ignore state_referenced_locally
	let precio = $state(producto ? String(producto.precio_venta) : '');
	// svelte-ignore state_referenced_locally
	let stockMinimo = $state(producto ? String(producto.stock_minimo) : '0');
	let errorValidacion = $state<string | null>(null);

	// Al editar se ofrece la categoría actual aunque esté inactiva: el backend
	// permite conservarla, pero no asignar una distinta inactiva.
	const opciones = $derived(
		categorias.filter((c) => c.activo || c.id === producto?.categoria.id)
	);
	const opcionesCategorias = $derived([
		{ valor: '', etiqueta: 'Elegí una categoría' },
		...opciones.map((categoria) => ({
			valor: String(categoria.id),
			etiqueta: categoria.activo ? categoria.nombre : `${categoria.nombre} (inactiva)`
		}))
	]);

	function numero(valor: string): number {
		return Number(valor.replace(',', '.'));
	}

	function enviar() {
		errorValidacion = null;
		if (!costo.trim() || !precio.trim() || !stockMinimo.trim()) {
			errorValidacion = 'Completá costo, precio de venta y stock mínimo.';
			return;
		}
		const datos = CrearProductoSchema.safeParse({
			nombre,
			sku,
			codigo_barras: codigoBarras,
			categoria_id: Number(categoriaId),
			unidad_medida: unidad,
			costo: numero(costo),
			precio_venta: numero(precio),
			stock_minimo: numero(stockMinimo)
		});
		if (!datos.success) {
			const primero = datos.error.issues[0];
			errorValidacion = primero ? primero.message : 'Revisá los datos';
			return;
		}
		onGuardar(datos.data);
	}

	const etiquetasUnidad: { valor: UnidadMedida; etiqueta: string }[] = [
		{ valor: 'UNIDAD', etiqueta: 'Unidad (UNIDAD)' },
		{ valor: 'KG', etiqueta: 'Kilogramo (KG)' },
		{ valor: 'METRO', etiqueta: 'Metro (METRO)' },
		{ valor: 'CAJA', etiqueta: 'Caja (CAJA)' }
	];
</script>

<Modal
	abierto={true}
	titulo={producto ? 'Editar producto' : 'Nuevo producto'}
	descripcion="El SKU se guarda en mayúsculas y es único incluso para productos inactivos."
	oncerrar={onCerrar}
>
	<form
		class="grilla"
		onsubmit={(event) => {
			event.preventDefault();
			enviar();
		}}
	>
		<div class="ancho">
			<Campo
				etiqueta="Nombre"
				nombre="nombre"
				maxlength={160}
				bind:valor={nombre}
				requerido={true}
			/>
		</div>
		<Campo
			etiqueta="SKU"
			nombre="sku"
			maxlength={30}
			marcador="YER-001"
			bind:valor={sku}
			requerido={true}
		/>
		<Campo
			etiqueta="Código de barras"
			nombre="codigo_barras"
			maxlength={64}
			ayuda="Opcional, 8–64 caracteres alfanuméricos."
			bind:valor={codigoBarras}
		/>
		<Campo
			etiqueta="Categoría"
			nombre="categoria"
			tipo="select"
			opciones={opcionesCategorias}
			bind:valor={categoriaId}
			requerido={true}
		/>
		<Campo
			etiqueta="Unidad de medida"
			nombre="unidad"
			tipo="select"
			opciones={etiquetasUnidad}
			bind:valor={unidad}
		/>
		<Campo
			etiqueta="Costo"
			nombre="costo"
			marcador="0.00"
			bind:valor={costo}
			requerido={true}
		/>
		<Campo
			etiqueta="Precio de venta"
			nombre="precio"
			marcador="0.00"
			bind:valor={precio}
			requerido={true}
		/>
		<Campo
			etiqueta="Stock mínimo"
			nombre="stock_minimo"
			marcador="0"
			bind:valor={stockMinimo}
			requerido={true}
		/>

		{#if error || errorValidacion}
			<div class="ancho">
				<Aviso variante="error">{error ?? errorValidacion}</Aviso>
			</div>
		{/if}

		<div class="acciones ancho">
			<Boton variante="secundario" deshabilitado={guardando} onclick={onCerrar}>Cancelar</Boton>
			<Boton tipo="submit" cargando={guardando}>
				{guardando ? 'Guardando…' : producto ? 'Guardar cambios' : 'Crear producto'}
			</Boton>
		</div>
	</form>
</Modal>

<style>
	.grilla {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
	}

	.ancho {
		grid-column: 1 / -1;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 2px;
	}

	@media (max-width: 520px) {
		.grilla {
			grid-template-columns: 1fr;
		}
	}
</style>
