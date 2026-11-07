<script lang="ts">
	import { onMount } from 'svelte';
	import {
		activarCliente,
		actualizarCliente,
		crearCliente,
		desactivarCliente,
		listarClientes
	} from '#lib/api/clientes';
	import type { Cliente } from '#lib/schemas/clientes';
	import { mensajeError } from '#lib/utils/errores';
	import Aviso from '#lib/components/ui/Aviso.svelte';
	import Boton from '#lib/components/ui/Boton.svelte';
	import Campo from '#lib/components/ui/Campo.svelte';
	import EncabezadoPagina from '#lib/components/ui/EncabezadoPagina.svelte';
	import EstadoVacio from '#lib/components/ui/EstadoVacio.svelte';
	import EtiquetaEstado from '#lib/components/ui/EtiquetaEstado.svelte';
	import Paginacion from '#lib/components/ui/Paginacion.svelte';
	import TablaDatos from '#lib/components/ui/TablaDatos.svelte';
	import Tarjeta from '#lib/components/ui/Tarjeta.svelte';
	import type { Columna } from '#lib/components/ui/tabla';

	const COLUMNAS: Columna<Cliente>[] = [
		{ clave: 'nombre', encabezado: 'Nombre' },
		{ clave: 'documento', encabezado: 'Documento' },
		{ clave: 'email', encabezado: 'Email' },
		{ clave: 'telefono', encabezado: 'Teléfono' },
		{ clave: 'activo', encabezado: 'Estado' },
		{ clave: 'acciones', encabezado: '' }
	];
	const FILTROS = [
		{ valor: 'true', etiqueta: 'Activos' },
		{ valor: 'false', etiqueta: 'Inactivos' },
		{ valor: '', etiqueta: 'Todos' }
	];
	const porPagina = 20;

	let items = $state<Cliente[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	let filtro = $state('true');
	let cargando = $state(false);
	let error = $state<string | null>(null);
	let aviso = $state<string | null>(null);

	let editando = $state<Cliente | null>(null);
	let nombre = $state('');
	let documento = $state('');
	let email = $state('');
	let telefono = $state('');
	let direccion = $state('');
	let guardando = $state(false);

	async function cargar() {
		cargando = true;
		error = null;
		try {
			const res = await listarClientes({
				activo: filtro === '' ? undefined : filtro === 'true',
				pagina,
				porPagina
			});
			items = res.items;
			total = res.total;
		} catch (e) {
			error = mensajeError(e, 'No se pudieron cargar los clientes.');
		} finally {
			cargando = false;
		}
	}

	function nuevo() {
		editando = null;
		nombre = documento = email = telefono = direccion = '';
	}

	function editar(c: Cliente) {
		editando = c;
		nombre = c.nombre;
		documento = c.documento ?? '';
		email = c.email ?? '';
		telefono = c.telefono ?? '';
		direccion = c.direccion ?? '';
	}

	async function guardar() {
		guardando = true;
		error = null;
		aviso = null;
		const datos = { nombre, documento, email, telefono, direccion };
		try {
			const c = editando ? await actualizarCliente(editando.id, datos) : await crearCliente(datos);
			aviso = editando ? `Cliente «${c.nombre}» actualizado.` : `Cliente «${c.nombre}» creado.`;
			nuevo();
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo guardar el cliente.');
		} finally {
			guardando = false;
		}
	}

	async function cambiarEstado(c: Cliente) {
		error = null;
		try {
			const r = c.activo ? await desactivarCliente(c.id) : await activarCliente(c.id);
			aviso = r.activo ? `«${r.nombre}» reactivado.` : `«${r.nombre}» dado de baja; sus ventas se conservan.`;
			await cargar();
		} catch (e) {
			error = mensajeError(e, 'No se pudo cambiar el estado del cliente.');
		}
	}

	onMount(() => {
		void cargar();
	});
</script>

<svelte:head>
	<title>Clientes · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina titulo="Clientes" descripcion="Alta, edición y baja lógica de clientes de la caja." />

	{#if aviso}<Aviso variante="exito">{aviso}</Aviso>{/if}
	{#if error}<Aviso variante="error">{error}</Aviso>{/if}

	<Tarjeta>
		<h2 class="titulo">{editando ? `Editar «${editando.nombre}»` : 'Nuevo cliente'}</h2>
		<div class="formulario">
			<Campo etiqueta="Nombre" nombre="nombre" bind:valor={nombre} />
			<Campo etiqueta="Documento" nombre="documento" bind:valor={documento} />
			<Campo etiqueta="Email" nombre="email" bind:valor={email} />
			<Campo etiqueta="Teléfono" nombre="telefono" bind:valor={telefono} />
			<Campo etiqueta="Dirección" nombre="direccion" bind:valor={direccion} />
		</div>
		<div class="acciones">
			<Boton onclick={() => void guardar()} cargando={guardando}>{editando ? 'Guardar cambios' : 'Crear cliente'}</Boton>
			{#if editando}<Boton variante="fantasma" onclick={nuevo}>Cancelar</Boton>{/if}
		</div>
	</Tarjeta>

	<Tarjeta>
		<Campo
			etiqueta="Mostrar"
			nombre="filtro"
			tipo="select"
			opciones={FILTROS}
			bind:valor={filtro}
			onchange={() => {
				pagina = 1;
				void cargar();
			}}
		/>
	</Tarjeta>

	<Tarjeta densa={true}>
		<TablaDatos columnas={COLUMNAS} filas={items} etiqueta="Clientes" {cargando}>
			{#snippet fila(c)}
				<td>{c.nombre}</td>
				<td>{c.documento ?? '—'}</td>
				<td>{c.email ?? '—'}</td>
				<td>{c.telefono ?? '—'}</td>
				<td>
					<EtiquetaEstado variante={c.activo ? 'exito' : 'neutro'}>{c.activo ? 'Activo' : 'Inactivo'}</EtiquetaEstado>
				</td>
				<td class="acciones-fila">
					<Boton tamano="sm" variante="fantasma" onclick={() => editar(c)}>Editar</Boton>
					<Boton tamano="sm" variante={c.activo ? 'peligro' : 'secundario'} onclick={() => void cambiarEstado(c)}>
						{c.activo ? 'Dar de baja' : 'Reactivar'}
					</Boton>
				</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio icono="usuarios" titulo="Sin clientes" descripcion="Todavía no hay clientes con ese filtro." />
			{/snippet}
		</TablaDatos>
	</Tarjeta>

	<Paginacion {pagina} {total} {porPagina} onCambiar={(n) => { pagina = n; void cargar(); }} />
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.titulo {
		margin: 0 0 8px;
		font-size: 15px;
	}

	.formulario {
		display: grid;
		grid-template-columns: repeat(3, minmax(160px, 1fr));
		gap: 12px;
	}

	.acciones,
	.acciones-fila {
		display: flex;
		gap: 8px;
		margin-top: 10px;
	}

	.acciones-fila {
		margin-top: 0;
	}

	@media (max-width: 720px) {
		.formulario {
			grid-template-columns: 1fr;
		}
	}
</style>
