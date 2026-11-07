<script lang="ts">
	import type { Proveedor } from '#lib/schemas/proveedores';

	interface Props {
		proveedores: Proveedor[];
		onEditar: (proveedor: Proveedor) => void;
		onAlternarEstado: (proveedor: Proveedor) => void;
	}

	let { proveedores, onEditar, onAlternarEstado }: Props = $props();
</script>

<div class="overflow-x-auto rounded border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-100">
			<tr>
				<th class="p-2 text-left">ID</th>
				<th class="p-2 text-left">Nombre</th>
				<th class="p-2 text-left">CUIT</th>
				<th class="p-2 text-left">Email</th>
				<th class="p-2 text-left">Teléfono</th>
				<th class="p-2 text-left">Estado</th>
				<th class="p-2 text-right">Acciones</th>
			</tr>
		</thead>
		<tbody>
			{#each proveedores as proveedor (proveedor.id)}
				<tr class="border-t border-slate-100 align-top">
					<td class="p-2 text-slate-500">{proveedor.id}</td>
					<td class="p-2 font-medium">{proveedor.nombre}</td>
					<td class="p-2">{proveedor.cuit ?? '—'}</td>
					<td class="p-2">{proveedor.email ?? '—'}</td>
					<td class="p-2">{proveedor.telefono ?? '—'}</td>
					<td class="p-2">
						<span
							class="rounded px-2 py-0.5 text-xs {proveedor.activo
								? 'bg-emerald-100 text-emerald-700'
								: 'bg-slate-200 text-slate-600'}"
						>
							{proveedor.activo ? 'Activo' : 'Inactivo'}
						</span>
					</td>
					<td class="p-2">
						<div class="flex justify-end gap-2">
							<button
								type="button"
								class="rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100"
								onclick={() => onEditar(proveedor)}
							>
								Editar
							</button>
							<button
								type="button"
								class="rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100"
								onclick={() => onAlternarEstado(proveedor)}
							>
								{proveedor.activo ? 'Desactivar' : 'Reactivar'}
							</button>
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>