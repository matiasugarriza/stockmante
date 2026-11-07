<script lang="ts">
	import Boton from '../ui/Boton.svelte';
	import Modal from '../ui/Modal.svelte';

	/**
	 * Modal obligatoria de credenciales (FR-008b, escenarios 1 y 5): muestra el
	 * correo y la contraseña temporal en claro para copiar y entregar de
	 * inmediato. Es la única superficie donde la clave se ve; se cierra solo
	 * cuando el administrador la reconoce.
	 */
	interface Props {
		titulo: string;
		email: string;
		password: string;
		onCerrar: () => void;
	}

	let { titulo, email, password, onCerrar }: Props = $props();
	let copiado = $state(false);

	async function copiar() {
		try {
			await navigator.clipboard.writeText(password);
			copiado = true;
			setTimeout(() => (copiado = false), 2000);
		} catch {
			// Sin clipboard (contexto no seguro): el admin puede seleccionarla a mano.
			copiado = false;
		}
	}
</script>

<Modal
	abierto={true}
	titulo={titulo}
	descripcion="Entregá estas credenciales al usuario. También se envió por correo (en desarrollo, la consola del backend). Deberá cambiarla en su primer ingreso."
	oncerrar={onCerrar}
>
	<div class="credenciales">
		<div class="dato">
			<span>Correo</span>
			<code>{email}</code>
		</div>
		<div class="dato">
			<span>Contraseña temporal</span>
			<code class="clave">{password}</code>
		</div>
		<Boton variante="secundario" tamano="sm" onclick={() => void copiar()}>
			{copiado ? '¡Copiada!' : 'Copiar contraseña'}
		</Boton>
	</div>

	<div class="acciones">
		<Boton onclick={onCerrar}>Cerré las credenciales</Boton>
	</div>
</Modal>

<style>
	.credenciales {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 16px;
		padding: 14px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie-2);
	}

	.dato {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.dato span {
		color: var(--texto-tenue);
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	code {
		color: var(--texto);
		font-family: var(--fuente-mono);
		font-size: 13px;
		word-break: break-all;
	}

	.clave {
		font-weight: 700;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
	}
</style>
