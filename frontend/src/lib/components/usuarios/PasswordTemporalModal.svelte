<script lang="ts">
	import type { Usuario } from '#lib/schemas/usuarios';

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
				errorValidacion = 'La contraseña debe tener al menos 8 caracteres, una mayúscula y un dígito';
				return;
			}
			onGenerar(passwordManual);
			return;
		}
		onGenerar(undefined);
	}
</script>

<div class="fixed inset-0 z-20 flex items-center justify-center bg-slate-900/50 p-4">
	<div
		class="w-full max-w-md rounded-xl border border-slate-200 bg-white p-5 shadow-lg"
		role="dialog"
		aria-modal="true"
		aria-label="Regenerar contraseña"
	>
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h2 class="text-lg font-semibold">Regenerar contraseña</h2>
				<p class="text-xs text-slate-500">
					{usuario.nombre} · {usuario.email} — se revocan sus sesiones vigentes.
				</p>
			</div>
			<button
				type="button"
				class="rounded px-2 py-1 text-sm text-slate-500 hover:bg-slate-100"
				onclick={onCerrar}
				disabled={guardando}
			>
				Cerrar
			</button>
		</div>

		<form
			class="flex flex-col gap-3"
			onsubmit={(e) => {
				e.preventDefault();
				enviar();
			}}
		>
			<fieldset class="flex flex-col gap-2 rounded border border-slate-200 p-3 text-sm">
				<legend class="px-1 font-medium">Contraseña</legend>
				<label class="flex items-center gap-2">
					<input
						type="radio"
						name="metodo-password-regenerar"
						value="auto"
						checked={metodoPassword === 'auto'}
						onchange={() => (metodoPassword = 'auto')}
					/>
					<span>Automática (se genera de 14 caracteres)</span>
				</label>
				<label class="flex items-center gap-2">
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
					<label class="flex flex-col gap-1">
						<span class="text-xs text-slate-500">
							Mínimo 8 caracteres, 1 mayúscula y 1 dígito
						</span>
						<input
							class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
							type="text"
							autocomplete="off"
							bind:value={passwordManual}
							required
						/>
					</label>
				{/if}
			</fieldset>

			{#if error || errorValidacion}
				<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700">
					{error ?? errorValidacion}
				</p>
			{/if}

			<div class="flex justify-end gap-2">
				<button
					type="button"
					class="rounded border border-slate-300 px-3 py-2 text-sm hover:bg-slate-100"
					onclick={onCerrar}
					disabled={guardando}
				>
					Cancelar
				</button>
				<button
					type="submit"
					class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-60"
					disabled={guardando}
				>
					{guardando ? 'Generando...' : 'Generar contraseña'}
				</button>
			</div>
		</form>
	</div>
</div>
