// 004 T023 — Mide SC-002: p95 de GET /api/productos/buscar visto desde el cliente
// (desde el fetch hasta terminar de parsear el JSON, sin debounce), sobre el seed SEED4-.
//
// Uso (Node 18+, sin dependencias):
//   STOCKMANTE_URL=http://localhost:3000 STOCKMANTE_EMAIL=... STOCKMANTE_PASSWORD=... \
//     node scripts/medir_sc002.mjs
//
// Ojo: la sesión es única por usuario (001); el login de este script cierra otras sesiones
// abiertas de ese mismo usuario.

const CONSULTAS_POR_TIPO = 40;
const META_P95_MS = 1000;

const faltantes = ['STOCKMANTE_URL', 'STOCKMANTE_EMAIL', 'STOCKMANTE_PASSWORD'].filter(
	(nombre) => !process.env[nombre]
);
if (faltantes.length > 0) {
	console.error(
		`Faltan variables de entorno: ${faltantes.join(', ')}. ` +
			'Definilas con la URL del backend y un usuario de desarrollo, y volvé a ejecutar.'
	);
	process.exit(2);
}

const base = process.env.STOCKMANTE_URL.replace(/\/$/, '');

async function iniciarSesion() {
	const respuesta = await fetch(`${base}/api/auth/login`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({
			email: process.env.STOCKMANTE_EMAIL,
			password: process.env.STOCKMANTE_PASSWORD
		})
	});
	if (!respuesta.ok) {
		throw new Error(
			`El login respondió ${respuesta.status}. Revisá STOCKMANTE_EMAIL y STOCKMANTE_PASSWORD.`
		);
	}
	const cookie = respuesta.headers
		.getSetCookie()
		.find((valor) => valor.startsWith('stockmante_sesion='));
	if (!cookie) throw new Error('El login no devolvió la cookie stockmante_sesion.');
	return cookie.split(';')[0];
}

function numero(i) {
	return String(1 + ((i * 251) % 10000)).padStart(5, '0');
}

// Términos que existen en el seed; se varían para no medir siempre la misma fila.
const RUBROS = ['yerba', 'café', 'harina', 'aceite', 'galletitas', 'detergente', 'gaseosa', 'atún'];
const TIPOS = {
	sku: (i) => `SEED4-${numero(i)}`,
	codigo: (i) => `7790999${numero(i).padStart(6, '0')}`,
	nombre: (i) => RUBROS[i % RUBROS.length],
	categoria: (i) => `seed4-cat-${String(1 + (i % 20)).padStart(2, '0')}`
};

function percentil(valores, p) {
	const ordenados = [...valores].sort((a, b) => a - b);
	const indice = Math.min(ordenados.length - 1, Math.ceil((p / 100) * ordenados.length) - 1);
	return ordenados[indice];
}

async function medir(cookie, campo, termino) {
	const url = `${base}/api/productos/buscar?q=${encodeURIComponent(termino)}&campo=${campo}&limite=20`;
	const inicio = performance.now();
	const respuesta = await fetch(url, { headers: { Cookie: cookie } });
	const cuerpo = await respuesta.json();
	const ms = performance.now() - inicio;
	if (respuesta.status !== 200) {
		throw new Error(
			`campo=${campo} q=${termino} respondió ${respuesta.status}: ${cuerpo?.error?.mensaje ?? 'sin mensaje'}`
		);
	}
	return { ms, cantidad: cuerpo.items.length };
}

async function principal() {
	const cookie = await iniciarSesion();
	// Una consulta de calentamiento por tipo para no medir el arranque de conexiones.
	for (const [campo, termino] of Object.entries(TIPOS)) await medir(cookie, campo, termino(0));

	const filas = [];
	let cumpleTodo = true;
	for (const [campo, termino] of Object.entries(TIPOS)) {
		const tiempos = [];
		let sinResultados = 0;
		for (let i = 1; i <= CONSULTAS_POR_TIPO; i++) {
			const { ms, cantidad } = await medir(cookie, campo, termino(i));
			tiempos.push(ms);
			if (cantidad === 0) sinResultados++;
		}
		const p95 = percentil(tiempos, 95);
		const cumple = p95 <= META_P95_MS;
		cumpleTodo &&= cumple;
		filas.push({
			campo,
			consultas: tiempos.length,
			sin_resultados: sinResultados,
			p50_ms: percentil(tiempos, 50).toFixed(1),
			p95_ms: p95.toFixed(1),
			max_ms: Math.max(...tiempos).toFixed(1),
			cumple_p95_1s: cumple ? 'sí' : 'NO'
		});
	}
	console.table(filas);
	console.log(
		cumpleTodo
			? `SC-002 cumplido: p95 ≤ ${META_P95_MS} ms en los cuatro tipos.`
			: `SC-002 NO cumplido: al menos un tipo supera p95 de ${META_P95_MS} ms. Registrá EXPLAIN (ANALYZE, BUFFERS) de esa consulta.`
	);
	process.exit(cumpleTodo ? 0 : 1);
}

principal().catch((e) => {
	console.error(`No se pudo completar la medición: ${e.message}`);
	process.exit(2);
});
