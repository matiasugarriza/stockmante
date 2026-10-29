<script lang="ts">
	import { goto } from '$app/navigation';
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
		if ($sesion.estado === 'activa' && $sesion.usuario.familia_rol !== 'GESTION') {
			void goto('/');
		}
	});

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

<div class="flex flex-col gap-4">
	<div>
		<h1 class="text-xl font-semibold">Depósitos y ubicaciones</h1>
		<p class="text-sm text-slate-500">
			Jerarquía física del stock. El borrado físico solo se permite sin dependencias; la baja
			lógica conserva la historia.
		</p>
	</div>

	{#if error}
		<p class="rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>
	{/if}
	{#if aviso}
		<p class="rounded border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
			{aviso}
		</p>
	{/if}

	<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
		<section class="rounded border border-slate-200 bg-white p-4">
			<h2 class="mb-3 text-base font-semibold">
				{editandoDeposito ? 'Editar depósito' : 'Nuevo depósito'}
			</h2>
			<form
				class="grid grid-cols-1 gap-3 sm:grid-cols-2"
				onsubmit={(e) => {
					e.preventDefault();
					void guardarDeposito();
				}}
			>
				<label class="flex flex-col gap-1 text-sm sm:col-span-2">
					<span class="font-medium">Nombre</span>
					<input
						class="rounded border border-slate-300 px-3 py-2"
						bind:value={nombreDeposito}
						maxlength="120"
						required
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Identificador</span>
					<input
						class="rounded border border-slate-300 px-3 py-2 uppercase"
						bind:value={identificador}
						maxlength="40"
						required
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="font-medium">Dirección (opcional)</span>
					<input
						class="rounded border border-slate-300 px-3 py-2"
						bind:value={direccion}
						maxlength="500"
					/>
				</label>
				<div class="flex justify-end gap-2 sm:col-span-2">
					{#if editandoDeposito}
						<button
							type="button"
							class="rounded border border-slate-300 px-3 py-2 text-sm hover:bg-slate-100"
							onclick={limpiarFormularioDeposito}
							disabled={guardando}
						>
							Cancelar
						</button>
					{/if}
					<button
						type="submit"
						class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-60"
						disabled={guardando || !nombreDeposito.trim() || !identificador.trim()}
					>
						{guardando ? 'Guardando…' : editandoDeposito ? 'Guardar cambios' : 'Crear depósito'}
					</button>
				</div>
			</form>

			<div class="mt-4 border-t border-slate-100 pt-3">
				<h3 class="mb-2 text-sm font-semibold text-slate-600">Depósitos cargados</h3>
				{#if cargando}
					<p class="text-sm text-slate-500">Cargando…</p>
				{:else if depositos.length === 0}
					<p class="text-sm text-slate-500">Todavía no hay depósitos.</p>
				{:else}
					<ul class="divide-y divide-slate-100">
						{#each depositos as deposito (deposito.id)}
							<li
								class="flex flex-wrap items-center gap-2 py-2 text-sm"
								class:bg-slate-50={seleccionado?.id === deposito.id}
							>
								<button
									type="button"
									class="min-w-0 flex-1 text-left"
									onclick={() => void seleccionar(deposito)}
								>
									<span class="font-medium">{deposito.nombre}</span>
									<span class="ml-2 font-mono text-xs text-slate-500">
										{deposito.identificador}
									</span>
									{#if deposito.direccion}
										<span class="block text-xs text-slate-500">{deposito.direccion}</span>
									{/if}
								</button>
								<span
									class="rounded px-1.5 py-0.5 text-xs {deposito.activo
										? 'bg-emerald-50 text-emerald-700'
										: 'bg-slate-100 text-slate-500'}"
								>
									{deposito.activo ? 'Activo' : 'Inactivo'}
								</span>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-slate-600 hover:bg-slate-100"
									onclick={() => editarDeposito(deposito)}
								>
									Editar
								</button>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-slate-600 hover:bg-slate-100"
									onclick={() => void alternarEstadoDeposito(deposito)}
								>
									{deposito.activo ? 'Desactivar' : 'Reactivar'}
								</button>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-red-600 hover:bg-red-50"
									onclick={() => void borrarDeposito(deposito)}
								>
									Borrar
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</section>

		<section class="rounded border border-slate-200 bg-white p-4">
			<h2 class="mb-1 text-base font-semibold">
				Ubicaciones{seleccionado ? ` de ${seleccionado.nombre}` : ''}
			</h2>
			{#if !seleccionado}
				<p class="text-sm text-slate-500">Elegí un depósito de la lista para ver su árbol.</p>
			{:else}
				<p class="mb-3 text-xs text-slate-500">
					El tipo es un enum simple; la profundidad es libre y el padre debe pertenecer al mismo
					depósito.
				</p>
				<form
					class="grid grid-cols-1 gap-3 sm:grid-cols-2"
					onsubmit={(e) => {
						e.preventDefault();
						void guardarUbicacion();
					}}
				>
					<label class="flex flex-col gap-1 text-sm sm:col-span-2">
						<span class="font-medium">Nombre</span>
						<input
							class="rounded border border-slate-300 px-3 py-2"
							bind:value={nombreUbicacion}
							maxlength="80"
							required
						/>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						<span class="font-medium">Tipo</span>
						<select class="rounded border border-slate-300 px-3 py-2" bind:value={tipoUbicacion}>
							{#each TIPOS as tipo (tipo.valor)}
								<option value={tipo.valor}>{tipo.etiqueta}</option>
							{/each}
						</select>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						<span class="font-medium">Padre</span>
						<select class="rounded border border-slate-300 px-3 py-2" bind:value={padreUbicacion}>
							<option value="">Sin padre (raíz)</option>
							{#each opcionesPadre as nodo (nodo.ubicacion.id)}
								<option value={String(nodo.ubicacion.id)}>
									{'—'.repeat(nodo.profundidad)}{nodo.ubicacion.nombre}
								</option>
							{/each}
						</select>
					</label>
					<div class="flex justify-end gap-2 sm:col-span-2">
						{#if editandoUbicacion}
							<button
								type="button"
								class="rounded border border-slate-300 px-3 py-2 text-sm hover:bg-slate-100"
								onclick={limpiarFormularioUbicacion}
								disabled={guardando}
							>
								Cancelar
							</button>
						{/if}
						<button
							type="submit"
							class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-60"
							disabled={guardando || !nombreUbicacion.trim()}
						>
							{guardando
								? 'Guardando…'
								: editandoUbicacion
									? 'Guardar cambios'
									: 'Agregar ubicación'}
						</button>
					</div>
				</form>

				{#if arbol.length === 0}
					<p class="mt-3 rounded border border-slate-200 bg-slate-50 px-3 py-4 text-sm text-slate-500">
						Este depósito todavía no tiene ubicaciones.
					</p>
				{:else}
					<ul class="mt-3 divide-y divide-slate-100">
						{#each arbol as nodo (nodo.ubicacion.id)}
							<li class="flex flex-wrap items-center gap-2 py-2 text-sm">
								<span
									class="min-w-0 flex-1"
									style="padding-left: {nodo.profundidad * 16}px"
								>
									<span class="font-medium">{nodo.ubicacion.nombre}</span>
									<span class="ml-2 rounded bg-slate-100 px-1.5 py-0.5 text-xs text-slate-600">
										{nodo.ubicacion.tipo}
									</span>
									{#if !nodo.ubicacion.activo}
										<span class="ml-2 text-xs text-slate-400">Inactiva</span>
									{/if}
								</span>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-slate-600 hover:bg-slate-100"
									onclick={() => agregarHija(nodo.ubicacion)}
								>
									+ Hija
								</button>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-slate-600 hover:bg-slate-100"
									onclick={() => editarUbicacion(nodo.ubicacion)}
								>
									Editar
								</button>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-slate-600 hover:bg-slate-100"
									onclick={() => void alternarEstadoUbicacion(nodo.ubicacion)}
								>
									{nodo.ubicacion.activo ? 'Desactivar' : 'Reactivar'}
								</button>
								<button
									type="button"
									class="rounded px-2 py-1 text-xs text-red-600 hover:bg-red-50"
									onclick={() => void borrarUbicacion(nodo.ubicacion)}
								>
									Borrar
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			{/if}
		</section>
	</div>

	<section class="rounded border border-slate-200 bg-white p-4">
		<h2 class="text-base font-semibold">Ubicaciones permitidas por producto</h2>
		<p class="mb-3 text-sm text-slate-500">
			El producto solo podrá recibir stock en las ubicaciones marcadas. Se reemplaza el conjunto
			completo.
		</p>

		<form
			class="flex flex-wrap items-end gap-2"
			onsubmit={(e) => {
				e.preventDefault();
				void buscar();
			}}
		>
			<label class="flex min-w-64 flex-1 flex-col gap-1 text-sm">
				<span class="font-medium">Buscar producto</span>
				<input
					class="rounded border border-slate-300 px-3 py-2"
					bind:value={busqueda}
					placeholder="SKU, código o nombre"
				/>
			</label>
			<button
				type="submit"
				class="rounded border border-slate-300 px-3 py-2 text-sm hover:bg-slate-100 disabled:opacity-60"
				disabled={buscando || busqueda.trim() === ''}
			>
				{buscando ? 'Buscando…' : 'Buscar'}
			</button>
		</form>

		{#if resultados.length > 0}
			<ul class="mt-3 divide-y divide-slate-100 rounded border border-slate-200">
				{#each resultados as encontrado (encontrado.id)}
					<li class="flex items-center gap-2 px-3 py-2 text-sm">
						<span class="min-w-0 flex-1">
							<span class="font-medium">{encontrado.nombre}</span>
							<span class="ml-2 font-mono text-xs text-slate-500">{encontrado.sku}</span>
						</span>
						<button
							type="button"
							class="rounded bg-slate-800 px-3 py-1 text-xs font-medium text-white hover:bg-slate-700"
							onclick={() => void elegirProducto(encontrado)}
						>
							Elegir
						</button>
					</li>
				{/each}
			</ul>
		{/if}

		{#if producto}
			<div class="mt-4">
				<p class="text-sm">
					Producto: <span class="font-medium">{producto.nombre}</span>
					<span class="ml-2 font-mono text-xs text-slate-500">{producto.sku}</span>
				</p>
				{#if cargandoAsignacion}
					<p class="mt-2 text-sm text-slate-500">Cargando ubicaciones…</p>
				{:else if grupos.length === 0}
					<p class="mt-2 text-sm text-slate-500">
						No hay ubicaciones activas en depósitos activos.
					</p>
				{:else}
					<div class="mt-2 grid grid-cols-1 gap-3 md:grid-cols-2">
						{#each grupos as grupo (grupo.deposito.id)}
							<fieldset class="rounded border border-slate-200 p-3">
								<legend class="px-1 text-sm font-medium">{grupo.deposito.nombre}</legend>
								{#each grupo.ubicaciones as ubicacion (ubicacion.id)}
									<label class="flex items-center gap-2 py-0.5 text-sm">
										<input
											type="checkbox"
											class="h-4 w-4"
											checked={seleccion.includes(ubicacion.id)}
											onchange={(e) =>
												alternarSeleccion(
													ubicacion.id,
													(e.currentTarget as HTMLInputElement).checked
												)}
										/>
										{ubicacion.nombre}
										<span class="text-xs text-slate-400">{ubicacion.tipo}</span>
									</label>
								{/each}
							</fieldset>
						{/each}
					</div>
					<div class="mt-3 flex items-center justify-between gap-2">
						<p class="text-xs text-slate-500">
							{seleccion.length} ubicación{seleccion.length === 1 ? '' : 'es'} seleccionada{seleccion.length === 1 ? '' : 's'}
						</p>
						<button
							type="button"
							class="rounded bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-60"
							onclick={() => void guardarAsignacion()}
							disabled={guardando}
						>
							{guardando ? 'Guardando…' : 'Guardar ubicaciones'}
						</button>
					</div>
				{/if}
			</div>
		{/if}
	</section>
</div>
