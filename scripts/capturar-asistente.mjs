/**
 * Captura de evidencia del asistente (spec 017).
 *
 * Requiere backend, frontend y asistente levantados, y un usuario GESTION sin
 * cambio de contraseña pendiente:
 *
 *   CAPTURE_EMAIL=... CAPTURE_PASSWORD=... node scripts/capturar-asistente.mjs
 *
 * Salida: specs/017-asistente-inventario/evidencia/asistente-demo.png
 */
import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const BASE = process.env.CAPTURE_URL ?? 'http://localhost:5173';
const EMAIL = process.env.CAPTURE_EMAIL;
const PASSWORD = process.env.CAPTURE_PASSWORD;
const PREGUNTA =
	process.env.CAPTURE_PREGUNTA ?? '¿Cuánto stock hay de Aceite girasol 1,5 L?';

const RAIZ = fileURLToPath(new URL('../', import.meta.url));
const SALIDA = join(RAIZ, 'specs/017-asistente-inventario/evidencia/asistente-demo.png');

if (!EMAIL || !PASSWORD) {
	console.error('Definí CAPTURE_EMAIL y CAPTURE_PASSWORD (usuario GESTION sin cambio pendiente).');
	process.exit(1);
}

const browser = await chromium.launch();
const contexto = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await contexto.newPage();

try {
	await page.goto(`${BASE}/login`, { waitUntil: 'networkidle' });
	await page.locator('input[type="email"]').fill(EMAIL);
	await page.locator('input[type="password"]').first().fill(PASSWORD);
	await page.locator('button[type="submit"]').click();
	await page.waitForURL((url) => !url.pathname.startsWith('/login'), { timeout: 20_000 });

	await page.goto(`${BASE}/asistente`, { waitUntil: 'networkidle' });
	await page.locator('input[name="mensaje-asistente"]').fill(PREGUNTA);
	await page.getByRole('button', { name: 'Enviar' }).click();

	// Espera a que aparezcan el mensaje del usuario y la respuesta del asistente.
	await page.waitForFunction(
		() => document.querySelectorAll('.mensaje').length >= 2,
		null,
		{ timeout: 90_000 }
	);
	await page.waitForTimeout(600);

	await mkdir(dirname(SALIDA), { recursive: true });
	await page.screenshot({ path: SALIDA, fullPage: true });
	console.log(`captura: ${SALIDA}`);
} finally {
	await contexto.close();
	await browser.close();
}
