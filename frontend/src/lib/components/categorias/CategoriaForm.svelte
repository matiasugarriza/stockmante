<script lang="ts">
	import {
		ActualizarCategoriaSchema,
		CrearCategoriaSchema,
		type Categoria
	} from '#lib/schemas/categorias';

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

<div class="fixed inset-0 z-10 flex items-center justify-center bg-slate-900/40 p-4">
	<div class="w-full max-w-md rounded-xl border border-slate-200 bg-white p-5 shadow-lg">
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h2 class="text-lg font-semibold">
					{categoria ? 'Editar categoría' : 'Nueva categoría'}
				</h2>
				<p class="text-xs text-slate-500">
					El nombre es único sin distinguir mayúsculas ni espacios de extremo.
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
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Nombre</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={nombre}
					maxlength="80"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Descripción <span class="text-slate-400">(opcional)</span></span>
				<textarea
					class="min-h-20 rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={descripcion}
					maxlength="255"
				></textarea>
			</label>

			{#if error || errorValidacion}
				<p class="rounded bg-red-50 px-3 py-2 text-sm text-red-700">
					{error ?? errorValidacion}
				</p>
			{/if}

			<div class="mt-1 flex justify-end gap-2">
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
					{guardando ? 'Guardando…' : categoria ? 'Guardar cambios' : 'Crear categoría'}
				</button>
			</div>
		</form>
	</div>
</div>
