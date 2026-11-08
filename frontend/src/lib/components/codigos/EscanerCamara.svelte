<script lang="ts">
	import Modal from '#lib/components/ui/Modal.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import {
		FORMATOS_LECTURA,
		MENSAJES_CAMARA,
		clasificarErrorCamara,
		elegirLectura,
		type Deteccion,
		type ErrorCamara,
		type Lectura
	} from '#lib/codigos/escaner';

	// 020 (R1/R9): cámara trasera + ponyfill `barcode-detector` (zxing-wasm servido local).
	// Una lectura por activación; las pistas se apagan al leer, cancelar, fallar o desmontar.

	interface Props {
		abierto: boolean;
		oncerrar: () => void;
		onLectura: (lectura: Lectura) => void;
	}

	let { abierto, oncerrar, onLectura }: Props = $props();

	const INTERVALO_MS = 150;

	type Estado = 'solicitando' | 'buscando' | 'error';

	interface Detector {
		detect(fuente: HTMLVideoElement): Promise<Deteccion[]>;
	}

	let estado = $state<Estado>('solicitando');
	let tipoError = $state<ErrorCamara | null>(null);
	let video: HTMLVideoElement | undefined = $state();
	let intento = $state(0);

	let flujo: MediaStream | null = null;
	let temporizador: ReturnType<typeof setTimeout> | null = null;
	let detectorCache: Detector | null = null;

	const textoEstado = $derived(
		estado === 'solicitando'
			? 'Pidiendo permiso para usar la cámara…'
			: estado === 'buscando'
				? 'Buscando código… Apuntá al código de barras o QR.'
				: ''
	);

	async function crearDetector(): Promise<Detector> {
		if (detectorCache) return detectorCache;
		const [{ BarcodeDetector, setZXingModuleOverrides }, { default: urlWasm }] =
			await Promise.all([
				import('barcode-detector/ponyfill'),
				import('zxing-wasm/reader/zxing_reader.wasm?url')
			]);
		// Sin CDN: el wasm sale del propio servidor (funciona en la red local sin internet).
		setZXingModuleOverrides({
			locateFile: (ruta: string, prefijo: string) =>
				ruta.endsWith('.wasm') ? urlWasm : prefijo + ruta
		});
		detectorCache = new BarcodeDetector({ formats: [...FORMATOS_LECTURA] });
		return detectorCache;
	}

	function detener() {
		if (temporizador !== null) clearTimeout(temporizador);
		temporizador = null;
		flujo?.getTracks().forEach((pista) => pista.stop());
		flujo = null;
		if (video) video.srcObject = null;
	}

	function fallar(error: unknown) {
		detener();
		tipoError = clasificarErrorCamara(error, {
			contextoSeguro: window.isSecureContext,
			tieneMediaDevices: Boolean(navigator.mediaDevices?.getUserMedia)
		});
		estado = 'error';
	}

	async function iniciar(esVigente: () => boolean) {
		estado = 'solicitando';
		tipoError = null;
		if (!window.isSecureContext || !navigator.mediaDevices?.getUserMedia) {
			fallar(null);
			return;
		}
		try {
			const [nuevoFlujo, detector] = await Promise.all([
				navigator.mediaDevices.getUserMedia({
					video: { facingMode: { ideal: 'environment' } },
					audio: false
				}),
				crearDetector()
			]);
			if (!esVigente() || !video) {
				nuevoFlujo.getTracks().forEach((pista) => pista.stop());
				return;
			}
			flujo = nuevoFlujo;
			video.srcObject = nuevoFlujo;
			await video.play();
			if (!esVigente()) return;
			estado = 'buscando';
			void ciclo(detector, esVigente);
		} catch (error) {
			if (esVigente()) fallar(error);
		}
	}

	async function ciclo(detector: Detector, esVigente: () => boolean) {
		if (!esVigente() || !video || flujo === null) return;
		try {
			const lectura = elegirLectura(await detector.detect(video));
			if (lectura && esVigente()) {
				detener();
				onLectura(lectura);
				oncerrar();
				return;
			}
		} catch {
			// Un cuadro ilegible no es un error: se reintenta en el siguiente.
		}
		if (esVigente()) temporizador = setTimeout(() => void ciclo(detector, esVigente), INTERVALO_MS);
	}

	$effect(() => {
		if (!abierto || !video) return;
		void intento; // «Reintentar» vuelve a correr el efecto sin recargar (FR-152).
		let vigente = true;
		void iniciar(() => vigente);
		return () => {
			vigente = false;
			detener();
		};
	});

	function cancelar() {
		detener();
		oncerrar();
	}
</script>

<Modal
	{abierto}
	titulo="Escanear código"
	descripcion="Apuntá la cámara al código de barras o QR del producto."
	oncerrar={cancelar}
>
	<div class="escaner">
		<div class="visor" class:oculto={estado === 'error'}>
			<!-- svelte-ignore a11y_media_has_caption -->
			<video bind:this={video} playsinline muted></video>
			<div class="guia" aria-hidden="true"></div>
		</div>

		<p class="estado" aria-live="polite">{textoEstado}</p>

		{#if estado === 'error' && tipoError}
			<Aviso variante="advertencia" titulo="No se pudo usar la cámara">
				{MENSAJES_CAMARA[tipoError]}
			</Aviso>
		{/if}
	</div>

	{#snippet acciones()}
		{#if estado === 'error' && tipoError !== 'inseguro' && tipoError !== 'sin_soporte'}
			<Boton variante="secundario" onclick={() => (intento += 1)}>Reintentar</Boton>
		{/if}
		<Boton variante="fantasma" onclick={cancelar}>Cancelar</Boton>
	{/snippet}
</Modal>

<style>
	.escaner {
		display: grid;
		gap: 12px;
	}
	.visor {
		position: relative;
		overflow: hidden;
		border-radius: var(--radio-control);
		background: #000;
		aspect-ratio: 4 / 3;
	}
	.visor.oculto {
		display: none;
	}
	video {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}
	.guia {
		position: absolute;
		inset: 22% 12%;
		border: 2px solid var(--acento, #3fd0c9);
		border-radius: 10px;
		box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.35);
	}
	.estado {
		margin: 0;
		font-size: 13px;
		color: var(--texto-suave);
		min-height: 1.2em;
	}
	@media (prefers-reduced-motion: no-preference) {
		.guia {
			transition: border-color 0.2s;
		}
	}
</style>
