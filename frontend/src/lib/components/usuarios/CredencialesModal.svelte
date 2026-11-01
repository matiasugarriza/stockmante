<script lang="ts">
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

<div class="fixed inset-0 z-20 flex items-center justify-center bg-slate-900/50 p-4">
	<div
		class="w-full max-w-md rounded-xl border border-amber-200 bg-white p-5 shadow-lg"
		role="dialog"
		aria-modal="true"
		aria-label={titulo}
	>
		<h2 class="mb-1 text-lg font-semibold">{titulo}</h2>
		<p class="mb-4 text-xs text-slate-500">
			Entregá estas credenciales al usuario. También se envió por correo (en desarrollo,
			la consola del backend). Deberá cambiarla en su primer ingreso.
		</p>

		<div class="mb-4 flex flex-col gap-3 rounded border border-slate-200 bg-slate-50 p-3">
			<div>
				<p class="text-xs font-medium text-slate-500">Correo</p>
				<p class="break-all font-mono text-sm">{email}</p>
			</div>
			<div>
				<p class="text-xs font-medium text-slate-500">Contraseña temporal</p>
				<p class="break-all font-mono text-sm font-semibold">{password}</p>
			</div>
			<button
				type="button"
				class="self-start rounded border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-100"
				onclick={() => void copiar()}
			>
				{copiado ? '¡Copiada!' : 'Copiar contraseña'}
			</button>
		</div>

		<div class="flex justify-end">
			<button
				type="button"
				class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
				onclick={onCerrar}
			>
				Cerré las credenciales
			</button>
		</div>
	</div>
</div>
