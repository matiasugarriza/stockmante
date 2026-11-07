<script lang="ts">
	import type { MovimientoDetalle, TipoMovimiento } from '#lib/schemas/movimientos';
	import { formatearFechaCorta, formatearNumero } from '#lib/utils/formato';
	import Aviso from '../ui/Aviso.svelte';
	import Boton from '../ui/Boton.svelte';
	import Esqueleto from '../ui/Esqueleto.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';

	interface Props {
		movimientos: MovimientoDetalle[];
		cargando?: boolean;
		error?: string | null;
		onreintentar?: () => void;
	}

	let { movimientos, cargando = false, error = null, onreintentar }: Props = $props();

	const variantes: Record<TipoMovimiento, 'exito' | 'info' | 'aviso' | 'neutro'> = {
		ENTRADA: 'exito',
		SALIDA: 'info',
		MERMA: 'aviso',
		AJUSTE: 'neutro'
	};

	function resumen(movimiento: MovimientoDetalle): { producto: string; cantidad: number } {
		const primera = movimiento.lineas[0];
		const cantidad = movimiento.lineas.reduce((suma, linea) => suma + linea.cantidad_delta, 0);
		return { producto: primera?.producto_nombre ?? 'Sin líneas', cantidad };
	}
</script>

{#if error}
	<Aviso variante="error" titulo="No se pudieron cargar los movimientos">{error}</Aviso>
	{#if onreintentar}
		<div class="reintentar">
			<Boton variante="secundario" tamano="sm" onclick={onreintentar}>Reintentar</Boton>
		</div>
	{/if}
{:else if cargando}
	<div class="cargando">
		{#each Array(4) as _, indice (indice)}
			<Esqueleto alto="38px" />
		{/each}
	</div>
{:else if movimientos.length === 0}
	<EstadoVacio
		icono="movimientos"
		titulo="Sin movimientos recientes"
		descripcion="Cuando registres el primer movimiento vas a verlo acá."
	/>
{:else}
	<ul class="lista">
		{#each movimientos as movimiento (movimiento.id)}
			{@const dato = resumen(movimiento)}
			<li>
				<a href="/movimientos" class="fila">
					<div class="principal">
						<EtiquetaEstado variante={variantes[movimiento.tipo]}>{movimiento.tipo}</EtiquetaEstado>
						<span class="producto">{dato.producto}</span>
						<span class="cantidad" class:positivo={dato.cantidad > 0}>
							{dato.cantidad > 0 ? '+' : ''}{formatearNumero(dato.cantidad)}
						</span>
					</div>
					<div class="meta">
						<span>{movimiento.deposito.nombre}</span>
						<span>{formatearFechaCorta(movimiento.creado_en)}</span>
					</div>
				</a>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.lista {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.fila {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding: 9px 0;
		border-bottom: 1px solid var(--borde);
		color: var(--texto);
		text-decoration: none;
	}

	.lista li:last-child .fila {
		border-bottom: 0;
	}

	.fila:hover .producto {
		color: var(--primario-texto);
	}

	.principal {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.producto {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 13px;
		font-weight: 500;
	}

	.cantidad {
		flex: none;
		color: var(--peligro-texto);
		font-size: 12px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}

	.cantidad.positivo {
		color: var(--exito-texto);
	}

	.meta {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		color: var(--texto-tenue);
		font-size: 11.5px;
	}

	.cargando {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.reintentar {
		margin-top: 8px;
	}
</style>
