/**
 * Evidencia móvil de la spec 019 (Playwright).
 *
 * Requiere el frontend en marcha (por defecto http://localhost:5173) y credenciales de un
 * usuario GESTION en CAPTURE_EMAIL / CAPTURE_PASSWORD. Recorre las 21 rutas en tres tamaños.
 */
import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

const BASE = process.env.CAPTURE_URL ?? 'http://localhost:5173';
const EMAIL = process.env.CAPTURE_EMAIL;
const PASSWORD = process.env.CAPTURE_PASSWORD;
const SALIDA = 'specs/019-rediseno-visual-mobile/evidencia';

const RUTAS = [
	['01-login', '/login'],
	['02-registro', '/registro'],
	['03-verificar-correo', '/verificar-correo'],
	['04-inicio', '/'],
	['05-caja', '/caja'],
	['06-movimientos', '/movimientos'],
	['07-transferencias', '/transferencias'],
	['08-conteos', '/conteos'],
	['09-asistente', '/asistente'],
	['10-admin-usuarios', '/admin/usuarios'],
	['11-admin-categorias', '/admin/categorias'],
	['12-admin-productos', '/admin/productos'],
	['13-admin-depositos', '/admin/depositos'],
	['14-admin-stock', '/admin/stock'],
	['15-admin-proveedores', '/admin/proveedores'],
	['16-admin-compras', '/admin/compras'],
	['17-admin-auditoria', '/admin/auditoria'],
	['18-admin-ventas', '/admin/ventas'],
	['19-admin-clientes', '/admin/clientes'],
	['20-admin-reportes', '/admin/reportes'],
	['21-admin-importacion', '/admin/importacion']
];

const TAMANOS = [
	['vertical', { width: 390, height: 844 }],
	['horizontal', { width: 844, height: 390 }],
	['compacta', { width: 320, height: 568 }]
];

if (!EMAIL || !PASSWORD) {
	console.error('Faltan CAPTURE_EMAIL y CAPTURE_PASSWORD en el entorno.');
	process.exit(1);
}

const navegador = await chromium.launch();
for (const [carpeta, viewport] of TAMANOS) {
	const contexto = await navegador.newContext({ viewport });
	const pagina = await contexto.newPage();
	await pagina.goto(`${BASE}/login`);
	await pagina.fill('input[name="email"]', EMAIL);
	await pagina.fill('input[name="password"]', PASSWORD);
	await pagina.click('button[type="submit"]');
	await pagina.waitForURL((url) => !url.pathname.includes('/login'), { timeout: 15000 });
	for (const [nombre, ruta] of RUTAS) {
		if (ruta === '/login' || ruta === '/registro' || ruta === '/verificar-correo') continue;
		await pagina.goto(`${BASE}${ruta}`);
		await pagina.waitForLoadState('networkidle');
		const destino = join(SALIDA, carpeta);
		await mkdir(destino, { recursive: true });
		await pagina.screenshot({ path: join(destino, `${nombre}.png`), fullPage: false });
	}
	await contexto.close();
}
await navegador.close();
console.log('Evidencia generada en', SALIDA);
