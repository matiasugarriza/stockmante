/**
 * Capturas de evidencia del rediseño (spec 018, T005).
 *
 * Requiere backend y frontend levantados, y un usuario GESTION sin cambio de
 * contraseña pendiente:
 *
 *   CAPTURE_EMAIL=... CAPTURE_PASSWORD=... node scripts/capturar-rediseno.mjs
 *
 * Salida:
 *   specs/018-rediseno-visual-desktop/evidencia/claro/<captura>.png
 *   specs/018-rediseno-visual-desktop/evidencia/oscuro/<captura>.png
 *   specs/018-rediseno-visual-desktop/evidencia/angosto/<ruta>.png
 */
import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const BASE = process.env.CAPTURE_URL ?? 'http://localhost:5173';
const EMAIL = process.env.CAPTURE_EMAIL;
const PASSWORD = process.env.CAPTURE_PASSWORD;
const EMAIL_TEMP = process.env.CAPTURE_EMAIL_TEMP;
const PASSWORD_TEMP = process.env.CAPTURE_PASSWORD_TEMP;

const RAIZ = fileURLToPath(
	new URL('../specs/018-rediseno-visual-desktop/evidencia/', import.meta.url)
);
const TEMAS = ['light', 'dark'];

const PUBLICAS = [
	{ archivo: '01-login.png', ruta: '/login' },
	{ archivo: '02-registro.png', ruta: '/registro' },
	{ archivo: '03-verificar-correo.png', ruta: '/verificar-correo' }
];

const PRIVADAS = [
	{ archivo: '04-inicio.png', ruta: '/' },
	{ archivo: '05-caja.png', ruta: '/caja', angosta: true },
	{ archivo: '06-movimientos.png', ruta: '/movimientos', angosta: true },
	{ archivo: '07-transferencias.png', ruta: '/transferencias', angosta: true },
	{ archivo: '08-conteos.png', ruta: '/conteos', angosta: true },
	{ archivo: '09-asistente.png', ruta: '/asistente' },
	{ archivo: '10-admin-usuarios.png', ruta: '/admin/usuarios', angosta: true },
	{ archivo: '11-admin-categorias.png', ruta: '/admin/categorias', angosta: true },
	{ archivo: '12-admin-productos.png', ruta: '/admin/productos', angosta: true },
	{ archivo: '13-admin-depositos.png', ruta: '/admin/depositos', angosta: true },
	{ archivo: '14-admin-stock.png', ruta: '/admin/stock', angosta: true },
	{ archivo: '15-admin-proveedores.png', ruta: '/admin/proveedores', angosta: true },
	{ archivo: '16-admin-compras.png', ruta: '/admin/compras', angosta: true },
	{ archivo: '17-admin-auditoria.png', ruta: '/admin/auditoria', angosta: true }
];

if (!EMAIL || !PASSWORD) {
	console.error('Definí CAPTURE_EMAIL y CAPTURE_PASSWORD (usuario GESTION sin cambio pendiente).');
	process.exit(1);
}

const dirTema = (tema) => join(RAIZ, tema === 'light' ? 'claro' : 'oscuro');
const dirAngosto = join(RAIZ, 'angosto');

async function prepararContexto(browser, tema, viewport) {
	const contexto = await browser.newContext({ viewport, deviceScaleFactor: 1 });
	await contexto.addInitScript((valor) => {
		window.localStorage.setItem('stockmante:tema', valor);
	}, tema);
	return contexto;
}

async function esperarEstable(page) {
	await page.waitForLoadState('networkidle');
	await page.evaluate(() => document.fonts.ready.then(() => true));
	await page.waitForTimeout(350);
}

async function iniciarSesion(page) {
	await page.goto(`${BASE}/login`, { waitUntil: 'networkidle' });
	await page.locator('input[type="email"]').fill(EMAIL);
	await page.locator('input[type="password"]').first().fill(PASSWORD);
	await page.locator('button[type="submit"]').click();
	await page.waitForURL((url) => !url.pathname.startsWith('/login'), { timeout: 20_000 });
	await esperarEstable(page);
}

async function capturarTema(browser, tema) {
	const salida = dirTema(tema);
	await mkdir(salida, { recursive: true });

	// Públicas: contexto limpio, sin sesión.
	const contextoPublico = await prepararContexto(browser, tema, { width: 1440, height: 900 });
	const paginaPublica = await contextoPublico.newPage();
	for (const item of PUBLICAS) {
		await paginaPublica.goto(`${BASE}${item.ruta}`, { waitUntil: 'networkidle' });
		await esperarEstable(paginaPublica);
		await paginaPublica.screenshot({ path: join(salida, item.archivo) });
	}
	await contextoPublico.close();

	// Privadas: sesión iniciada.
	const contexto = await prepararContexto(browser, tema, { width: 1440, height: 900 });
	const page = await contexto.newPage();
	await iniciarSesion(page);
	for (const item of PRIVADAS) {
		await page.goto(`${BASE}${item.ruta}`, { waitUntil: 'networkidle' });
		await esperarEstable(page);
		await page.screenshot({ path: join(salida, item.archivo) });
	}
	await contexto.close();
}

async function capturarAngosto(browser) {
	await mkdir(dirAngosto, { recursive: true });
	const contexto = await prepararContexto(browser, 'light', { width: 320, height: 800 });
	const page = await contexto.newPage();
	await iniciarSesion(page);
	for (const item of PRIVADAS.filter((ruta) => ruta.angosta)) {
		await page.goto(`${BASE}${item.ruta}`, { waitUntil: 'networkidle' });
		await esperarEstable(page);
		const desborde = await page.evaluate(
			() => document.documentElement.scrollWidth > window.innerWidth + 1
		);
		if (desborde) {
			throw new Error(`Desborde horizontal a 320 px en ${item.ruta}`);
		}
		await page.screenshot({
			path: join(dirAngosto, item.archivo)
		});
	}
	await contexto.close();
}

async function capturarCambioPassword(browser) {
	if (!EMAIL_TEMP || !PASSWORD_TEMP) {
		console.log('Sin CAPTURE_EMAIL_TEMP/CAPTURE_PASSWORD_TEMP: se omite el modo de cambio.');
		return;
	}
	const contexto = await prepararContexto(browser, 'light', { width: 1440, height: 900 });
	const page = await contexto.newPage();
	await page.goto(`${BASE}/login`, { waitUntil: 'networkidle' });
	await page.locator('input[type="email"]').fill(EMAIL_TEMP);
	await page.locator('input[type="password"]').first().fill(PASSWORD_TEMP);
	await page.locator('button[type="submit"]').click();
	await page.waitForTimeout(800);
	await page.screenshot({
		path: join(dirTema('light'), '18-cambio-password.png')
	});
	await contexto.close();
}

const browser = await chromium.launch();
try {
	for (const tema of TEMAS) {
		await capturarTema(browser, tema);
		console.log(`Capturas ${tema}: ${PUBLICAS.length + PRIVADAS.length}`);
	}
	await capturarAngosto(browser);
	await capturarCambioPassword(browser);
	console.log('Evidencia completa en specs/018-rediseno-visual-desktop/evidencia/');
} finally {
	await browser.close();
}
