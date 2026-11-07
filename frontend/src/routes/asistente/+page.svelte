<script lang="ts">
	import { ApiError } from '#lib/api/client';
	import { agregarAlHistorial, confirmarPropuesta, consultarAsistente } from '#lib/api/asistente';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import Esqueleto from '#lib/components/ui/Esqueleto.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
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
		return e instanceof ApiError
			? e.message
			: 'No se pudo hablar con el asistente. Revisá tu conexión e intentá de nuevo.';
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

<div class="pagina">
	<EncabezadoPagina
		titulo="Asistente de inventario"
		descripcion="Preguntá por stock, productos o ventas, o pedí registrar un movimiento. Ningún movimiento se registra sin tu confirmación."
	/>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}

	<Tarjeta>
		{#if historial.length === 0 && !enviando}
			<EstadoVacio
				icono="asistente"
				titulo="¿Qué querés consultar?"
				descripcion="Por ejemplo: «¿cuánto stock hay de yerba?» o «registrá una merma de 2 unidades en Principal»."
			/>
		{:else}
			<ol class="conversacion" aria-live="polite">
				{#each historial as mensaje, indice (indice)}
					<li class="mensaje" class:usuario={mensaje.rol === 'usuario'}>
						{mensaje.texto}
					</li>
				{/each}
				{#if enviando}
					<li class="pensando">
						<Esqueleto alto="14px" ancho="180px" />
						<span>El asistente está pensando…</span>
					</li>
				{/if}
			</ol>
		{/if}
	</Tarjeta>

	{#if propuesta}
		<TarjetaPropuesta {propuesta} ocupado={confirmando} onConfirmar={confirmar} onCancelar={cancelar} />
	{/if}

	<Tarjeta densa={true}>
		<form class="compositor" onsubmit={enviar}>
			<div class="campo-mensaje">
				<Campo
					etiqueta="Mensaje"
					nombre="mensaje-asistente"
					marcador="¿Cuánto stock hay de yerba?"
					maxlength={2000}
					deshabilitado={enviando}
					bind:valor={texto}
				/>
			</div>
			<Boton tipo="submit" cargando={enviando} deshabilitado={!texto.trim()}>Enviar</Boton>
		</form>
	</Tarjeta>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.conversacion {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.mensaje {
		max-width: min(680px, 90%);
		align-self: flex-start;
		padding: 9px 12px;
		border: 1px solid var(--borde);
		border-radius: 10px 10px 10px 3px;
		background: var(--superficie-2);
		color: var(--texto);
		font-size: 13.5px;
		line-height: 1.5;
		white-space: pre-wrap;
	}

	.mensaje.usuario {
		align-self: flex-end;
		border-color: transparent;
		border-radius: 10px 10px 3px 10px;
		background: var(--primario);
		color: var(--primario-contraste);
	}

	.pensando {
		display: flex;
		align-items: center;
		gap: 8px;
		align-self: flex-start;
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	.compositor {
		display: flex;
		align-items: flex-end;
		gap: 10px;
	}

	.campo-mensaje {
		flex: 1;
		min-width: 0;
	}

	@media (max-width: 560px) {
		.compositor {
			flex-direction: column;
			align-items: stretch;
		}
	}
</style>
