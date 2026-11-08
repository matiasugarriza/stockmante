import { defineConfig } from 'vitest/config';
import tailwindcss from '@tailwindcss/vite';
import basicSsl from '@vitejs/plugin-basic-ssl';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';

/** Backend al que el modo `movil` reenvía `/api` (020, R7). */
const BACKEND_LOCAL = 'http://localhost:3000';

export default defineConfig(({ mode }) => {
	// 020: la cámara del celular exige HTTPS, y en el teléfono `localhost:3000` sería el propio
	// teléfono. En modo `movil` se sirve por HTTPS autofirmado y la API va por el mismo origen.
	const movil = mode === 'movil';

	return {
		plugins: [
			tailwindcss(),
			...(movil ? [basicSsl()] : []),
			sveltekit({
				compilerOptions: {
					// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
					runes: ({ filename }) =>
						filename.split(/[/\\]/).includes('node_modules') ? undefined : true
				},
				adapter: adapter({
					// SPA mode: serve index.html for all routes (no prerender server)
					fallback: 'index.html'
				})
			})
		],
		// Rutas relativas en el cliente: `client.ts` usa `PUBLIC_API_URL ?? localhost:3000`.
		define: movil ? { 'import.meta.env.PUBLIC_API_URL': JSON.stringify('') } : {},
		server: movil ? { proxy: { '/api': { target: BACKEND_LOCAL, changeOrigin: false } } } : {},
		test: {
			expect: { requireAssertions: true },
			projects: [
				{
					extends: './vite.config.ts',
					test: {
						name: 'server',
						environment: 'node',
						include: ['src/**/*.{test,spec}.{js,ts}'],
						exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
					}
				}
			]
		}
	};
});
