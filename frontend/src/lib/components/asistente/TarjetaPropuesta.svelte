<script lang="ts">
	import type { AccionPropuesta } from '#lib/schemas/asistente';

	interface Props {
		propuesta: AccionPropuesta;
		ocupado: boolean;
		onConfirmar: () => void;
		onCancelar: () => void;
	}

	let { propuesta, ocupado, onConfirmar, onCancelar }: Props = $props();

	const signo = $derived(propuesta.cantidad_delta > 0 ? '+' : '');
</script>

<section
	class="rounded-lg border border-amber-300 bg-amber-50 p-4 text-sm"
	aria-label="Propuesta del asistente"
>
	<p class="font-semibold text-amber-900">Propuesta pendiente de confirmación</p>
	<p class="mt-1 text-amber-800">Todavía no se registró nada. Revisá los datos antes de confirmar.</p>
	<dl class="mt-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
		<dt class="text-slate-500">Tipo</dt>
		<dd>{propuesta.tipo}</dd>
		<dt class="text-slate-500">Producto</dt>
		<dd>{propuesta.producto_nombre} <span class="text-slate-500">(SKU {propuesta.sku})</span></dd>
		<dt class="text-slate-500">Depósito</dt>
		<dd>{propuesta.deposito_nombre}</dd>
		<dt class="text-slate-500">Cantidad</dt>
		<dd>{signo}{propuesta.cantidad_delta}</dd>
		<dt class="text-slate-500">Motivo</dt>
		<dd>{propuesta.motivo}{propuesta.observacion ? ` — ${propuesta.observacion}` : ''}</dd>
	</dl>
	<div class="mt-4 flex gap-2">
		<button
			class="rounded bg-amber-600 px-3 py-1.5 font-medium text-white hover:bg-amber-700 disabled:opacity-50"
			disabled={ocupado}
			onclick={onConfirmar}
		>
			{ocupado ? 'Registrando…' : 'Confirmar'}
		</button>
		<button
			class="rounded border border-slate-300 px-3 py-1.5 hover:bg-white disabled:opacity-50"
			disabled={ocupado}
			onclick={onCancelar}
		>
			Cancelar
		</button>
	</div>
</section>
