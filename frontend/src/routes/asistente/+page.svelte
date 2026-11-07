<script lang="ts">
	import { ApiError } from '#lib/api/client';
	import { agregarAlHistorial, confirmarPropuesta, consultarAsistente } from '#lib/api/asistente';
	import TarjetaPropuesta from '#lib/components/asistente/TarjetaPropuesta.svelte';
	import type { AccionPropuesta, MensajeHistorial } from '#lib/schemas/asistente';

	// La conversación vive solo en memoria: al recargar o salir se pierde (FR-077).
	// Nada de localStorage ni sessionStorage.
	let historial = $state<MensajeHistorial[]>([]);
	let texto = $state('');
	let propuesta = $state<AccionPropuesta | null>(null);
	let enviando = $state(false);
	let confirmando = $state(false);
	let error = $state<string | null>(null);

	function mensajeDe(e: unknown): string {
		return e instanceof ApiError ? e.message : 'No se pudo hablar con el asistente. Revisá tu conexión e intentá de nuevo.';
	}

	async function enviar(evento: SubmitEvent) {
		evento.preventDefault();
		const mensaje = texto.trim();
		if (!mensaje || enviando) return;
		error = null;
		enviando = true;
		const previo = historial;
		historial = agregarAlHistorial(historial, { rol: 'usuario', texto: mensaje });
		texto = '';
		try {
			const res = await consultarAsistente(mensaje, previo);
			historial = agregarAlHistorial(historial, { rol: 'asistente', texto: res.respuesta });
			propuesta = res.accion_propuesta;
		} catch (e) {
			error = mensajeDe(e);
		} finally {
			enviando = false;
		}
	}

	async function confirmar() {
		if (!propuesta) return;
		error = null;
		confirmando = true;
		try {
			const movimiento = await confirmarPropuesta(propuesta);
			const linea = movimiento.lineas[0];
			// FR-076: resultado concreto con producto, depósito y stock resultante.
			historial = agregarAlHistorial(historial, {
				rol: 'asistente',
				texto:
					`Registrado: ${movimiento.tipo} de ${Math.abs(linea.cantidad_delta)} u. de ${linea.producto_nombre} ` +
					`en ${movimiento.deposito.nombre}. Stock resultante: ${linea.stock_resultante}.`
			});
			propuesta = null;
		} catch (e) {
			error = mensajeDe(e);
		} finally {
			confirmando = false;
		}
	}

	function cancelar() {
		propuesta = null;
		historial = agregarAlHistorial(historial, {
			rol: 'asistente',
			texto: 'Cancelado: no se registró ningún movimiento.'
		});
	}
</script>

<svelte:head>
	<title>Asistente · Stockmante</title>
</svelte:head>

<main class="mx-auto flex max-w-3xl flex-col gap-4 p-4">
	<header>
		<h1 class="text-xl font-semibold">Asistente de inventario</h1>
		<p class="text-sm text-slate-500">
			Preguntá por stock, productos o ventas, o pedí registrar un movimiento. Ningún movimiento se
			registra sin tu confirmación.
		</p>
	</header>

	<ol class="flex flex-col gap-2" aria-live="polite">
		{#each historial as mensaje, i (i)}
			<li
				class={mensaje.rol === 'usuario'
					? 'self-end rounded-lg bg-slate-800 px-3 py-2 text-sm whitespace-pre-wrap text-white'
					: 'self-start rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm whitespace-pre-wrap'}
			>
				{mensaje.texto}
			</li>
		{/each}
		{#if enviando}
			<li class="self-start text-sm text-slate-500">El asistente está pensando…</li>
		{/if}
	</ol>

	{#if propuesta}
		<TarjetaPropuesta {propuesta} ocupado={confirmando} onConfirmar={confirmar} onCancelar={cancelar} />
	{/if}

	{#if error}
		<p class="rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{error}</p>
	{/if}

	<form class="flex gap-2" onsubmit={enviar}>
		<label class="sr-only" for="mensaje-asistente">Mensaje</label>
		<input
			id="mensaje-asistente"
			class="flex-1 rounded border border-slate-300 px-3 py-2 text-sm"
			placeholder="¿Cuánto stock hay de yerba?"
			maxlength="2000"
			autocomplete="off"
			bind:value={texto}
			disabled={enviando}
		/>
		<button
			class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50"
			disabled={enviando || !texto.trim()}
		>
			Enviar
		</button>
	</form>
</main>
