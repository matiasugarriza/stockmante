# Data Model: Rediseño visual del escritorio

**Feature**: `018-rediseno-visual-desktop` | **Fecha**: 2026-11-07

Esta feature no agrega tablas ni migraciones. El "modelo" describe las entidades de presentación que
el plan y las tareas necesitan: páginas, navegación, tokens, componentes, widgets y evidencia.

## 1. Inventario de páginas

Cada captura de `captures/` mapea a una ruta existente. El modo de cambio obligatorio de contraseña
vive dentro de `/login` y se lista como fila propia porque tiene su propio estado y aceptación.

| # | Captura | Ruta | Título | Patrón | Navegación | Rol mínimo | Componentes principales | Endpoints leídos/escritos |
|---|---|---|---|---|---|---|---|---|
| 1 | `01-login.png` | `/login` | Iniciar sesión | Acceso | — | Público | `LienzoAcceso`, `Campo`, `Boton`, `BotonTema`, `Aviso` | `POST /api/auth/login`, `GET /api/auth/sso/config` |
| 2 | `02-registro.png` | `/registro` | Crear cuenta | Acceso | — | Público | `LienzoAcceso`, `Campo`, `Boton`, `Aviso` | `POST /api/auth/registro` |
| 3 | `03-verificar-correo.png` | `/verificar-correo` | Verificación de correo | Acceso | — | Público | `LienzoAcceso`, `Aviso`, `Boton` | `GET /api/auth/verificar-correo` |
| 4 | `04-inicio.png` | `/` | Inicio | Tablero | Operación | Sesión | `TarjetaKpi`, `GraficoBarras`, `GraficoDona`, `PanelActividad`, `AccesosRapidos` | `GET /api/stock`, `GET /api/movimientos`, `GET /api/productos`, `GET /api/transferencias`, `GET /api/reportes/stock-valorizado` |
| 5 | `05-caja.png` | `/caja` | Caja | Operativa | Operación | Sesión | `FiltrosCaja`, `TablaDatos`, `EtiquetaEstado`, `EstadoVacio` | `GET /api/productos/buscar`, `GET /api/categorias` |
| 6 | `06-movimientos.png` | `/movimientos` | Movimientos de stock | Operativa | Operación | Sesión | `FormularioMovimiento`, `TablaDatos`, `Aviso`, `EtiquetaEstado` | `GET/POST /api/movimientos`, `GET /api/depositos`, `GET /api/depositos/{id}/ubicaciones`, `GET /api/productos/buscar` |
| 7 | `07-transferencias.png` | `/transferencias` | Transferencias entre depósitos | Operativa | Operación | Sesión | `FormularioTransferencia`, `TablaDatos`, `Paginacion`, `Aviso` | `GET/POST /api/transferencias`, `GET /api/depositos`, `GET /api/productos/buscar` |
| 8 | `08-conteos.png` | `/conteos` | Conteos de inventario | Operativa | Operación | GESTION | `FormularioConteo`, `TablaDatos`, `Modal`, `EtiquetaEstado`, `Paginacion` | `GET/POST /api/conteos`, `PATCH /api/conteos/{id}/lineas`, `POST /api/conteos/{id}/cerrar|cancelar`, `GET /api/depositos` |
| 9 | `09-asistente.png` | `/asistente` | Asistente de inventario | Operativa | Operación | Sesión | `PanelConversacion`, `TarjetaPropuesta`, `Aviso`, `Esqueleto` | `POST /api/asistente/consulta` |
| 10 | `10-admin-usuarios.png` | `/admin/usuarios` | Usuarios | Administración | Administración | GESTION | `UsuarioTabla`, `UsuarioForm`, `PasswordTemporalModal`, `CredencialesModal` | `GET/POST /api/usuarios`, `PUT /api/usuarios/{id}`, `POST /api/usuarios/{id}/desactivar|reactivar|password-temporal` |
| 11 | `11-admin-categorias.png` | `/admin/categorias` | Categorías | Administración | Administración | GESTION | `CategoriaTabla`, `CategoriaForm`, `Modal` | `GET/POST /api/categorias`, `PUT /api/categorias/{id}`, `POST .../desactivar|reactivar` |
| 12 | `12-admin-productos.png` | `/admin/productos` | Productos | Administración | Administración | GESTION | `FiltrosCatalogo`, `ProductoTabla`, `ProductoForm` | `GET/POST/PUT /api/productos`, `GET /api/categorias`, `GET /api/productos/{id}/ubicaciones` |
| 13 | `13-admin-depositos.png` | `/admin/depositos` | Depósitos y ubicaciones | Administración | Administración | GESTION | `PanelDepositos`, `PanelUbicaciones`, `UbicacionesProducto`, `Modal` | `GET/POST/PUT /api/depositos`, `POST /api/ubicaciones`, `GET/PUT/DELETE /api/ubicaciones/{id}`, `GET/PUT /api/productos/{id}/ubicaciones` |
| 14 | `14-admin-stock.png` | `/admin/stock` | Stock | Administración | Administración | GESTION | `FiltrosStock`, `TablaDatos`, `EtiquetaEstado`, `Paginacion` | `GET /api/stock`, `GET /api/depositos` |
| 15 | `15-admin-proveedores.png` | `/admin/proveedores` | Proveedores | Administración | Administración | GESTION | `ProveedorTabla`, `ProveedorForm`, `EtiquetaEstado`, `Paginacion` | `GET/POST/PUT /api/proveedores`, `POST .../activar|desactivar` |
| 16 | `16-admin-compras.png` | `/admin/compras` | Compras | Administración | Administración | GESTION | `CompraTabla`, `CompraForm`, `EtiquetaEstado`, `Paginacion` | `GET/POST/PUT /api/compras`, `POST .../confirmar|anular`, `GET /api/proveedores`, `GET /api/depositos`, `GET /api/productos` |
| 17 | `17-admin-auditoria.png` | `/admin/auditoria` | Auditoría | Administración | Administración | GESTION | `FiltrosAuditoria`, `TablaDatos`, `Paginacion` | `GET /api/auditoria` |
| 18 | modo de `/login` | `/login` (`modo = cambio`) | Cambio de contraseña | Acceso | — | Sesión temporal | `LienzoAcceso`, `Campo`, `Aviso`, `Boton` | `POST /api/auth/cambiar-password`, `GET /api/auth/sesion` |

**Reglas**:

- La ruta de cada ítem de navegación DEBE existir en esta tabla.
- Toda ruta de `/admin/**` y `/conteos` DEBE exigir `GESTION`; el backend sigue siendo la autoridad.
- Las rutas de acceso NO usan `MarcoApp`; el resto SÍ.
- Ninguna captura puede quedar sin ruta ni ruta nueva sin captura de referencia.

## 2. Navegación

```ts
export type FamiliaRol = 'GESTION' | 'OPERACION';
export type GrupoNavegacion = 'operacion' | 'administracion';

export interface ItemNavegacion {
  etiqueta: string;
  ruta: string;
  icono: NombreIcono;
  grupo: GrupoNavegacion;
  roles: FamiliaRol[];
  contador?: 'alertas-stock';
}
```

| Grupo | Ítem | Ruta | Icono | Roles | Contador |
|---|---|---|---|---|---|
| Operación | Inicio | `/` | `inicio` | GESTION, OPERACION | — |
| Operación | Caja | `/caja` | `caja` | GESTION, OPERACION | — |
| Operación | Movimientos | `/movimientos` | `movimientos` | GESTION, OPERACION | — |
| Operación | Transferencias | `/transferencias` | `transferencias` | GESTION, OPERACION | — |
| Operación | Conteos | `/conteos` | `conteos` | GESTION | — |
| Operación | Asistente | `/asistente` | `asistente` | GESTION, OPERACION | — |
| Administración | Productos | `/admin/productos` | `productos` | GESTION | — |
| Administración | Categorías | `/admin/categorias` | `categorias` | GESTION | — |
| Administración | Depósitos | `/admin/depositos` | `depositos` | GESTION | — |
| Administración | Stock | `/admin/stock` | `stock` | GESTION | `alertas-stock` |
| Administración | Proveedores | `/admin/proveedores` | `proveedores` | GESTION | — |
| Administración | Compras | `/admin/compras` | `compras` | GESTION | — |
| Administración | Usuarios | `/admin/usuarios` | `usuarios` | GESTION | — |
| Administración | Auditoría | `/admin/auditoria` | `auditoria` | GESTION | — |

**Estado activo**: coincidencia exacta para `/`; prefijo para el resto (`/admin/productos` activa
Productos; `/caja` activa Caja). El ítem activo expone `aria-current="page"`.

## 3. Tokens visuales

Contrato completo en [contracts/tokens.md](./contracts/tokens.md). Nombres semánticos:

| Token | Uso | Claro | Oscuro |
|---|---|---|---|
| `--fondo` | Fondo del área de contenido | `#f1f4f9` | `#121722` |
| `--fondo-elevado` | Fondo de secciones internas | `#f7f9fc` | `#161c28` |
| `--superficie` | Tarjetas, tablas, barra lateral | `#ffffff` | `#1b2230` |
| `--superficie-2` | Inputs, hover suave | `#f5f7fb` | `#222b3b` |
| `--superficie-3` | Encabezado de tabla, activo suave | `#eaeff7` | `#2a3447` |
| `--borde` | Bordes de tarjeta y separadores | `#e3e8f0` | `#2b3648` |
| `--borde-fuerte` | Bordes de control | `#c9d3e0` | `#3b4a63` |
| `--texto` | Texto principal | `#1b2432` | `#e8edf6` |
| `--texto-suave` | Subtítulos y ayudas | `#5f6f86` | `#9aa9bf` |
| `--texto-tenue` | Metadatos y placeholders | `#93a1b5` | `#6c7c93` |
| `--primario` | Acción principal, activo | `#2f6bed` | `#4c8dff` |
| `--primario-suave` | Fondo de activo/selección | `#e9f0ff` | `#1c2b48` |
| `--teal` | Acento secundario, éxito informativo | `#14b8bd` | `#2fd0d6` |
| `--ambar` | Advertencia, bajo mínimo | `#f0a63a` | `#f3b45c` |
| `--exito` | Estado positivo | `#22a45d` | `#3ecf7a` |
| `--peligro` | Error y acción destructiva | `#dc3d43` | `#ff6b6e` |

## 4. Catálogo de componentes

| Componente | Responsabilidad | Variantes / estados | Consumidores |
|---|---|---|---|
| `Boton` | Acción | primario, secundario, fantasma, peligro; cargando, deshabilitado | Todas |
| `Campo` | Label + input/select/textarea + ayuda + error | texto, email, password, número, select, textarea | Acceso, operación, administración |
| `Tarjeta` | Superficie con encabezado y acciones | simple, con encabezado, con pie | Todas |
| `EtiquetaEstado` | Badge de estado | éxito, aviso, peligro, info, neutro | Movimientos, conteos, compras, stock, usuarios |
| `TablaDatos` | Shell de tabla | cargando, vacío, con filas, compacta | Caja, movimientos, administración |
| `Paginacion` | Navegación de páginas | deshabilitada, con total | Administración, caja, movimientos |
| `Modal` | Diálogo | abierto/cerrado, con foco atrapado | Formularios y confirmaciones |
| `Aviso` | Mensaje inline | info, éxito, error, advertencia | Todas |
| `EstadoVacio` | Sin resultados | con icono y acción | Tablas y tablero |
| `Esqueleto` | Carga | línea, bloque, tarjeta | Tablero y tablas |
| `Icono` | Iconografía SVG | 18/20 px, 18 nombres | Marco y páginas |
| `TarjetaKpi` | Indicador | etiqueta, valor, delta/contexto | Inicio |
| `GraficoBarras` | Serie por categoría | vacío, con datos, con leyenda | Inicio |
| `GraficoDona` | Proporción | vacío, con datos, con leyenda | Inicio |
| `BotonTema` | Selector Light/Dark/System | tres estados | Marco y acceso |
| `MarcoApp` | Shell privado | expandido, colapsado, off-canvas | Rutas privadas |
| `BarraLateral` | Navegación | expandida, colapsada, superpuesta | Marco |
| `BarraSuperior` | Contexto y usuario | título, acciones, menú | Marco |
| `MenuUsuario` | Sesión | abierto/cerrado | Marco |
| `LienzoAcceso` | Fondo de acceso | login, registro, verificación | Rutas públicas |

## 5. Widgets del tablero

| Widget | Fuente | Cálculo | Rol | Vacío | Error |
|---|---|---|---|---|---|
| Stock bajo mínimo | `GET /api/stock?bajo_minimo=true&por_pagina=1` | `total` | Todos | `0` con texto "Sin alertas" | Aviso con reintento |
| Movimientos de hoy | `GET /api/movimientos?desde=<hoy>&por_pagina=1` | `total` | Todos | `0` | Aviso con reintento |
| Productos activos | `GET /api/productos?activo=true&por_pagina=1` | `total` | GESTION | Oculto | Oculto |
| Valor del stock | `GET /api/reportes/stock-valorizado` | `total` | GESTION | Oculto | Aviso con reintento |
| Movimientos por tipo | `GET /api/movimientos?tipo=X&por_pagina=1` (4 llamadas) | `total` por tipo | Todos | Estado vacío | Aviso |
| Estado del stock | `GET /api/stock?por_pagina=1` + `bajo_minimo=true` | bajo mínimo / total | Todos | Estado vacío | Aviso |
| Actividad reciente | `GET /api/movimientos?por_pagina=6` | últimos 6 | Todos | Estado vacío | Aviso |
| Casos urgentes | `GET /api/stock?bajo_minimo=true&por_pagina=5` | primeros 5 | Todos | Estado vacío | Aviso |
| Accesos rápidos | sin endpoint | enlaces por rol | Todos | — | — |

**Regla de independencia**: cada widget resuelve su promesa por separado; el fallo de uno no bloquea
el tablero y no se muestra un error crudo de 403 (se oculta el widget).

## 6. Evidencia visual

| Artefacto | Ruta | Regla |
|---|---|---|
| Captura clara | `specs/018-rediseno-visual-desktop/evidencia/claro/<captura>.png` | 1440×900, tema claro, `data-tema="light"` |
| Captura oscura | `specs/018-rediseno-visual-desktop/evidencia/oscuro/<captura>.png` | 1440×900, tema oscuro, `data-tema="dark"` |
| Chequeo angosto | `specs/018-rediseno-visual-desktop/evidencia/angosto/<ruta>.png` | 320 px de ancho, sin overflow del `body` |

El nombre de archivo conserva el de la captura original (`04-inicio.png`, etc.) para que la
comparación sea directa.

## 7. Estados y transiciones

### Tema

```text
(sin preferencia) --carga--> system
system --prefers-color-scheme--> light | dark
light | dark | system --selección del usuario--> preferencia persistida
preferencia --recarga--> aplicar antes del primer pintado
almacenamiento bloqueado --> light en memoria (sin excepción)
```

### Marco

```text
expandido --viewport < 1280--> colapsado
colapsado --viewport < 1024--> superpuesto (cerrado)
superpuesto --botón menú--> superpuesto (abierto) --Escape/backdrop--> cerrado
```

### Página / widget

```text
inactivo --montaje--> cargando --> listo
                              ├─ sin datos --> vacío
                              └─ error --> error (reintento)
```

## 8. Reglas de validación del modelo

1. **Cobertura**: 17 capturas + 1 modo = 18 filas; toda fila tiene ruta, patrón, rol y evidencia
   esperada.
2. **Navegación**: todo ítem apunta a una ruta existente; los ítems de administración y Conteos solo
   se muestran a `GESTION`.
3. **Tokens**: ningún archivo fuera de `lib/diseno/tokens.css` declara colores hexadecimales de
   marca; las páginas usan tokens o utilidades derivadas.
4. **Contraste**: texto normal ≥ 4.5:1 y texto grande ≥ 3:1 en ambos temas; estados no dependen solo
   del color (llevan texto o icono).
5. **Responsive**: ninguna ruta genera `scrollWidth > clientWidth` en `body` a 320, 768, 1024 y
   1440 px.
6. **Evidencia**: 18 capturas claras + 18 oscuras + al menos 8 chequeos angostos, todas con el mismo
   viewport y sin datos sensibles.
