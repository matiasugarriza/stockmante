<script lang="ts">
	import { CrearUsuarioSchema, type Rol, type Usuario } from '#lib/schemas/usuarios';

	interface Props {
		usuario: Usuario | null;
		guardando: boolean;
		error: string | null;
		onGuardar: (datos: { nombre: string; email: string; rol: Rol }) => void;
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
	let errorValidacion = $state<string | null>(null);

	const roles: { valor: Rol; etiqueta: string }[] = [
		{ valor: 'ADMIN', etiqueta: 'Administrador' },
		{ valor: 'GERENTE', etiqueta: 'Gerente' },
		{ valor: 'OPERARIO', etiqueta: 'Operario' },
		{ valor: 'CAJERO', etiqueta: 'Cajero' }
	];

	function enviar() {
		errorValidacion = null;
		const datos = CrearUsuarioSchema.safeParse({ nombre, email, rol });
		if (!datos.success) {
			const primero = datos.error.issues[0];
			errorValidacion = primero ? primero.message : 'Revisá los datos';
			return;
		}
		onGuardar(datos.data);
	}
</script>

<div class="fixed inset-0 z-10 flex items-center justify-center bg-slate-900/40 p-4">
	<div class="w-full max-w-md rounded-xl border border-slate-200 bg-white p-5 shadow-lg">
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h2 class="text-lg font-semibold">
					{usuario ? 'Editar usuario' : 'Nuevo usuario'}
				</h2>
				{#if !usuario}
					<p class="text-xs text-slate-500">
						Se genera una contraseña temporal y se envía por correo.
					</p>
				{/if}
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
					maxlength="120"
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Email</span>
				<input
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					type="email"
					autocomplete="off"
					bind:value={email}
					required
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">Rol</span>
				<select
					class="rounded border border-slate-300 px-3 py-2 focus:border-slate-500 focus:outline-none"
					bind:value={rol}
				>
					{#each roles as opcion (opcion.valor)}
						<option value={opcion.valor}>{opcion.etiqueta} ({opcion.valor})</option>
					{/each}
				</select>
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
					{guardando ? 'Guardando…' : usuario ? 'Guardar cambios' : 'Crear usuario'}
				</button>
			</div>
		</form>
	</div>
</div>
