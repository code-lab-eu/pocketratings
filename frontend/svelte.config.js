// Static adapter: build outputs a static site for the production stack (Caddy serves it).
// fallback: '200.html' enables SPA mode so client-side routing works for all paths.
// This only affects the output of `bun run build`; `bun run dev` is unchanged (same HMR, Vite dev server).
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({ fallback: '200.html' })
	}
};

export default config;
