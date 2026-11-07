<script lang="ts">
	import { goto } from '$app/navigation';
	import { tick } from 'svelte';
	import { sesion } from '#lib/stores/sesion';
	import { buscarInventario } from '#lib/api/inventario';
	import { crearBusquedaDiferida } from '#lib/utils/busquedaCaja';
	import { mensajeError } from '#lib/utils/errores';
	import type { StockItem } from '#lib/schemas/movimientos';
	import Aviso from '../ui/Aviso.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import Icono from '../ui/Icono.svelte';
	import TarjetaInventario from './TarjetaInventario.svelte';

	interface Props {
		abierto: boolean;
		oncerrar: () => void;
	}

	let { abierto, oncerrar }: Props = $props();

	let texto = $state('');
	let items = $state<StockItem[]>([]);
	let cargando = $state(false);
	let error = $state<string | null>(null);
	let buscado = $state(false);
	let campo = $state<HTMLInputElement>();

	const busqueda = crearBusquedaDiferida(250);
	const familiaRol = $derived(
		$sesion.estado === 'activa' ? $sesion.usuario.familia_rol : 'OPERACION'
	);

	function reiniciar() {
		texto = '';
		items = [];
		cargando = false;
		error = null;
		buscado = false;
		busqueda.cancelar();
	}

	$effect(() => {
		if (abierto) {
			void tick().then(() => campo?.focus());
		} else {
			reiniciar();
		}
	});

	function alEscribir(valor: string) {
		texto = valor;
		error = null;
		const limpio = valor.trim();
		if (limpio.length < 2) {
			items = [];
			buscado = false;
			cargando = false;
			busqueda.cancelar();
			return;
		}
		cargando = true;
		busqueda.programar(async (esVigente) => {
			try {
				const resultado = await buscarInventario(limpio);
				if (!esVigente()) return;
				items = resultado;
				buscado = true;
			} catch (fallo) {
				if (!esVigente()) return;
				items = [];
				buscado = true;
				error = mensajeError(fallo, 'No se pudo buscar el producto.');
			} finally {
				if (esVigente()) cargando = false;
			}
		});
	}

	function elegir(item: StockItem) {
		oncerrar();
		const destino = familiaRol === 'GESTION' ? '/admin/stock' : '/caja';
		void goto(`${destino}?q=${encodeURIComponent(item.sku)}`);
	}
</script>

{#if abierto}
	<div class="capa" role="dialog" aria-modal="true" aria-label="Buscar producto">
		<header class="barra">
			<button type="button" class="cerrar" aria-label="Cerrar búsqueda" onclick={oncerrar}>
				<Icono nombre="cerrar" tamano={20} />
			</button>
			<input
				bind:this={campo}
				type="search"
				class="campo"
				placeholder="SKU, código de barras o nombre"
				aria-label="Buscar por SKU, código de barras o nombre"
				autocomplete="off"
				enterkeyhint="search"
				value={texto}
				oninput={(event) => alEscribir(event.currentTarget.value)}
			/>
		</header>

		<div class="cuerpo">
			{#if error}
				<Aviso variante="error" titulo="No se pudo buscar">{error}</Aviso>
			{:else if cargando}
				<div class="cargando" aria-busy="true" aria-label="Buscando">
					{#each [1, 2, 3] as numero (numero)}
						<div class="esqueleto"></div>
					{/each}
				</div>
			{:else if items.length > 0}
				<p class="conteo">{items.length} resultados</p>
				<ul class="resultados">
					{#each items as item (item.producto_id)}
						<li>
							<TarjetaInventario
								nombre={item.nombre}
								sku={item.sku}
								stockActual={item.stock_actual}
								bajoMinimo={item.bajo_minimo}
								contexto={item.categoria_nombre}
								onabrir={() => elegir(item)}
							/>
						</li>
					{/each}
				</ul>
			{:else if buscado}
				<EstadoVacio
					icono="buscar"
					titulo="Sin resultados"
					descripcion="Probá con otro SKU, código de barras o parte del nombre."
				/>
			{:else}
				<p class="ayuda">Buscá por SKU, código de barras o nombre del producto.</p>
			{/if}
		</div>
	</div>
{/if}

<style>
	.capa {
		position: fixed;
		inset: 0;
		z-index: 58;
		display: flex;
		flex-direction: column;
		background: var(--fondo);
	}

	.barra {
		display: flex;
		flex: none;
		align-items: center;
		gap: 8px;
		padding: calc(var(--zona-segura-superior) + 8px) var(--espacio-movil) 8px;
		border-bottom: 1px solid var(--borde);
		background: var(--fondo-elevado);
	}

	.cerrar {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: var(--toque-min);
		height: var(--toque-min);
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto-suave);
		cursor: pointer;
	}

	.campo {
		flex: 1;
		min-width: 0;
		height: var(--toque-min);
		padding: 0 12px;
		border: 1px solid var(--borde-control);
		border-radius: var(--radio-control);
		background: var(--superficie);
		color: var(--texto);
		font-family: var(--fuente-sans);
		font-size: 16px;
	}

	.campo::placeholder {
		color: var(--texto-tenue);
	}

	.campo:focus-visible {
		border-color: var(--primario);
		outline: 2px solid var(--foco);
		outline-offset: 1px;
	}

	.cuerpo {
		flex: 1;
		overflow-y: auto;
		padding: 12px var(--espacio-movil) calc(var(--barra-inferior-alto) + 24px);
		overscroll-behavior: contain;
	}

	.conteo {
		margin: 0 0 8px;
		color: var(--texto-tenue);
		font-size: 12px;
	}

	.resultados {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.cargando {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.esqueleto {
		height: 86px;
		border-radius: var(--radio-tarjeta-movil);
		background: var(--superficie-2);
	}

	.ayuda {
		margin: 24px 0 0;
		color: var(--texto-suave);
		font-size: 13px;
		text-align: center;
	}
</style>
