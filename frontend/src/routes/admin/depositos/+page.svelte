<script lang="ts">
	import { ApiError } from '#lib/api/client';
	import {
		actualizarDeposito,
		actualizarUbicacion,
		crearDeposito,
		crearUbicacion,
		desactivarDeposito,
		desactivarUbicacion,
		eliminarDeposito,
		eliminarUbicacion,
		listarDepositos,
		listarUbicaciones,
		reactivarDeposito,
		reactivarUbicacion,
		reemplazarUbicacionesDeProducto,
		ubicacionesDeProducto
	} from '#lib/api/depositos';
	import { buscarProductos } from '#lib/api/productos';
	import type { Deposito, TipoUbicacion, Ubicacion } from '#lib/schemas/depositos';
	import type { Producto } from '#lib/schemas/productos';
	import { sesion } from '#lib/stores/sesion';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import Esqueleto from '#lib/components/ui/Esqueleto.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import EtiquetaEstado from '#lib/components/ui/EtiquetaEstado.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';

	const TIPOS: { valor: TipoUbicacion; etiqueta: string }[] = [
		{ valor: 'PASILLO', etiqueta: 'Pasillo' },
		{ valor: 'ESTANTERIA', etiqueta: 'Estantería' },
		{ valor: 'NIVEL', etiqueta: 'Nivel' },
		{ valor: 'POSICION', etiqueta: 'Posición' }
	];

	interface NodoUbicacion {
		ubicacion: Ubicacion;
		profundidad: number;
	}

	interface GrupoUbicaciones {
		deposito: Deposito;
		ubicaciones: Ubicacion[];
	}

	let depositos = $state<Deposito[]>([]);
	let ubicaciones = $state<Ubicacion[]>([]);
	let cargando = $state(false);
	let cargado = $state(false);
	let guardando = $state(false);
	let error = $state<string | null>(null);
	let aviso = $state<string | null>(null);

	// Formulario de depósito: `editandoDeposito` alterna alta y edición.
	let editandoDeposito = $state<Deposito | null>(null);
	let nombreDeposito = $state('');
	let identificador = $state('');
	let direccion = $state('');

	let seleccionado = $state<Deposito | null>(null);

	// Formulario de ubicación.
	let editandoUbicacion = $state<Ubicacion | null>(null);
	let nombreUbicacion = $state('');
	let tipoUbicacion = $state<TipoUbicacion>('PASILLO');
	let padreUbicacion = $state('');

	// Asignación producto–ubicación.
	let busqueda = $state('');
	let resultados = $state<Producto[]>([]);
	let buscando = $state(false);
	let producto = $state<Producto | null>(null);
	let grupos = $state<GrupoUbicaciones[]>([]);
	let seleccion = $state<number[]>([]);
	let cargandoAsignacion = $state(false);

	const arbol = $derived(aplanarArbol(ubicaciones));

	const opcionesPadre = $derived(
		arbol.filter((nodo) => nodo.ubicacion.id !== editandoUbicacion?.id)
	);

	$effect(() => {
		if (
			!cargado &&
			$sesion.estado === 'activa' &&
			$sesion.usuario.familia_rol === 'GESTION'
		) {
			cargado = true;
			void cargarDepositos();
		}
	});

	function aplanarArbol(items: Ubicacion[]): NodoUbicacion[] {
		const porPadre = new Map<number | null, Ubicacion[]>();
		for (const ubicacion of items) {
			const clave = ubicacion.padre_id ?? null;
			const lista = porPadre.get(clave) ?? [];
			lista.push(ubicacion);
			porPadre.set(clave, lista);
		}
		for (const lista of porPadre.values()) {
			lista.sort((a, b) => a.nombre.localeCompare(b.nombre));
		}
		const resultado: NodoUbicacion[] = [];
		const visitados = new Set<number>();
		const visitar = (padre: number | null, profundidad: number) => {
			for (const ubicacion of porPadre.get(padre) ?? []) {
				if (visitados.has(ubicacion.id)) continue;
				visitados.add(ubicacion.id);
				resultado.push({ ubicacion, profundidad });
				visitar(ubicacion.id, profundidad + 1);
			}
		};
		visitar(null, 0);
		return resultado;
	}

	function mensajeError(e: unknown, fallback: string): string {
		if (e instanceof ApiError) return e.message;
		if (e instanceof Error) return e.message;
		return fallback;
	}

	function limpiarMensajes() {
		error = null;
		aviso = null;
	}

	async function cargarDepositos() {
		cargando = true;
		try {
			depositos = await listarDepositos('all');
			if (seleccionado) {
				seleccionado = depositos.find((d) => d.id === seleccionado?.id) ?? null;
			}
			if (seleccionado) {
				await cargarUbicaciones(seleccionado);
			}
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los depósitos');
		} finally {
			cargando = false;
		}
	}

	async function cargarUbicaciones(deposito: Deposito) {
		try {
			ubicaciones = await listarUbicaciones(deposito.id);
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar las ubicaciones');
		}
	}

	async function seleccionar(deposito: Deposito) {
		limpiarMensajes();
		seleccionado = deposito;
		editandoUbicacion = null;
		nombreUbicacion = '';
		padreUbicacion = '';
		await cargarUbicaciones(deposito);
	}

	function limpiarFormularioDeposito() {
		editandoDeposito = null;
		nombreDeposito = '';
		identificador = '';
		direccion = '';
	}

	function editarDeposito(deposito: Deposito) {
		limpiarMensajes();
		editandoDeposito = deposito;
		nombreDeposito = deposito.nombre;
		identificador = deposito.identificador;
		direccion = deposito.direccion ?? '';
	}

	async function guardarDeposito() {
		limpiarMensajes();
		guardando = true;
		try {
			if (editandoDeposito) {
				const actualizado = await actualizarDeposito(editandoDeposito.id, {
					nombre: nombreDeposito,
					identificador,
					direccion: direccion.trim() === '' ? null : direccion
				});
				aviso = `Depósito ${actualizado.nombre} actualizado.`;
				if (seleccionado?.id === actualizado.id) seleccionado = actualizado;
			} else {
				const creado = await crearDeposito({
					nombre: nombreDeposito,
					identificador,
					direccion: direccion.trim() === '' ? undefined : direccion
				});
				aviso = `Depósito ${creado.nombre} creado.`;
				seleccionado = creado;
			}
			limpiarFormularioDeposito();
			await cargarDepositos();
		} catch (e) {
			error = mensajeError(e, 'No se pudo guardar el depósito');
		} finally {
			guardando = false;
		}
	}

	async function alternarEstadoDeposito(deposito: Deposito) {
		limpiarMensajes();
		try {
			if (deposito.activo) {
				await desactivarDeposito(deposito.id);
				aviso = `${deposito.nombre} quedó inactivo. Sus ubicaciones no se modificaron.`;
			} else {
				await reactivarDeposito(deposito.id);
				aviso = `${deposito.nombre} volvió a estar activo.`;
			}
			await cargarDepositos();
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado del depósito');
		}
	}

	async function borrarDeposito(deposito: Deposito) {
		if (!confirm(`¿Borrar el depósito ${deposito.nombre}? Solo se permite sin ubicaciones.`)) {
			return;
		}
		limpiarMensajes();
		try {
			await eliminarDeposito(deposito.id);
			aviso = 'Depósito borrado.';
			if (seleccionado?.id === deposito.id) {
				seleccionado = null;
				ubicaciones = [];
			}
			await cargarDepositos();
		} catch (e) {
			error = mensajeError(e, 'No se pudo borrar el depósito');
		}
	}

	function limpiarFormularioUbicacion() {
		editandoUbicacion = null;
		nombreUbicacion = '';
		tipoUbicacion = 'PASILLO';
		padreUbicacion = '';
	}

	function editarUbicacion(ubicacion: Ubicacion) {
		limpiarMensajes();
		editandoUbicacion = ubicacion;
		nombreUbicacion = ubicacion.nombre;
		tipoUbicacion = ubicacion.tipo;
		padreUbicacion = ubicacion.padre_id === null ? '' : String(ubicacion.padre_id);
	}

	function agregarHija(ubicacion: Ubicacion) {
		limpiarMensajes();
		editandoUbicacion = null;
		nombreUbicacion = '';
		tipoUbicacion = 'NIVEL';
		padreUbicacion = String(ubicacion.id);
	}

	async function guardarUbicacion() {
		if (!seleccionado) return;
		limpiarMensajes();
		guardando = true;
		const padre = padreUbicacion === '' ? null : Number(padreUbicacion);
		try {
			if (editandoUbicacion) {
				await actualizarUbicacion(editandoUbicacion.id, {
					nombre: nombreUbicacion,
					tipo: tipoUbicacion,
					padre_id: padre
				});
				aviso = 'Ubicación actualizada.';
			} else {
				await crearUbicacion({
					deposito_id: seleccionado.id,
					padre_id: padre,
					nombre: nombreUbicacion,
					tipo: tipoUbicacion
				});
				aviso = 'Ubicación creada.';
			}
			limpiarFormularioUbicacion();
			await cargarUbicaciones(seleccionado);
		} catch (e) {
			error = mensajeError(e, 'No se pudo guardar la ubicación');
		} finally {
			guardando = false;
		}
	}

	async function alternarEstadoUbicacion(ubicacion: Ubicacion) {
		limpiarMensajes();
		try {
			if (ubicacion.activo) {
				await desactivarUbicacion(ubicacion.id);
				aviso = `${ubicacion.nombre} quedó inactiva.`;
			} else {
				await reactivarUbicacion(ubicacion.id);
				aviso = `${ubicacion.nombre} volvió a estar activa.`;
			}
			if (seleccionado) await cargarUbicaciones(seleccionado);
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado de la ubicación');
		}
	}

	async function borrarUbicacion(ubicacion: Ubicacion) {
		if (!confirm(`¿Borrar la ubicación ${ubicacion.nombre}? Solo se permite sin hijos.`)) {
			return;
		}
		limpiarMensajes();
		try {
			await eliminarUbicacion(ubicacion.id);
			aviso = 'Ubicación borrada.';
			if (editandoUbicacion?.id === ubicacion.id) limpiarFormularioUbicacion();
			if (seleccionado) await cargarUbicaciones(seleccionado);
		} catch (e) {
			error = mensajeError(e, 'No se pudo borrar la ubicación');
		}
	}

	async function buscar() {
		limpiarMensajes();
		buscando = true;
		try {
			const resultado = await buscarProductos({ q: busqueda, limite: 10 });
			resultados = resultado.items;
			if (resultados.length === 0) aviso = 'No se encontraron productos con ese término.';
		} catch (e) {
			error = mensajeError(e, 'No se pudo buscar el producto');
		} finally {
			buscando = false;
		}
	}

	async function elegirProducto(elegido: Producto) {
		limpiarMensajes();
		producto = elegido;
		resultados = [];
		cargandoAsignacion = true;
		try {
			const [asignadas, pares] = await Promise.all([
				ubicacionesDeProducto(elegido.id),
				Promise.all(
					depositos
						.filter((d) => d.activo)
						.map(async (deposito) => ({
							deposito,
							ubicaciones: (await listarUbicaciones(deposito.id)).filter((u) => u.activo)
						}))
				)
			]);
			seleccion = asignadas.map((u) => u.id);
			grupos = pares.filter((par) => par.ubicaciones.length > 0);
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar las ubicaciones del producto');
		} finally {
			cargandoAsignacion = false;
		}
	}

	function alternarSeleccion(id: number, marcado: boolean) {
		if (marcado) {
			if (!seleccion.includes(id)) seleccion = [...seleccion, id];
		} else {
			seleccion = seleccion.filter((valor) => valor !== id);
		}
	}

	async function guardarAsignacion() {
		if (!producto) return;
		limpiarMensajes();
		guardando = true;
		try {
			await reemplazarUbicacionesDeProducto(producto.id, seleccion);
			aviso = `Ubicaciones de ${producto.nombre} actualizadas.`;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron guardar las ubicaciones del producto');
		} finally {
			guardando = false;
		}
	}
</script>

<svelte:head>
	<title>Depósitos · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Depósitos y ubicaciones"
		descripcion="Jerarquía física del stock. El borrado físico solo se permite sin dependencias; la baja lógica conserva la historia."
	/>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}
	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}

	<div class="grid-principal">
		<Tarjeta titulo={editandoDeposito ? 'Editar depósito' : 'Nuevo depósito'}>
			<form
				class="grilla"
				onsubmit={(event) => {
					event.preventDefault();
					void guardarDeposito();
				}}
			>
				<div class="ancho">
					<Campo
						etiqueta="Nombre"
						nombre="nombre-deposito"
						maxlength={120}
						bind:valor={nombreDeposito}
						requerido={true}
					/>
				</div>
				<Campo
					etiqueta="Identificador"
					nombre="identificador"
					maxlength={40}
					bind:valor={identificador}
					requerido={true}
				/>
				<Campo
					etiqueta="Dirección"
					nombre="direccion"
					maxlength={500}
					bind:valor={direccion}
				/>
				<div class="acciones ancho">
					{#if editandoDeposito}
						<Boton
							variante="secundario"
							deshabilitado={guardando}
							onclick={limpiarFormularioDeposito}
						>
							Cancelar
						</Boton>
					{/if}
					<Boton
						tipo="submit"
						cargando={guardando}
						deshabilitado={!nombreDeposito.trim() || !identificador.trim()}
					>
						{guardando ? 'Guardando…' : editandoDeposito ? 'Guardar cambios' : 'Crear depósito'}
					</Boton>
				</div>
			</form>

			<div class="lista-depositos">
				<h3>Depósitos cargados</h3>
				{#if cargando}
					<Esqueleto alto="40px" />
				{:else if depositos.length === 0}
					<EstadoVacio
						icono="depositos"
						titulo="Todavía no hay depósitos"
						descripcion="Creá el primero con el formulario de arriba."
					/>
				{:else}
					<ul class="depositos">
						{#each depositos as deposito (deposito.id)}
							<li class="deposito" class:activo={seleccionado?.id === deposito.id}>
								<button
									type="button"
									class="deposito-info"
									onclick={() => void seleccionar(deposito)}
								>
									<span class="deposito-nombre">
										{deposito.nombre}
										<span class="mono">{deposito.identificador}</span>
									</span>
									{#if deposito.direccion}<small>{deposito.direccion}</small>{/if}
								</button>
								<EtiquetaEstado variante={deposito.activo ? 'exito' : 'neutro'}>
									{deposito.activo ? 'Activo' : 'Inactivo'}
								</EtiquetaEstado>
								<div class="acciones-fila">
									<Boton variante="fantasma" tamano="sm" onclick={() => editarDeposito(deposito)}>
										Editar
									</Boton>
									<Boton
										variante="fantasma"
										tamano="sm"
										onclick={() => void alternarEstadoDeposito(deposito)}
									>
										{deposito.activo ? 'Desactivar' : 'Reactivar'}
									</Boton>
									<Boton variante="fantasma" tamano="sm" onclick={() => void borrarDeposito(deposito)}>
										Borrar
									</Boton>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</Tarjeta>

		<Tarjeta titulo={`Ubicaciones${seleccionado ? ` de ${seleccionado.nombre}` : ''}`}>
			{#if !seleccionado}
				<EstadoVacio
					icono="depositos"
					titulo="Elegí un depósito"
					descripcion="Seleccioná un depósito de la lista para ver y editar su árbol de ubicaciones."
				/>
			{:else}
				<p class="ayuda">
					El tipo es un enum simple; la profundidad es libre y el padre debe pertenecer al mismo
					depósito.
				</p>
				<form
					class="grilla"
					onsubmit={(event) => {
						event.preventDefault();
						void guardarUbicacion();
					}}
				>
					<div class="ancho">
						<Campo
							etiqueta="Nombre"
							nombre="nombre-ubicacion"
							maxlength={80}
							bind:valor={nombreUbicacion}
							requerido={true}
						/>
					</div>
					<Campo
						etiqueta="Tipo"
						nombre="tipo-ubicacion"
						tipo="select"
						opciones={TIPOS}
						bind:valor={tipoUbicacion}
					/>
					<Campo
						etiqueta="Padre"
						nombre="padre-ubicacion"
						tipo="select"
						opciones={[
							{ valor: '', etiqueta: 'Sin padre (raíz)' },
							...opcionesPadre.map((nodo) => ({
								valor: String(nodo.ubicacion.id),
								etiqueta: `${'—'.repeat(nodo.profundidad)}${nodo.ubicacion.nombre}`
							}))
						]}
						bind:valor={padreUbicacion}
					/>
					<div class="acciones ancho">
						{#if editandoUbicacion}
							<Boton
								variante="secundario"
								deshabilitado={guardando}
								onclick={limpiarFormularioUbicacion}
							>
								Cancelar
							</Boton>
						{/if}
						<Boton tipo="submit" cargando={guardando} deshabilitado={!nombreUbicacion.trim()}>
							{guardando
								? 'Guardando…'
								: editandoUbicacion
									? 'Guardar cambios'
									: 'Agregar ubicación'}
						</Boton>
					</div>
				</form>

				{#if arbol.length === 0}
					<EstadoVacio
						icono="depositos"
						titulo="Sin ubicaciones"
						descripcion="Este depósito todavía no tiene ubicaciones."
					/>
				{:else}
					<ul class="arbol">
						{#each arbol as nodo (nodo.ubicacion.id)}
							<li class="nodo" style:padding-left={`${nodo.profundidad * 16}px`}>
								<div class="nodo-info">
									<span class="nodo-nombre">{nodo.ubicacion.nombre}</span>
									<EtiquetaEstado variante="info">{nodo.ubicacion.tipo}</EtiquetaEstado>
									{#if !nodo.ubicacion.activo}
										<EtiquetaEstado variante="neutro">Inactiva</EtiquetaEstado>
									{/if}
								</div>
								<div class="acciones-fila">
									<Boton variante="fantasma" tamano="sm" onclick={() => agregarHija(nodo.ubicacion)}>
										+ Hija
									</Boton>
									<Boton variante="fantasma" tamano="sm" onclick={() => editarUbicacion(nodo.ubicacion)}>
										Editar
									</Boton>
									<Boton
										variante="fantasma"
										tamano="sm"
										onclick={() => void alternarEstadoUbicacion(nodo.ubicacion)}
									>
										{nodo.ubicacion.activo ? 'Desactivar' : 'Reactivar'}
									</Boton>
									<Boton variante="fantasma" tamano="sm" onclick={() => void borrarUbicacion(nodo.ubicacion)}>
										Borrar
									</Boton>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			{/if}
		</Tarjeta>
	</div>

	<Tarjeta
		titulo="Ubicaciones permitidas por producto"
		descripcion="El producto solo podrá recibir stock en las ubicaciones marcadas. Se reemplaza el conjunto completo."
	>
		<form
			class="buscador"
			onsubmit={(event) => {
				event.preventDefault();
				void buscar();
			}}
		>
			<div class="campo-busqueda">
				<Campo
					etiqueta="Buscar producto"
					nombre="busqueda-producto"
					tipo="search"
					marcador="SKU, código o nombre"
					bind:valor={busqueda}
				/>
			</div>
			<Boton
				tipo="submit"
				variante="secundario"
				cargando={buscando}
				deshabilitado={busqueda.trim() === ''}
			>
				Buscar
			</Boton>
		</form>

		{#if resultados.length > 0}
			<ul class="resultados">
				{#each resultados as encontrado (encontrado.id)}
					<li>
						<span class="resultado-info">
							<strong>{encontrado.nombre}</strong>
							<span class="mono">{encontrado.sku}</span>
						</span>
						<Boton variante="secundario" tamano="sm" onclick={() => void elegirProducto(encontrado)}>
							Elegir
						</Boton>
					</li>
				{/each}
			</ul>
		{/if}

		{#if producto}
			<div class="producto-seleccionado">
				<span>
					Producto: <strong>{producto.nombre}</strong>
					<span class="mono">{producto.sku}</span>
				</span>
			</div>
			{#if cargandoAsignacion}
				<Esqueleto alto="80px" />
			{:else if grupos.length === 0}
				<EstadoVacio
					icono="depositos"
					titulo="Sin ubicaciones activas"
					descripcion="No hay ubicaciones activas en depósitos activos."
				/>
			{:else}
				<div class="grupos">
					{#each grupos as grupo (grupo.deposito.id)}
						<fieldset class="grupo">
							<legend>{grupo.deposito.nombre}</legend>
							{#each grupo.ubicaciones as ubicacion (ubicacion.id)}
								<label class="check">
									<input
										type="checkbox"
										checked={seleccion.includes(ubicacion.id)}
										onchange={(event) =>
											alternarSeleccion(ubicacion.id, event.currentTarget.checked)}
									/>
									<span>{ubicacion.nombre}</span>
									<span class="tipo">{ubicacion.tipo}</span>
								</label>
							{/each}
						</fieldset>
					{/each}
				</div>
				<div class="pie-asignacion">
					<span>
						{seleccion.length} ubicación{seleccion.length === 1 ? '' : 'es'} seleccionada{seleccion.length === 1 ? '' : 's'}
					</span>
					<Boton cargando={guardando} onclick={() => void guardarAsignacion()}>
						Guardar ubicaciones
					</Boton>
				</div>
			{/if}
		{/if}
	</Tarjeta>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.grid-principal {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 14px;
		align-items: start;
	}

	.grilla {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
	}

	.ancho {
		grid-column: 1 / -1;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 2px;
	}

	.lista-depositos {
		margin-top: 18px;
		padding-top: 14px;
		border-top: 1px solid var(--borde);
	}

	h3 {
		margin: 0 0 8px;
		color: var(--texto-suave);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.depositos,
	.arbol {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.deposito,
	.nodo {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 9px 8px;
		border-bottom: 1px solid var(--borde);
	}

	.deposito:last-child,
	.nodo:last-child {
		border-bottom: 0;
	}

	.deposito.activo {
		border-radius: var(--radio-control);
		background: var(--primario-suave);
	}

	.deposito-info {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
		border: 0;
		background: transparent;
		color: var(--texto);
		text-align: left;
		cursor: pointer;
		padding: 0;
	}

	.deposito-nombre {
		font-size: 13px;
		font-weight: 500;
	}

	.deposito-info small {
		color: var(--texto-tenue);
		font-size: 11.5px;
	}

	.mono {
		margin-left: 6px;
		color: var(--texto-tenue);
		font-family: var(--fuente-mono);
		font-size: 10.5px;
	}

	.acciones-fila {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: 2px;
	}

	.ayuda {
		margin: 0 0 10px;
		color: var(--texto-suave);
		font-size: 12.5px;
		line-height: 1.45;
	}

	.nodo {
		flex-wrap: wrap;
		align-items: center;
	}

	.nodo-info {
		display: flex;
		flex: 1;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.nodo-nombre {
		font-size: 13px;
		font-weight: 500;
	}

	.buscador {
		display: flex;
		align-items: flex-end;
		gap: 10px;
		flex-wrap: wrap;
		margin-bottom: 12px;
	}

	.campo-busqueda {
		flex: 1;
		min-width: 240px;
	}

	.resultados {
		display: flex;
		flex-direction: column;
		margin: 0 0 12px;
		padding: 0;
		list-style: none;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
	}

	.resultados li {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--borde);
	}

	.resultados li:last-child {
		border-bottom: 0;
	}

	.resultado-info {
		display: flex;
		align-items: baseline;
		gap: 6px;
		min-width: 0;
		font-size: 13px;
	}

	.producto-seleccionado {
		padding: 9px 12px;
		border-radius: var(--radio-control);
		background: var(--primario-suave);
		color: var(--primario-texto);
		font-size: 13px;
		margin-bottom: 12px;
	}

	.grupos {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
		gap: 12px;
	}

	.grupo {
		margin: 0;
		padding: 10px 12px;
		border: 1px solid var(--borde);
		border-radius: var(--radio-control);
	}

	.grupo legend {
		padding: 0 4px;
		color: var(--texto);
		font-size: 12.5px;
		font-weight: 600;
	}

	.check {
		display: flex;
		align-items: center;
		gap: 7px;
		padding: 3px 0;
		color: var(--texto);
		font-size: 12.5px;
	}

	.check input {
		accent-color: var(--primario);
	}

	.tipo {
		color: var(--texto-tenue);
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.pie-asignacion {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		flex-wrap: wrap;
		margin-top: 12px;
		color: var(--texto-suave);
		font-size: 12.5px;
	}

	@media (max-width: 1100px) {
		.grid-principal {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 560px) {
		.grilla {
			grid-template-columns: 1fr;
		}
	}
</style>