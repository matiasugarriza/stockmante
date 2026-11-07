<script lang="ts">
	import {
		CrearProveedorSchema,
		type CrearProveedorInput,
		type Proveedor
	} from '#lib/schemas/proveedores';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';

	interface Props {
		proveedor: Proveedor | null;
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: CrearProveedorInput) => void;
		onCerrar: () => void;
	}

	let { proveedor, guardando, error, onGuardar, onCerrar }: Props = $props();

	// El padre monta el formulario al abrirlo, con `proveedor` ya definido.
	// svelte-ignore state_referenced_locally
	let nombre = $state(proveedor?.nombre ?? '');
	// svelte-ignore state_referenced_locally
	let cuit = $state(proveedor?.cuit ?? '');
	// svelte-ignore state_referenced_locally
	let email = $state(proveedor?.email ?? '');
	// svelte-ignore state_referenced_locally
	let telefono = $state(proveedor?.telefono ?? '');
	// svelte-ignore state_referenced_locally
	let direccion = $state(proveedor?.direccion ?? '');
	let errorValidacion = $state<string | null>(null);

	function enviar() {
		errorValidacion = null;
		const datos = CrearProveedorSchema.safeParse({ nombre, cuit, email, telefono, direccion });
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
	titulo={proveedor ? 'Editar proveedor' : 'Nuevo proveedor'}
	descripcion="El nombre es único. El CUIT es opcional, pero si se informa debe tener 11 dígitos."
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
				maxlength={120}
				bind:valor={nombre}
				requerido={true}
			/>
		</div>
		<Campo etiqueta="CUIT" nombre="cuit" marcador="20123456789" maxlength={11} bind:valor={cuit} />
		<Campo etiqueta="Email" nombre="email" tipo="email" maxlength={200} bind:valor={email} />
		<Campo etiqueta="Teléfono" nombre="telefono" maxlength={60} bind:valor={telefono} />
		<Campo etiqueta="Dirección" nombre="direccion" maxlength={500} bind:valor={direccion} />

		{#if error || errorValidacion}
			<div class="ancho">
				<Aviso variante="error">{error ?? errorValidacion}</Aviso>
			</div>
		{/if}

		<div class="acciones ancho">
			<Boton variante="secundario" deshabilitado={guardando} onclick={onCerrar}>Cancelar</Boton>
			<Boton tipo="submit" cargando={guardando}>
				{guardando ? 'Guardando…' : proveedor ? 'Guardar cambios' : 'Crear proveedor'}
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
