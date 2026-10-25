<script lang="ts">
	import type { Usuario } from '#lib/schemas/usuarios';

	interface Props {
		usuarios: Usuario[];
		onEditar: (usuario: Usuario) => void;
		onAlternarEstado: (usuario: Usuario) => void;
		onRegenerar: (usuario: Usuario) => void;
	}

	let { usuarios, onEditar, onAlternarEstado, onRegenerar }: Props = $props();

	const botonAccion =
		'rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100 disabled:opacity-60';
</script>

<div class="overflow-x-auto rounded-lg border border-slate-200 bg-white">
	<table class="min-w-full divide-y divide-slate-200 text-sm">
		<thead class="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
			<tr>
				<th class="px-4 py-2 font-medium">Nombre</th>
				<th class="px-4 py-2 font-medium">Email</th>
				<th class="px-4 py-2 font-medium">Rol</th>
				<th class="px-4 py-2 font-medium">Estado</th>
				<th class="px-4 py-2 text-right font-medium">Acciones</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each usuarios as usuario (usuario.id)}
				<tr>
					<td class="px-4 py-2 font-medium">{usuario.nombre}</td>
					<td class="px-4 py-2 text-slate-600">{usuario.email}</td>
					<td class="px-4 py-2">
						<span class="rounded bg-slate-100 px-1.5 py-0.5 text-xs text-slate-600">
							{usuario.rol}
						</span>
					</td>
					<td class="px-4 py-2">
						<span
							class="rounded px-1.5 py-0.5 text-xs {usuario.activo
								? 'bg-emerald-50 text-emerald-700'
								: 'bg-slate-100 text-slate-500'}"
						>
							{usuario.activo ? 'Activo' : 'Inactivo'}
						</span>
						{#if usuario.password_temporal}
							<span class="ml-1 rounded bg-amber-50 px-1.5 py-0.5 text-xs text-amber-700">
								temporal
							</span>
						{/if}
					</td>
					<td class="px-4 py-2">
						<div class="flex flex-wrap justify-end gap-1">
							<button class={botonAccion} onclick={() => onEditar(usuario)}>Editar</button>
							{#if usuario.activo}
								<button class={botonAccion} onclick={() => onAlternarEstado(usuario)}>
									Desactivar
								</button>
							{:else}
								<button class={botonAccion} onclick={() => onAlternarEstado(usuario)}>
									Reactivar
								</button>
							{/if}
							<button class={botonAccion} onclick={() => onRegenerar(usuario)}>
								Nueva temporal
							</button>
						</div>
					</td>
				</tr>
			{/each}
			{#if usuarios.length === 0}
				<tr>
					<td colspan="5" class="px-4 py-8 text-center text-slate-500">
						No hay usuarios para este filtro.
					</td>
				</tr>
			{/if}
		</tbody>
	</table>
</div>
