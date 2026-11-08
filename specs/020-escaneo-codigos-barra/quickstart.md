# Quickstart: validar el escaneo y la generación de códigos (020)

Guía de validación de punta a punta. Contratos: [contracts/api.md](./contracts/api.md) y
[contracts/ui.md](./contracts/ui.md). Modelo: [data-model.md](./data-model.md).

## Requisitos previos

- PostgreSQL con las migraciones aplicadas y un usuario de gestión y uno de operación (seed).
- Backend corriendo en `http://localhost:3000` (`cargo run` en `backend/`).
- Un celular con cámara conectado **a la misma red Wi-Fi** que el PC.
- Un producto activo con código `7791234567898`, uno sin código, uno con código interno
  alfanumérico (p. ej. `INT00001`) y uno inactivo.

## 1. Gates automáticos

```bash
cd backend && cargo test && cargo sqlx prepare --check
```

```bash
cd frontend && npm test && npm run check && npm run build
```

Esperado: todo en verde, incluidos `test_ean13`, `test_codigo_barras_http`, `ean13.test.ts`,
`escaner.test.ts`, `etiqueta.test.ts` y los tests de `resolverLectura` / `asignarCodigoBarras`.

## 2. Abrir la app en el celular por HTTPS

```bash
cd frontend && npm run dev:movil
```

1. Vite muestra `Network: https://<IP-del-PC>:5173/`. Si Windows pregunta, permitir Node.js en
   redes privadas.
2. En el celular abrir esa URL, aceptar el aviso del certificado autofirmado
   («Avanzado → Continuar»).
3. Iniciar sesión. Esperado: la sesión funciona (la API va por el mismo origen vía `/api`).

## 3. Escanear con la cámara (US1)

1. En **Caja**, tocar «Escanear con cámara» y aceptar el permiso.
2. Apuntar al código `7791234567898` (impreso o en otra pantalla). Esperado: en < 2 s se muestra
   el producto con nombre, SKU y stock del depósito elegido; la cámara se apaga.
3. Escanear un código desconocido. Esperado: «No se encontró un producto con el código …» y el
   valor queda en el buscador.
4. Escanear el producto inactivo. Esperado: aviso «dado de baja», no se agrega.
5. Repetir el paso 2 en Movimientos, Transferencias, Conteos (resalta la línea) y Catálogo (abre
   la ficha). Esperado: el botón existe en las 5 pantallas (SC-061).
6. Verificar en la BD que no hay movimientos nuevos (SC-054).

## 4. Errores de cámara (FR-151)

1. Abrir la app por `http://<IP-del-PC>:5173` (modo `npm run dev -- --host`). Esperado: al tocar
   «Escanear» aparece el aviso de página no segura con las alternativas.
2. Denegar el permiso de cámara. Esperado: aviso de permiso con «Reintentar» y búsqueda manual.
3. `Escape` o «Cancelar» cierran el escáner y el foco vuelve al buscador.

## 5. Lector físico o entrada manual (US2)

1. En el PC, con el foco en el buscador de Caja, escribir ` 7791234567898 ` y Enter (simula la
   pistola). Esperado: mismo resultado que con la cámara.
2. Buscar el SKU en minúsculas. Esperado: encuentra el producto.

## 6. Generar y asignar un EAN-13 (US3, usuario de gestión)

1. En Catálogo, abrir el producto sin código → «Asignar código» → «Automático». Esperado: vista
   previa `20…` y, al asignar, el mensaje con el código, el nombre y el SKU.
2. Intentar `7791234567890` (verificador incorrecto) en «Desde 12 dígitos»/ficha. Esperado: 422
   con el dígito correcto (8).
3. Asignar a otro producto un código ya usado. Esperado: 409 duplicado, sin cambios.
4. Reemplazar un código existente sin marcar la confirmación. Esperado: «Asignar» deshabilitado;
   por API sin `reemplazar` → 409 `codigo_barras_existente`.
5. Con el usuario de operación: los botones no aparecen y la API responde 403.
6. Consultar Auditoría. Esperado: `ASIGNAR_CODIGO_BARRAS` y `REEMPLAZAR_CODIGO_BARRAS` con usuario
   y fecha (SC-059).

## 7. Etiquetas (US4)

1. Abrir la etiqueta del producto con EAN-13: los 3 formatos habilitados, vista previa 50 × 30 mm.
2. Producto con código interno: EAN-13 deshabilitado con motivo; Code 128 y QR codifican el código.
3. Producto sin código: Code 128 y QR codifican el SKU; EAN-13 ofrece «Generar código».
4. «Descargar SVG» e «Imprimir» (al 100 %). Escanear cada etiqueta con la cámara del celular.
   Esperado: las 3 resuelven al producto correcto (SC-056, SC-060).

## Evidencia

Guardar capturas del celular (escáner activo, resultado, errores, etiquetas) en
`specs/020-escaneo-codigos-barra/evidencia/` y registrar los resultados en `progress.md`.
