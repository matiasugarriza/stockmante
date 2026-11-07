<script lang="ts">
	import { CrearProveedorSchema, type CrearProveedorInput, type Proveedor } from '#lib/schemas/proveedores';

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

<div class="fixed inset-0 z-10 flex items-center justify-center bg-slate-900/40 p-4">
	<div class="max-h-full w-full max-w-xl overflow-y-auto rounded-xl border border-slate-200 bg-white p-5 shadow-lg">
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h2 class="text-lg font-semibold">{proveedor ? 'Editar proveedor' : 'Nuevo proveedor'}</h2>
				<p class="text-xs text-slate-500">
					El nombre es único. El CUIT es opcional, pero si se informa debe tener 11 dígitos.
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
			class="grid grid-cols-1 gap-3 sm:grid-cols-2"
			onsubmit={(e) => {
				e.preventDefault();
				enviar();
			}}
		>
			<label class="flex flex-col gap-1 text-sm sm:col-span-2">
				<span class="font-medium">Nombre</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={nombre}
					maxlength="120"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">CUIT <span class="text-slate-400">(opcional)</span></span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={cuit}
					inputmode="numeric"
					maxlength="11"
					placeholder="20123456789"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Email <span class="text-slate-400">(opcional)</span></span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={email}
					type="email"
					maxlength="200"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Teléfono <span class="text-slate-400">(opcional)</span></span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={telefono}
					maxlength="60"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Dirección <span class="text-slate-400">(opcional)</span></span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={direccion}
					maxlength="500"
				/>
			</label>

			{#if error || errorValidacion}
				<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700 sm:col-span-2">
					{error ?? errorValidacion}
				</p>
			{/if}

			<div class="mt-1 flex justify-end gap-2 sm:col-span-2">
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
					{guardando ? 'Guardando...' : proveedor ? 'Guardar cambios' : 'Crear proveedor'}
				</button>
			</div>
		</form>
	</div>
</div>