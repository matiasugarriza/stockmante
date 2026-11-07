<script lang="ts">
	import {
		ActualizarCategoriaSchema,
		CrearCategoriaSchema,
		type Categoria
	} from '#lib/schemas/categorias';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';

	interface Props {
		categoria: Categoria | null;
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: { nombre: string; descripcion: string | null }) => void;
		onCerrar: () => void;
	}

	let { categoria, guardando, error, onGuardar, onCerrar }: Props = $props();

	// El padre monta el formulario al abrirlo, con `categoria` ya definido.
	// svelte-ignore state_referenced_locally
	let nombre = $state(categoria?.nombre ?? '');
	// svelte-ignore state_referenced_locally
	let descripcion = $state(categoria?.descripcion ?? '');
	let errorValidacion = $state<string | null>(null);

	function enviar() {
		errorValidacion = null;
		const esquema = categoria ? ActualizarCategoriaSchema : CrearCategoriaSchema;
		const datos = esquema.safeParse({ nombre, descripcion });
		if (!datos.success) {
			const primero = datos.error.issues[0];
			errorValidacion = primero ? primero.message : 'Revisá los datos';
			return;
		}
		onGuardar({
			nombre: datos.data.nombre ?? nombre,
			descripcion: datos.data.descripcion ?? null
		});
	}
</script>

<Modal
	abierto={true}
	titulo={categoria ? 'Editar categoría' : 'Nueva categoría'}
	descripcion="El nombre es único sin distinguir mayúsculas ni espacios de extremo."
	oncerrar={onCerrar}
>
	<form
		class="formulario"
		onsubmit={(event) => {
			event.preventDefault();
			enviar();
		}}
	>
		<Campo
			etiqueta="Nombre"
			nombre="nombre"
			maxlength={80}
			bind:valor={nombre}
			requerido={true}
		/>
		<Campo
			etiqueta="Descripción"
			nombre="descripcion"
			tipo="textarea"
			maxlength={255}
			ayuda="Opcional. Máximo 255 caracteres."
			bind:valor={descripcion}
		/>

		{#if error || errorValidacion}
			<Aviso variante="error">{error ?? errorValidacion}</Aviso>
		{/if}

		<div class="acciones">
			<Boton variante="secundario" deshabilitado={guardando} onclick={onCerrar}>Cancelar</Boton>
			<Boton tipo="submit" cargando={guardando}>
				{guardando ? 'Guardando…' : categoria ? 'Guardar cambios' : 'Crear categoría'}
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

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
	}
</style>
