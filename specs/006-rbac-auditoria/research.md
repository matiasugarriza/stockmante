# Research — RBAC transversal y auditoría (006)

**Fecha**: 2026-10-26 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Decisiones

### 1. Número de migración

**Decisión**: `0008_auditoria.sql`.

**Por qué**: el Supuesto original reservaba `0006`, pero esa versión quedó ocupada por
`0006_catalogo_descripcion_codigo_barras.sql` (003) y `0007` por la búsqueda de 004. La
numeración es global y no se reutiliza; el texto de la spec se corrige para reflejar `0008`.

### 2. Dónde se escribe la auditoría

**Alternativas**:
- En el handler con una transacción aparte: simple, pero puede auditar un cambio que luego falla.
- En el servicio, dentro de la transacción del cambio: atómico.
- Con triggers de base: invisible para el código, pero difícil de ligar al actor de la sesión.

**Decisión**: servicio `auditoria::registrar(&mut tx, ...)` invocado antes del `commit`.

**Por qué**: garantiza "solo se auditan las acciones que se completan" y permite pasar el actor
validado desde `RequiereGestion`; un trigger no conoce el usuario de la sesión sin variables de
sesión adicionales.

### 3. Acciones cubiertas

**Decisión**: altas, ediciones, bajas lógicas, reactivaciones y reseteo de contraseña temporal de
usuarios; altas, ediciones (incluido precio), bajas y reactivaciones de productos.

**Por qué**: FR-019 pide "altas, bajas y cambios sensibles de usuarios y productos". Incluir el
alta y el reseteo completa el ciclo sin agregar un log rico.

### 4. Referencia al registro y al autor

**Decisión**: `registro_id` sin FK (sobrevive a limpiezas de datos) y `usuario_id` con FK
`ON DELETE SET NULL`.

**Por qué**: la auditoría debe conservar el hecho aunque el registro se borre; en producción los
usuarios solo se dan de baja lógica, por lo que el autor se conserva. `SET NULL` evita romper
tests y mantenimientos que borran usuarios de prueba.

### 5. Sin endpoint de consulta

**Decisión**: la tabla no se expone por API en esta spec.

**Por qué**: "log rico y consultas de auditoría" está explícitamente fuera de alcance; la
verificación se hace por SQL en tests y quickstart.

### 6. Guard de frontend

**Decisión**: `frontend/src/routes/admin/+layout.svelte` redirige a `/caja` cuando la familia no
es GESTION y no renderiza el contenido.

**Por qué**: el header ya ocultaba enlaces, pero el acceso directo por URL seguía mostrando
formularios que el backend rechazaba con 403. El guard es UX; la autoridad sigue en el backend.

### 7. Matriz RBAC

**Decisión**: un solo test parametrizado recorre métodos y rutas con IDs inexistentes y cuerpos
vacíos.

**Por qué**: así se prueba el control de acceso sin crear datos ni depender del estado; los
códigos 404/422 demuestran que GESTION pasó el extractor y OPERACION recibió 403 antes de
cualquier validación.

## Riesgos abiertos

- La tabla crecerá sin política de retención; se documenta como candidato a módulo posterior.
- `GET /api/categorias` y `/api/productos/buscar` son lecturas permitidas a OPERACION: no forman
  parte de los listados restringidos.
