<script lang="ts">
	import { CrearCompraSchema, type CompraDetalle, type CrearCompraInput } from '#lib/schemas/compras';
	import type { Deposito } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import type { Proveedor } from '#lib/schemas/proveedores';
	import { formatearMoneda } from '#lib/utils/formato';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';

	interface Props {
		compra: CompraDetalle | null;
		proveedores: Proveedor[];
		depositos: Deposito[];
		productos: Producto[];
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: CrearCompraInput) => void;
		onCerrar: () => void;
	}

	let { compra, proveedores, depositos, productos, guardando, error, onGuardar, onCerrar }: Props =
		$props();

	interface LineaForm {
		productoId: string;
		cantidad: string;
		costo: string;
	}

	// El padre monta el formulario al abrirlo, con `compra` ya resuelta.
	// svelte-ignore state_referenced_locally
	let proveedorId = $state(compra ? String(compra.compra.proveedor_id) : '');
	// svelte-ignore state_referenced_locally
	let depositoId = $state(compra ? String(compra.compra.deposito_id) : '');
	// svelte-ignore state_referenced_locally
	let numero = $state(compra?.compra.numero ?? '');
	// svelte-ignore state_referenced_locally
	let observacion = $state(compra?.compra.observacion ?? '');
	// svelte-ignore state_referenced_locally
	let lineas = $state<LineaForm[]>(
		compra && compra.lineas.length > 0
			? compra.lineas.map((linea) => ({
					productoId: String(linea.producto_id),
					cantidad: String(linea.cantidad),
					costo: String(linea.costo_unitario)
				}))
			: [{ productoId: '', cantidad: '1', costo: '' }]
	);
	let errorValidacion = $state<string | null>(null);

	const opcionesProveedores = $derived([
		{ valor: '', etiqueta: 'Elegí un proveedor activo' },
		...proveedores.map((proveedor) => ({
			valor: String(proveedor.id),
			etiqueta: proveedor.nombre
		}))
	]);
	const opcionesDepositos = $derived([
		{ valor: '', etiqueta: 'Elegí un depósito activo' },
		...depositos.map((deposito) => ({
			valor: String(deposito.id),
			etiqueta: `${deposito.nombre} (${deposito.identificador})`
		}))
	]);

	const total = $derived(
		lineas.reduce((acumulado, linea) => {
			const cantidad = Number(linea.cantidad.replace(',', '.'));
			const costo = Number(linea.costo.replace(',', '.'));
			if (!Number.isFinite(cantidad) || !Number.isFinite(costo)) return acumulado;
			return acumulado + cantidad * costo;
		}, 0)
	);

	function subtotal(linea: LineaForm): string {
		const cantidad = Number(linea.cantidad.replace(',', '.'));
		const costo = Number(linea.costo.replace(',', '.'));
		return Number.isFinite(cantidad) && Number.isFinite(costo)
			? formatearMoneda(cantidad * costo)
			: '—';
	}

	function agregarLinea() {
		lineas.push({ productoId: '', cantidad: '1', costo: '' });
	}

	function quitarLinea(indice: number) {
		if (lineas.length === 1) return;
		lineas.splice(indice, 1);
	}

	function elegirProducto(indice: number, productoId: string) {
		lineas[indice].productoId = productoId;
		const producto = productos.find((p) => String(p.id) === productoId);
		if (producto && !lineas[indice].costo.trim()) {
			lineas[indice].costo = String(producto.costo);
		}
	}

	function enviar() {
		errorValidacion = null;
		// El backend rechaza líneas repetidas del mismo producto (unique en
		// movimiento_lineas); lo validamos antes para no mostrar un 500.
		const productosElegidos = lineas
			.map((linea) => linea.productoId)
			.filter((productoId) => productoId !== '');
		if (new Set(productosElegidos).size !== productosElegidos.length) {
			errorValidacion = 'No repitas el mismo producto en dos líneas.';
			return;
		}
		const datos = CrearCompraSchema.safeParse({
			proveedor_id: Number(proveedorId),
			deposito_id: Number(depositoId),
			numero,
			observacion,
			lineas: lineas.map((linea) => ({
				producto_id: Number(linea.productoId),
				cantidad: Number(linea.cantidad.replace(',', '.')),
				costo_unitario: Number(linea.costo.replace(',', '.'))
			}))
		});
		if (!datos.success) {
			const primero = datos.error.issues[0];
			errorValidacion = primero ? primero.message : 'Revisá los datos';
			return;
		}
		onGuardar(datos.data);
	}
</script>

<Modal
	abierto={true}
	tamano="lg"
	titulo={compra ? 'Editar compra en borrador' : 'Nueva compra'}
	descripcion="La compra nace en BORRADOR. Confirmar la recepción recién ahí mueve el stock."
	oncerrar={onCerrar}
>
	<form
		class="formulario"
		onsubmit={(event) => {
			event.preventDefault();
			enviar();
		}}
	>
		<div class="grilla">
			<Campo
				etiqueta="Proveedor"
				nombre="proveedor"
				tipo="select"
				opciones={opcionesProveedores}
				bind:valor={proveedorId}
				requerido={true}
			/>
			<Campo
				etiqueta="Depósito de destino"
				nombre="deposito"
				tipo="select"
				opciones={opcionesDepositos}
				bind:valor={depositoId}
				requerido={true}
			/>
			<Campo
				etiqueta="Nº factura/remito del proveedor (máx. 40)"
				nombre="numero"
				marcador="OC-0001"
				maxlength={40}
				bind:valor={numero}
			/>
			<Campo
				etiqueta="Observación"
				nombre="observacion"
				maxlength={500}
				bind:valor={observacion}
			/>
		</div>

		<div class="lineas">
			<div class="lineas-cabecera">
				<span class="titulo">Líneas</span>
				<Boton variante="secundario" tamano="sm" icono="mas" onclick={agregarLinea}>
					Agregar línea
				</Boton>
			</div>

			<div class="linea-cabecera" aria-hidden="true">
				<span>Producto</span>
				<span>Cantidad</span>
				<span>Costo unitario</span>
				<span class="derecha">Subtotal</span>
				<span></span>
			</div>

			{#each lineas as linea, indice (indice)}
				{@const producto = productos.find((p) => String(p.id) === linea.productoId)}
				<div class="linea">
					<select
						class="control"
						aria-label={`Producto de la línea ${indice + 1}`}
						value={linea.productoId}
						onchange={(event) => elegirProducto(indice, event.currentTarget.value)}
					>
						<option value="" disabled>Elegí un producto</option>
						{#each productos as opcion (opcion.id)}
							<option value={String(opcion.id)}>{opcion.nombre} ({opcion.sku})</option>
						{/each}
					</select>
					<input
						class="control"
						aria-label={`Cantidad de la línea ${indice + 1}`}
						bind:value={linea.cantidad}
					/>
					<input
						class="control"
						aria-label={`Costo unitario de la línea ${indice + 1}`}
						bind:value={linea.costo}
						placeholder={producto ? String(producto.costo) : '0.00'}
					/>
					<span class="subtotal">{subtotal(linea)}</span>
					<Boton
						variante="fantasma"
						tamano="sm"
						icono="basura"
						deshabilitado={lineas.length === 1}
						onclick={() => quitarLinea(indice)}
					>
						Quitar
					</Boton>
				</div>
			{/each}
		</div>

		<div class="total">
			<span>Total estimado</span>
			<strong>{formatearMoneda(total)}</strong>
		</div>

		{#if error || errorValidacion}
			<Aviso variante="error">{error ?? errorValidacion}</Aviso>
		{/if}

		<div class="acciones">
			<Boton variante="secundario" deshabilitado={guardando} onclick={onCerrar}>Cancelar</Boton>
			<Boton tipo="submit" cargando={guardando}>
				{guardando ? 'Guardando…' : compra ? 'Guardar cambios' : 'Crear compra'}
			</Boton>
		</div>
	</form>
</Modal>

<style>
	.formulario {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.grilla {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
	}

	.lineas {
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		overflow: hidden;
	}

	.lineas-cabecera {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--borde);
		background: var(--superficie-2);
	}

	.titulo {
		color: var(--texto);
		font-size: 13px;
		font-weight: 600;
	}

	.linea-cabecera,
	.linea {
		display: grid;
		grid-template-columns: minmax(200px, 2fr) 100px 140px 120px 90px;
		gap: 8px;
		align-items: center;
		padding: 8px 12px;
	}

	.linea-cabecera {
		border-bottom: 1px solid var(--borde);
		color: var(--texto-tenue);
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
	}

	.linea {
		border-bottom: 1px solid var(--borde);
	}

	.linea:last-child {
		border-bottom: 0;
	}

	.derecha {
		text-align: right;
	}

	.control {
		width: 100%;
		min-height: 34px;
		padding: 5px 8px;
		border: 1px solid var(--borde-fuerte);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto);
		font-family: var(--fuente-sans);
		font-size: 12.5px;
	}

	.subtotal {
		text-align: right;
		color: var(--texto);
		font-size: 12.5px;
		font-variant-numeric: tabular-nums;
	}

	.total {
		display: flex;
		align-items: baseline;
		justify-content: flex-end;
		gap: 10px;
		color: var(--texto-suave);
		font-size: 13px;
	}

	.total strong {
		color: var(--texto);
		font-size: 17px;
		font-variant-numeric: tabular-nums;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	@media (max-width: 760px) {
		.grilla {
			grid-template-columns: 1fr;
		}

		.linea-cabecera {
			display: none;
		}

		.linea {
			grid-template-columns: 1fr 1fr;
		}
	}
</style>
