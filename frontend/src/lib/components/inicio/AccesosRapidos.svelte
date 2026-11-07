<script lang="ts">
	import Icono from '../ui/Icono.svelte';
	import type { NombreIcono } from '../ui/iconos';

	interface Props {
		esGestion: boolean;
	}

	let { esGestion }: Props = $props();

	interface Acceso {
		titulo: string;
		detalle: string;
		ruta: string;
		icono: NombreIcono;
	}

	const accesos = $derived<Acceso[]>([
		{ titulo: 'Caja', detalle: 'Buscar productos y vender', ruta: '/caja', icono: 'caja' },
		{
			titulo: 'Movimientos',
			detalle: 'Registrar entradas, salidas y mermas',
			ruta: '/movimientos',
			icono: 'movimientos'
		},
		{
			titulo: 'Transferencias',
			detalle: 'Mover stock entre depósitos',
			ruta: '/transferencias',
			icono: 'transferencias'
		},
		esGestion
			? {
					titulo: 'Productos',
					detalle: 'Gestionar el catálogo maestro',
					ruta: '/admin/productos',
					icono: 'productos' as NombreIcono
				}
			: {
					titulo: 'Asistente',
					detalle: 'Preguntar en lenguaje natural',
					ruta: '/asistente',
					icono: 'asistente' as NombreIcono
				}
	]);
</script>

<div class="accesos">
	{#each accesos as acceso (acceso.ruta)}
		<a class="acceso" href={acceso.ruta}>
			<span class="marca" aria-hidden="true"><Icono nombre={acceso.icono} tamano={18} /></span>
			<span class="texto">
				<strong>{acceso.titulo}</strong>
				<small>{acceso.detalle}</small>
			</span>
			<span class="flecha" aria-hidden="true"><Icono nombre="flecha" tamano={15} /></span>
		</a>
	{/each}
</div>

<style>
	.accesos {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
		gap: 10px;
	}

	.acceso {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 11px 12px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto);
		text-decoration: none;
		transition:
			border-color 120ms ease,
			background-color 120ms ease;
	}

	.acceso:hover {
		border-color: var(--primario);
		background: var(--superficie-2);
	}

	.marca {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 34px;
		height: 34px;
		border-radius: 8px;
		background: var(--primario-suave);
		color: var(--primario-texto);
	}

	.texto {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
	}

	.texto strong {
		font-size: 13px;
		font-weight: 600;
	}

	.texto small {
		color: var(--texto-tenue);
		font-size: 11.5px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.flecha {
		flex: none;
		color: var(--texto-tenue);
	}
</style>
