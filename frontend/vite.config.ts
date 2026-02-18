import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			// In dev, proxy /api to the backend (default 3099)
			'/api': {
				target: 'http://127.0.0.1:3099',
				changeOrigin: true
			}
		}
	}
});
