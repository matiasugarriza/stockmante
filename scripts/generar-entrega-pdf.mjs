/**
 * Genera el PDF de entrega del TP (Grupo 2) a partir del HTML de la carátula.
 *
 * Uso:
 *   node scripts/generar-entrega-pdf.mjs
 *
 * Salida:
 *   docs/entrega/entrega-grupo-2.pdf
 *   docs/entrega/preview-1.png, preview-2.png (control visual)
 */
import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const RAIZ = fileURLToPath(new URL('../', import.meta.url));
const HTML = join(RAIZ, 'docs/entrega/entrega-grupo-2.html');
const PDF = join(RAIZ, 'docs/entrega/entrega-grupo-2.pdf');

const browser = await chromium.launch();
const page = await browser.newPage({
	viewport: { width: 794, height: 1123 },
	deviceScaleFactor: 2
});

try {
	await page.goto(pathToFileURL(HTML).href, { waitUntil: 'networkidle' });

	// Esperar la tipografía y las imágenes antes de imprimir.
	await page.evaluate(async () => {
		await document.fonts.ready;
		await Promise.all(
			[...document.images].map((img) =>
				img.complete
					? Promise.resolve()
					: new Promise((resolve) => {
							img.onload = resolve;
							img.onerror = resolve;
						})
			)
		);
	});

	await mkdir(dirname(PDF), { recursive: true });
	await page.pdf({
		path: PDF,
		format: 'A4',
		printBackground: true,
		preferCSSPageSize: true,
		margin: { top: '0', right: '0', bottom: '0', left: '0' }
	});

	const paginas = page.locator('.page');
	const cantidad = await paginas.count();
	for (let i = 0; i < cantidad; i++) {
		await paginas.nth(i).screenshot({
			path: join(RAIZ, `docs/entrega/preview-${i + 1}.png`),
			scale: 'css'
		});
	}

	console.log(`PDF: ${PDF}`);
	console.log(`Páginas HTML: ${cantidad}`);
} finally {
	await page.close();
	await browser.close();
}
