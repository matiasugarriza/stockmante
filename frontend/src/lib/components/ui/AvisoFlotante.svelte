<script lang="ts">
	import { aviso, cerrarAviso } from '#lib/stores/avisos';
	import Icono from './Icono.svelte';
</script>

{#if $aviso}
	<div class="aviso {$aviso.variante}" role="status" aria-live="polite">
		<span class="icono" aria-hidden="true">
			<Icono nombre={$aviso.variante === 'exito' ? 'check' : 'alerta'} tamano={18} />
		</span>
		<p>{$aviso.mensaje}</p>
		<button type="button" class="cerrar" aria-label="Cerrar aviso" onclick={cerrarAviso}>
			<Icono nombre="cerrar" tamano={16} />
		</button>
	</div>
{/if}

<style>
	.aviso {
		position: fixed;
		left: 50%;
		bottom: 20px;
		z-index: 55;
		display: flex;
		align-items: center;
		gap: 10px;
		width: min(560px, calc(100vw - 24px));
		padding: 10px 12px;
		transform: translateX(-50%);
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta-movil);
		background: var(--superficie);
		color: var(--texto);
		box-shadow: var(--sombra-elevada);
		animation: destello 700ms ease-out 1;
	}

	.aviso.exito {
		border-color: var(--exito);
		box-shadow:
			0 0 0 1px var(--exito),
			0 0 26px -8px var(--exito);
	}

	.aviso.error {
		border-color: var(--peligro);
		box-shadow:
			0 0 0 1px var(--peligro),
			0 0 26px -8px var(--peligro);
	}

	.icono {
		display: inline-flex;
		flex: none;
		color: var(--exito-texto);
	}

	.aviso.error .icono {
		color: var(--peligro-texto);
	}

	p {
		flex: 1;
		margin: 0;
		font-size: 13px;
		line-height: 1.35;
	}

	.cerrar {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: var(--toque-min);
		height: var(--toque-min);
		margin: -8px -8px -8px 0;
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

	@keyframes destello {
		from {
			opacity: 0.72;
			transform: translateX(-50%) scale(0.985);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) scale(1);
		}
	}

	@media (max-width: 1023px) {
		.aviso {
			bottom: calc(var(--barra-inferior-alto) + var(--zona-segura-inferior) + 12px);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.aviso {
			animation: none;
		}
	}
</style>
