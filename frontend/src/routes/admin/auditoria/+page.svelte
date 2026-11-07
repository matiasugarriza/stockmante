<script lang="ts">
	import { onMount } from 'svelte';
	import { ApiError } from '#lib/api/client';
	import { api } from '#lib/api/client';
	import { z } from 'zod';
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

	const ItemSchema = z.object({
		id: z.number().int(),
		tabla_ref: z.string(),
		registro_id: z.number().int(),
		accion: z.string(),
		usuario_id: z.number().int(),
		usuario_nombre: z.string(),
		cuando: z.string()
	});
	const ListaSchema = z.object({
		items: z.array(ItemSchema),
		total: z.number().int(),
		pagina: z.number().int(),
		por_pagina: z.number().int()
	});
	type ItemAuditoria = z.infer<typeof ItemSchema>;

	const COLUMNAS: Columna<ItemAuditoria>[] = [
		{ clave: 'id', encabezado: 'ID' },
		{ clave: 'tabla_ref', encabezado: 'Tabla' },
		{ clave: 'registro_id', encabezado: 'Registro', alineacion: 'derecha' },
		{ clave: 'accion', encabezado: 'Acción' },
		{ clave: 'usuario_nombre', encabezado: 'Usuario' },
		{ clave: 'cuando', encabezado: 'Cuándo' }
	];

	const formatoFecha = new Intl.DateTimeFormat('es-AR', {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	let items = $state<ItemAuditoria[]>([]);
	let total = $state(0);
	let pagina = $state(1);
	const porPagina = 20;
	let tablaRef = $state('');
	let usuarioId = $state('');
	let cargando = $state(false);
	let error = $state('');

	function varianteAccion(accion: string): 'exito' | 'info' | 'aviso' | 'peligro' | 'neutro' {
		const valor = accion.toUpperCase();
		if (valor.includes('CREAR') || valor.includes('ALTA')) return 'exito';
		if (valor.includes('ACTUALIZAR') || valor.includes('EDITAR')) return 'info';
		if (valor.includes('ELIMINAR') || valor.includes('BAJA') || valor.includes('DESACTIVAR'))
			return 'peligro';
		return 'neutro';
	}

	async function cargar() {
		cargando = true;
		error = '';
		try {
			const query = new URLSearchParams();
			if (tablaRef) query.set('tabla_ref', tablaRef);
			if (usuarioId) query.set('usuario_id', usuarioId);
			query.set('pagina', String(pagina));
			query.set('por_pagina', String(porPagina));
			const body = await api<unknown>(`/api/auditoria?${query.toString()}`);
			const res = ListaSchema.parse(body);
			items = res.items;
			total = res.total;
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'No se pudo cargar la auditoría';
		} finally {
			cargando = false;
		}
	}

	function aplicarFiltros() {
		pagina = 1;
		void cargar();
	}

	function cambiarPagina(nueva: number) {
		pagina = nueva;
		void cargar();
	}

	onMount(cargar);
</script>

<svelte:head>
	<title>Auditoría · Stockmante</title>
</svelte:head>

<div class="pagina">
	<EncabezadoPagina
		titulo="Auditoría"
		descripcion="Consultá las escrituras de gestión registradas, filtradas por tabla o usuario."
	/>

	{#if error}<Aviso variante="error">{error}</Aviso>{/if}

	<Tarjeta>
		<div class="filtros">
			<Campo etiqueta="Tabla" nombre="tabla_ref" marcador="Ej. productos" bind:valor={tablaRef} />
			<Campo
				etiqueta="Usuario ID"
				nombre="usuario_id"
				tipo="number"
				min={1}
				bind:valor={usuarioId}
			/>
			<div class="accion-filtro">
				<Boton onclick={aplicarFiltros}>Filtrar</Boton>
			</div>
		</div>
	</Tarjeta>

	<Tarjeta densa={true}>
		<TablaDatos
			columnas={COLUMNAS}
			filas={items}
			etiqueta="Registros de auditoría"
			cargando={cargando}
		>
			{#snippet fila(item)}
				<td>{item.id}</td>
				<td class="mono">{item.tabla_ref}</td>
				<td class="numero">{item.registro_id}</td>
				<td>
					<EtiquetaEstado variante={varianteAccion(item.accion)}>{item.accion}</EtiquetaEstado>
				</td>
				<td>{item.usuario_nombre}</td>
				<td class="fecha">{formatoFecha.format(new Date(item.cuando))}</td>
			{/snippet}
			{#snippet vacio()}
				<EstadoVacio
					icono="auditoria"
					titulo="Sin registros de auditoría"
					descripcion="Probá con otra tabla o quitá el filtro de usuario."
				/>
			{/snippet}
		</TablaDatos>
	</Tarjeta>

	<div class="pie">
		<Paginacion {pagina} {total} {porPagina} onCambiar={cambiarPagina} />
	</div>
</div>

<style>
	.pagina {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filtros {
		display: flex;
		align-items: flex-end;
		gap: 12px;
		flex-wrap: wrap;
	}

	.filtros :global(.campo) {
		min-width: 200px;
	}

	.accion-filtro {
		padding-bottom: 1px;
	}

	.mono {
		color: var(--texto-suave);
		font-family: var(--fuente-mono);
		font-size: 11.5px;
	}

	.numero {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	.fecha {
		white-space: nowrap;
		color: var(--texto-suave);
		font-size: 12px;
	}

	.pie {
		padding: 0 2px;
	}

	@media (max-width: 640px) {
		.filtros {
			flex-direction: column;
			align-items: stretch;
		}

		.filtros :global(.campo) {
			min-width: 100%;
		}
	}
</style>
