<script lang="ts">
	import type { AccionPropuesta } from '#lib/schemas/asistente';
	import Boton from '../ui/Boton.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';

	interface Props {
		propuesta: AccionPropuesta;
		ocupado: boolean;
		onConfirmar: () => void;
		onCancelar: () => void;
	}

	let { propuesta, ocupado, onConfirmar, onCancelar }: Props = $props();

	const signo = $derived(propuesta.cantidad_delta > 0 ? '+' : '');
</script>

<section class="propuesta" aria-label="Propuesta del asistente">
	<header>
		<EtiquetaEstado variante="aviso">Pendiente de confirmación</EtiquetaEstado>
		<h2>Revisá los datos antes de confirmar</h2>
		<p>Todavía no se registró nada. La escritura se ejecuta recién cuando confirmás.</p>
	</header>

	<dl>
		<dt>Tipo</dt>
		<dd>{propuesta.tipo}</dd>
		<dt>Producto</dt>
		<dd>{propuesta.producto_nombre} <span class="sku">(SKU {propuesta.sku})</span></dd>
		<dt>Depósito</dt>
		<dd>{propuesta.deposito_nombre}</dd>
		<dt>Cantidad</dt>
		<dd class="cantidad">{signo}{propuesta.cantidad_delta}</dd>
		<dt>Motivo</dt>
		<dd>{propuesta.motivo}{propuesta.observacion ? ` — ${propuesta.observacion}` : ''}</dd>
	</dl>

	<div class="acciones">
		<Boton cargando={ocupado} onclick={onConfirmar}>
			{ocupado ? 'Registrando…' : 'Confirmar'}
		</Boton>
		<Boton variante="secundario" deshabilitado={ocupado} onclick={onCancelar}>Cancelar</Boton>
	</div>
</section>

<style>
	.propuesta {
		border: 1px solid color-mix(in oklab, var(--ambar) 40%, var(--borde));
		border-radius: var(--radio-tarjeta);
		background: var(--ambar-suave);
		padding: 16px 18px;
	}

	header {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
	}

	h2 {
		margin: 0;
		color: var(--ambar-texto);
		font-size: 14px;
		font-weight: 650;
	}

	p {
		margin: 0;
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 5px 16px;
		margin: 14px 0 0;
		padding: 12px;
		border-radius: var(--radio-control);
		background: color-mix(in oklab, var(--superficie) 70%, transparent);
		font-size: 13px;
	}

	dt {
		color: var(--texto-tenue);
	}

	dd {
		margin: 0;
		color: var(--texto);
	}

	.sku {
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
		font-size: 11px;
	}

	.cantidad {
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}

	.acciones {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 14px;
	}
</style>
