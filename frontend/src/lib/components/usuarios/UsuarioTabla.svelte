<script lang="ts">
	import type { Usuario } from '#lib/schemas/usuarios';
	import Boton from '../ui/Boton.svelte';
	import EstadoVacio from '../ui/EstadoVacio.svelte';
	import EtiquetaEstado from '../ui/EtiquetaEstado.svelte';
	import TablaDatos from '../ui/TablaDatos.svelte';
	import type { Columna } from '../ui/tabla';

	interface Props {
		usuarios: Usuario[];
		cargando?: boolean;
		onEditar: (usuario: Usuario) => void;
		onAlternarEstado: (usuario: Usuario) => void;
		onRegenerar: (usuario: Usuario) => void;
	}

	let { usuarios, cargando = false, onEditar, onAlternarEstado, onRegenerar }: Props = $props();

	const COLUMNAS: Columna<Usuario>[] = [
		{ clave: 'nombre', encabezado: 'Nombre' },
		{ clave: 'email', encabezado: 'Email' },
		{ clave: 'rol', encabezado: 'Rol' },
		{ clave: 'activo', encabezado: 'Estado' },
		{ clave: 'acciones', encabezado: 'Acciones', alineacion: 'derecha' }
	];
</script>

<TablaDatos columnas={COLUMNAS} filas={usuarios} etiqueta="Usuarios" {cargando}>
	{#snippet fila(usuario)}
		<td class="principal">{usuario.nombre}</td>
		<td class="email">{usuario.email}</td>
		<td>
			<EtiquetaEstado variante="info">{usuario.rol}</EtiquetaEstado>
		</td>
		<td>
			<div class="estado">
				<EtiquetaEstado variante={usuario.activo ? 'exito' : 'neutro'}>
					{usuario.activo ? 'Activo' : 'Inactivo'}
				</EtiquetaEstado>
				{#if usuario.password_temporal}
					<EtiquetaEstado variante="aviso">temporal</EtiquetaEstado>
				{/if}
			</div>
		</td>
		<td>
			<div class="acciones">
				<Boton variante="secundario" tamano="sm" onclick={() => onEditar(usuario)}>
					Editar
				</Boton>
				<Boton variante="fantasma" tamano="sm" onclick={() => onAlternarEstado(usuario)}>
					{usuario.activo ? 'Desactivar' : 'Reactivar'}
				</Boton>
				<Boton variante="fantasma" tamano="sm" onclick={() => onRegenerar(usuario)}>
					Nueva temporal
				</Boton>
			</div>
		</td>
	{/snippet}
	{#snippet vacio()}
		<EstadoVacio
			icono="usuarios"
			titulo="No hay usuarios para este filtro"
			descripcion="Cambiá el filtro o creá el primer usuario."
		/>
	{/snippet}
</TablaDatos>

<style>
	.principal {
		font-weight: 500;
	}

	.email {
		color: var(--texto-suave);
	}

	.estado {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.acciones {
		display: flex;
		justify-content: flex-end;
		gap: 4px;
		flex-wrap: wrap;
	}
</style>
