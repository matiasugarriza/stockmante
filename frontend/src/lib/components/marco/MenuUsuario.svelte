<script lang="ts">
	import { goto } from '$app/navigation';
	import { cerrarSesion, sesion } from '#lib/stores/sesion';
	import Icono from '../ui/Icono.svelte';

	const usuario = $derived($sesion.estado === 'activa' ? $sesion.usuario : null);
	const iniciales = $derived(
		usuario
			? usuario.nombre
					.split(/\s+/)
					.filter(Boolean)
					.slice(0, 2)
					.map((parte) => parte[0]?.toUpperCase() ?? '')
					.join('')
			: '?'
	);

	async function salir() {
		await cerrarSesion();
		await goto('/login');
	}
</script>

{#if usuario}
	<details class="menu">
		<summary aria-label={`Menú de ${usuario.nombre}`}>
			<span class="avatar" aria-hidden="true">{iniciales}</span>
			<span class="datos">
				<strong>{usuario.nombre}</strong>
				<small>{usuario.rol}</small>
			</span>
			<span class="flecha" aria-hidden="true"><Icono nombre="flecha" tamano={14} /></span>
		</summary>
		<div class="panel">
			<div class="info">
				<strong>{usuario.nombre}</strong>
				<span>{usuario.email}</span>
				<span class="rol">{usuario.rol}</span>
			</div>
			<button type="button" class="salir" onclick={() => void salir()}>
				<Icono nombre="salir" tamano={16} />
				Cerrar sesión
			</button>
		</div>
	</details>
{/if}

<style>
	.menu {
		position: relative;
	}

	summary {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 6px;
		border-radius: var(--radio-control);
		list-style: none;
		cursor: pointer;
	}

	summary::-webkit-details-marker {
		display: none;
	}

	summary:hover {
		background: var(--superficie-2);
	}

	.avatar {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		background: var(--primario-suave);
		color: var(--primario-texto);
		font-size: 11px;
		font-weight: 700;
	}

	.datos {
		display: flex;
		flex-direction: column;
		line-height: 1.2;
	}

	.datos strong {
		color: var(--texto);
		font-size: 12.5px;
		font-weight: 600;
	}

	.datos small {
		color: var(--texto-tenue);
		font-size: 10.5px;
		letter-spacing: 0.04em;
	}

	.flecha {
		display: inline-flex;
		color: var(--texto-tenue);
	}

	.panel {
		position: absolute;
		top: calc(100% + 8px);
		right: 0;
		z-index: 40;
		width: 248px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-tarjeta);
		background: var(--superficie);
		box-shadow: var(--sombra-elevada);
		overflow: hidden;
	}

	.info {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding: 14px 16px;
		border-bottom: 1px solid var(--borde);
	}

	.info strong {
		color: var(--texto);
		font-size: 13px;
	}

	.info span {
		color: var(--texto-suave);
		font-size: 12px;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.rol {
		align-self: flex-start;
		margin-top: 3px;
		padding: 1px 7px;
		border-radius: var(--radio-pildora);
		background: var(--superficie-3);
		color: var(--texto-suave);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	.salir {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 11px 16px;
		border: 0;
		background: transparent;
		color: var(--texto-suave);
		font-family: var(--fuente-sans);
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}

	.salir:hover {
		background: var(--peligro-suave);
		color: var(--peligro-texto);
	}

	@media (max-width: 720px) {
		.datos {
			display: none;
		}
	}
</style>
