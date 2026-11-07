<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Opcion {
		valor: string;
		etiqueta: string;
	}

	interface Props {
		etiqueta: string;
		nombre: string;
		tipo?: 'text' | 'email' | 'password' | 'number' | 'search' | 'date' | 'select' | 'textarea';
		valor?: string;
		opciones?: Opcion[];
		error?: string | null;
		ayuda?: string;
		marcador?: string;
		requerido?: boolean;
		deshabilitado?: boolean;
		autocomplete?: 'username' | 'current-password' | 'new-password' | 'email' | 'off' | 'on';
		min?: number | string;
		max?: number | string;
		step?: number | string;
		maxlength?: number;
		oninput?: (valor: string) => void;
		onchange?: (valor: string) => void;
		onkeydown?: (event: KeyboardEvent) => void;
		children?: Snippet;
	}

	let {
		etiqueta,
		nombre,
		tipo = 'text',
		valor = $bindable(''),
		opciones = [],
		error = null,
		ayuda,
		marcador,
		requerido = false,
		deshabilitado = false,
		autocomplete,
		min,
		max,
		step,
		maxlength,
		oninput,
		onchange,
		onkeydown,
		children
	}: Props = $props();

	const idError = $derived(`${nombre}-error`);
	const idAyuda = $derived(`${nombre}-ayuda`);
	const descrito = $derived([error ? idError : '', ayuda ? idAyuda : ''].filter(Boolean).join(' '));

	function alEscribir(event: Event) {
		const objetivo = event.currentTarget as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement;
		valor = objetivo.value;
		oninput?.(objetivo.value);
	}

	function alCambiar(event: Event) {
		const objetivo = event.currentTarget as HTMLSelectElement;
		valor = objetivo.value;
		onchange?.(objetivo.value);
	}
</script>

<div class="campo" class:con-error={Boolean(error)}>
	<label for={nombre}>{etiqueta}{#if requerido}<span class="requerido" aria-hidden="true">*</span>{/if}</label>

	{#if tipo === 'select'}
		<select
			id={nombre}
			name={nombre}
			disabled={deshabilitado}
			required={requerido}
			value={valor}
			onchange={alCambiar}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={descrito || undefined}
		>
			{#each opciones as opcion}
				<option value={opcion.valor}>{opcion.etiqueta}</option>
			{/each}
		</select>
	{:else if tipo === 'textarea'}
		<textarea
			id={nombre}
			name={nombre}
			disabled={deshabilitado}
			required={requerido}
			placeholder={marcador}
			{maxlength}
			value={valor}
			oninput={alEscribir}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={descrito || undefined}
			rows="3"
		></textarea>
	{:else}
		<input
			id={nombre}
			name={nombre}
			type={tipo}
			disabled={deshabilitado}
			required={requerido}
			placeholder={marcador}
			{autocomplete}
			{min}
			{max}
			{step}
			{maxlength}
			value={valor}
			oninput={alEscribir}
			onkeydown={onkeydown}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={descrito || undefined}
		/>
	{/if}

	{#if error}
		<p class="mensaje error" id={idError} role="alert">{error}</p>
	{:else if ayuda}
		<p class="mensaje ayuda" id={idAyuda}>{ayuda}</p>
	{/if}

	{#if children}{@render children()}{/if}
</div>

<style>
	.campo {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}

	label {
		color: var(--texto);
		font-size: 13px;
		font-weight: 500;
		line-height: 1.3;
	}

	.requerido {
		margin-left: 2px;
		color: var(--peligro-texto);
	}

	input,
	select,
	textarea {
		width: 100%;
		min-height: 40px;
		padding: 8px 12px;
		border: 1px solid var(--borde-fuerte);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto);
		font-family: var(--fuente-sans);
		font-size: 14px;
		line-height: 1.4;
		transition:
			border-color 120ms ease,
			box-shadow 120ms ease;
	}

	textarea {
		min-height: 84px;
		resize: vertical;
	}

	select {
		appearance: none;
		padding-right: 34px;
		background-image:
			linear-gradient(45deg, transparent 50%, var(--texto-suave) 50%),
			linear-gradient(135deg, var(--texto-suave) 50%, transparent 50%);
		background-position:
			calc(100% - 17px) 50%,
			calc(100% - 12px) 50%;
		background-repeat: no-repeat;
		background-size: 5px 5px;
		cursor: pointer;
	}

	input::placeholder,
	textarea::placeholder {
		color: var(--texto-tenue);
		opacity: 1;
	}

	input:hover:not(:disabled),
	select:hover:not(:disabled),
	textarea:hover:not(:disabled) {
		border-color: var(--texto-tenue);
	}

	input:focus-visible,
	select:focus-visible,
	textarea:focus-visible {
		border-color: var(--primario);
		outline: none;
		box-shadow: 0 0 0 3px color-mix(in oklab, var(--primario) 22%, transparent);
	}

	input:disabled,
	select:disabled,
	textarea:disabled {
		background: var(--superficie-2);
		color: var(--texto-tenue);
		cursor: not-allowed;
	}

	.con-error input,
	.con-error select,
	.con-error textarea {
		border-color: var(--peligro);
	}

	.con-error input:focus-visible,
	.con-error select:focus-visible,
	.con-error textarea:focus-visible {
		box-shadow: 0 0 0 3px color-mix(in oklab, var(--peligro) 22%, transparent);
	}

	.mensaje {
		margin: 0;
		font-size: 12px;
		line-height: 1.35;
	}

	.error {
		color: var(--peligro-texto);
	}

	.ayuda {
		color: var(--texto-tenue);
	}

	/* Teclado táctil: 16 px evita el zoom automático y 44 px el toque perdido (FR-106). */
	@media (max-width: 1023px) {
		input,
		select,
		textarea {
			min-height: var(--toque-min);
			font-size: 16px;
		}

		textarea {
			min-height: 96px;
		}

		label {
			font-size: 13.5px;
		}
	}
</style>
