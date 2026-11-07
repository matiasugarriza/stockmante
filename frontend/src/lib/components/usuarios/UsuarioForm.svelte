<script lang="ts">
	import { CrearUsuarioSchema, type Rol, type Usuario } from '#lib/schemas/usuarios';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';

	interface Props {
		usuario: Usuario | null;
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: { nombre: string; email: string; rol: Rol; password?: string }) => void;
		onCerrar: () => void;
	}

	let { usuario, guardando, error, onGuardar, onCerrar }: Props = $props();

	// El padre monta este formulario recién al abrirlo (con `usuario` ya definido),
	// así que la captura inicial de las props es intencional.
	// svelte-ignore state_referenced_locally
	let nombre = $state(usuario?.nombre ?? '');
	// svelte-ignore state_referenced_locally
	let email = $state(usuario?.email ?? '');
	// svelte-ignore state_referenced_locally
	let rol = $state<Rol>(usuario?.rol ?? 'OPERARIO');
	// FR-008b: elección manual/automática (solo en el alta; editar no toca claves).
	let metodoPassword = $state<'auto' | 'manual'>('auto');
	let passwordManual = $state('');
	let errorValidacion = $state<string | null>(null);

	const roles: { valor: Rol; etiqueta: string }[] = [
		{ valor: 'ADMIN', etiqueta: 'Administrador (ADMIN)' },
		{ valor: 'GERENTE', etiqueta: 'Gerente (GERENTE)' },
		{ valor: 'OPERARIO', etiqueta: 'Operario (OPERARIO)' },
		{ valor: 'CAJERO', etiqueta: 'Cajero (CAJERO)' }
	];

	function enviar() {
		errorValidacion = null;
		const base = { nombre, email, rol };
		const datos = CrearUsuarioSchema.safeParse(
			metodoPassword === 'manual' && !usuario
				? { ...base, password: passwordManual }
				: base
		);
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
	titulo={usuario ? 'Editar usuario' : 'Nuevo usuario'}
	descripcion={usuario
		? undefined
		: 'Al crearla vas a ver la contraseña en pantalla; también se envía por correo.'}
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
			maxlength={120}
			bind:valor={nombre}
			requerido={true}
		/>
		<Campo
			etiqueta="Email"
			nombre="email"
			tipo="email"
			autocomplete="off"
			maxlength={255}
			bind:valor={email}
			requerido={true}
		/>
		<Campo
			etiqueta="Rol"
			nombre="rol"
			tipo="select"
			opciones={roles}
			bind:valor={rol}
		/>

		{#if !usuario}
			<fieldset class="claves">
				<legend>Contraseña</legend>
				<label class="radio">
					<input
						type="radio"
						name="metodo-password"
						value="auto"
						checked={metodoPassword === 'auto'}
						onchange={() => (metodoPassword = 'auto')}
					/>
					<span>Automática (se genera de 14 caracteres)</span>
				</label>
				<label class="radio">
					<input
						type="radio"
						name="metodo-password"
						value="manual"
						checked={metodoPassword === 'manual'}
						onchange={() => (metodoPassword = 'manual')}
					/>
					<span>Manual (la ingreso yo)</span>
				</label>
				{#if metodoPassword === 'manual'}
					<Campo
						etiqueta="Contraseña manual"
						nombre="password_manual"
						autocomplete="off"
						ayuda="Mínimo 8 caracteres, 1 mayúscula y 1 dígito"
						bind:valor={passwordManual}
						requerido={true}
					/>
				{/if}
			</fieldset>
		{/if}

		{#if error || errorValidacion}
			<Aviso variante="error">{error ?? errorValidacion}</Aviso>
		{/if}

		<div class="acciones">
			<Boton variante="secundario" deshabilitado={guardando} onclick={onCerrar}>Cancelar</Boton>
			<Boton tipo="submit" cargando={guardando}>
				{guardando ? 'Guardando…' : usuario ? 'Guardar cambios' : 'Crear usuario'}
			</Boton>
		</div>
	</form>
</Modal>

<style>
	.formulario {
		display: flex;
		flex-direction: column;
		gap: 13px;
	}

	.claves {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 10px 12px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
	}

	legend {
		padding: 0 4px;
		color: var(--texto-suave);
		font-size: 12px;
		font-weight: 600;
	}

	.radio {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--texto);
		font-size: 13px;
	}

	.radio input {
		accent-color: var(--primario);
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 2px;
	}
</style>
