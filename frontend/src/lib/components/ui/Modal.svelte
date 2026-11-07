<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tick } from 'svelte';
	import Icono from './Icono.svelte';

	interface Props {
		abierto: boolean;
		titulo: string;
		descripcion?: string;
		tamano?: 'md' | 'lg';
		oncerrar: () => void;
		children: Snippet;
		acciones?: Snippet;
	}

	let { abierto, titulo, descripcion, tamano = 'md', oncerrar, children, acciones }: Props = $props();

	let dialogo: HTMLDivElement | undefined = $state();
	let disparador: HTMLElement | null = null;
	const idTitulo = `modal-${Math.random().toString(36).slice(2, 9)}`;
	const idDescripcion = `${idTitulo}-desc`;

	$effect(() => {
		if (!abierto) return;
		disparador = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		const overflowPrevio = document.body.style.overflow;
		document.body.style.overflow = 'hidden';
		void tick().then(() => {
			const primero = dialogo?.querySelector<HTMLElement>(
				'input, select, textarea, button, a[href]'
			);
			(primero ?? dialogo)?.focus();
		});
		return () => {
			document.body.style.overflow = overflowPrevio;
			disparador?.focus();
		};
	});

	function alTeclear(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			oncerrar();
			return;
		}
		if (event.key !== 'Tab' || !dialogo) return;
		const enfocables = Array.from(
			dialogo.querySelectorAll<HTMLElement>(
				'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
			)
		).filter((elemento) => elemento.offsetParent !== null || elemento === document.activeElement);
		if (enfocables.length === 0) {
			event.preventDefault();
			dialogo.focus();
			return;
		}
		const primero = enfocables[0];
		const ultimo = enfocables[enfocables.length - 1];
		if (event.shiftKey && document.activeElement === primero) {
			event.preventDefault();
			ultimo?.focus();
		} else if (!event.shiftKey && document.activeElement === ultimo) {
			event.preventDefault();
			primero?.focus();
		}
	}
</script>

{#if abierto}
	<div class="capa">
		<button type="button" class="fondo" aria-label="Cerrar diálogo" onclick={oncerrar}></button>
		<div
			class="dialogo"
			class:lg={tamano === 'lg'}
			role="dialog"
			aria-modal="true"
			aria-labelledby={idTitulo}
			aria-describedby={descripcion ? idDescripcion : undefined}
			bind:this={dialogo}
			tabindex="-1"
			onkeydown={alTeclear}
		>
			<header class="cabecera">
				<div>
					<h2 id={idTitulo}>{titulo}</h2>
					{#if descripcion}<p id={idDescripcion}>{descripcion}</p>{/if}
				</div>
				<button type="button" class="cerrar" aria-label="Cerrar" onclick={oncerrar}>
					<Icono nombre="cerrar" tamano={18} />
				</button>
			</header>
			<div class="cuerpo">{@render children()}</div>
			{#if acciones}<footer class="acciones">{@render acciones()}</footer>{/if}
		</div>
	</div>
{/if}

<style>
	.capa {
		position: fixed;
		inset: 0;
		z-index: 60;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 20px;
	}

	.fondo {
		position: absolute;
		inset: 0;
		border: 0;
		background: rgba(9, 14, 24, 0.55);
		cursor: pointer;
	}

	.dialogo {
		position: relative;
		width: min(560px, 100%);
		max-height: calc(100vh - 40px);
		overflow: auto;
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta);
		background: var(--superficie);
		box-shadow: var(--sombra-elevada);
	}

	.dialogo.lg {
		width: min(880px, 100%);
	}

	.cabecera {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		padding: 18px 20px 0;
	}

	h2 {
		margin: 0;
		color: var(--texto);
		font-size: 17px;
		font-weight: 650;
		letter-spacing: -0.01em;
	}

	p {
		margin: 4px 0 0;
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	.cerrar {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: var(--radio-control);
		background: transparent;
		color: var(--texto-suave);
		cursor: pointer;
	}

	.cerrar:hover {
		background: var(--superficie-2);
		color: var(--texto);
	}

	.cuerpo {
		padding: 16px 20px 20px;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		border-top: 1px solid var(--borde);
		padding: 12px 20px;
		background: var(--superficie-2);
	}
</style>
