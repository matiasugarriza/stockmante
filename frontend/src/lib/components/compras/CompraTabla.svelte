<script lang="ts">
	import type { Compra } from '#lib/schemas/compras';

	interface Props {
		compras: Compra[];
		onEditar: (compra: Compra) => void;
		onConfirmar: (compra: Compra) => void;
		onAnular: (compra: Compra) => void;
		onEliminar: (compra: Compra) => void;
	}

	let { compras, onEditar, onConfirmar, onAnular, onEliminar }: Props = $props();

	function fecha(valor: string): string {
		const fecha = new Date(valor);
		return Number.isNaN(fecha.getTime()) ? valor : fecha.toLocaleDateString('es-AR');
	}

	const clasesEstado: Record<string, string> = {
		BORRADOR: 'bg-amber-100 text-amber-700',
		CONFIRMADA: 'bg-emerald-100 text-emerald-700',
		ANULADA: 'bg-slate-200 text-slate-600'
	};
</script>

<div class="overflow-x-auto rounded border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-100">
			<tr>
				<th class="p-2 text-left">ID</th>
				<th class="p-2 text-left">Fecha</th>
				<th class="p-2 text-left">Proveedor</th>
				<th class="p-2 text-left">Depósito</th>
				<th class="p-2 text-left">Número</th>
				<th class="p-2 text-left">Estado</th>
				<th class="p-2 text-right">Total</th>
				<th class="p-2 text-right">Acciones</th>
			</tr>
		</thead>
		<tbody>
			{#each compras as compra (compra.id)}
				<tr class="border-t border-slate-100 align-top">
					<td class="p-2 text-slate-500">{compra.id}</td>
					<td class="p-2">{fecha(compra.creado_en)}</td>
					<td class="p-2 font-medium">{compra.proveedor_nombre}</td>
					<td class="p-2">{compra.deposito_nombre}</td>
					<td class="p-2">{compra.numero ?? '—'}</td>
					<td class="p-2">
						<span class="rounded px-2 py-0.5 text-xs {clasesEstado[compra.estado] ?? ''}">
							{compra.estado}
						</span>
					</td>
					<td class="p-2 text-right">{compra.total_costo.toFixed(2)}</td>
					<td class="p-2">
						<div class="flex justify-end gap-2">
							{#if compra.estado === 'BORRADOR'}
								<button
									type="button"
									class="rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100"
									onclick={() => onEditar(compra)}
								>
									Editar
								</button>
								<button
									type="button"
									class="rounded border border-emerald-300 px-2 py-1 text-xs text-emerald-700 hover:bg-emerald-50"
									onclick={() => onConfirmar(compra)}
								>
									Confirmar
								</button>
								<button
									type="button"
									class="rounded border border-red-300 px-2 py-1 text-xs text-red-700 hover:bg-red-50"
									onclick={() => onEliminar(compra)}
								>
									Eliminar
								</button>
							{:else if compra.estado === 'CONFIRMADA'}
								<button
									type="button"
									class="rounded border border-red-300 px-2 py-1 text-xs text-red-700 hover:bg-red-50"
									onclick={() => onAnular(compra)}
								>
									Anular
								</button>
							{:else}
								<span class="text-xs text-slate-400">Sin acciones</span>
							{/if}
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>