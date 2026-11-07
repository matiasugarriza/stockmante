<script lang="ts">
	import type { Usuario } from '#lib/schemas/usuarios';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Campo from '../ui/Campo.svelte';
	import Modal from '../ui/Modal.svelte';

	/**
	 * Diálogo de regeneración de contraseña (escenario 5, FR-008b): mismo
	 * selector manual/automática que el alta; al confirmar, el padre pide la
	 * clave y muestra la CredencialesModal con la nueva.
	 */
	interface Props {
		usuario: Usuario;
		guardando: boolean;
		error: string | null;
		onGenerar: (password?: string) => void;
		onCerrar: () => void;
	}

	let { usuario, guardando, error, onGenerar, onCerrar }: Props = $props();

	let metodoPassword = $state<'auto' | 'manual'>('auto');
	let passwordManual = $state('');
	let errorValidacion = $state<string | null>(null);

	function enviar() {
		errorValidacion = null;
		if (metodoPassword === 'manual') {
			const cumple =
				passwordManual.length >= 8 &&
				/[A-Z]/.test(passwordManual) &&
				/[0-9]/.test(passwordManual);
			if (!cumple) {
				errorValidacion =
					'La contraseña debe tener al menos 8 caracteres, una mayúscula y un dígito';
				return;
			}
			onGenerar(passwordManual);
			return;
		}
		onGenerar(undefined);
	}
</script>

<Modal
	abierto={true}
	titulo="Regenerar contraseña"
	descripcion={`${usuario.nombre} · ${usuario.email} — se revocan sus sesiones vigentes.`}
	oncerrar={onCerrar}
>
	<form
		class="formulario"
		onsubmit={(event) => {
			event.preventDefault();
			enviar();
		}}
	>
		<fieldset class="claves">
			<legend>Contraseña</legend>
			<label class="radio">
				<input
					type="radio"
					name="metodo-password-regenerar"
					value="auto"
					checked={metodoPassword === 'auto'}
					onchange={() => (metodoPassword = 'auto')}
				/>
				<span>Automática (se genera de 14 caracteres)</span>
			</label>
			<label class="radio">
				<input
					type="radio"
					name="metodo-password-regenerar"
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

		{#if error || errorValidacion}
			<Aviso variante="error">{error ?? errorValidacion}</Aviso>
		{/if}

		<div class="acciones">
			<Boton variante="secundario" deshabilitado={guardando} onclick={onCerrar}>Cancelar</Boton>
			<Boton tipo="submit" cargando={guardando}>
				{guardando ? 'Generando…' : 'Generar contraseña'}
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
