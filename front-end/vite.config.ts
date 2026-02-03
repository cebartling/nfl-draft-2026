import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		proxy: {
			'/api': {
				target: 'http://localhost:8000',
				changeOrigin: true,
			},
			'/ws': {
				target: 'ws://localhost:8000',
				ws: true,
			},
		},
	},
	test: {
		globals: true,
		environment: 'jsdom',
		setupFiles: ['./vitest.setup.ts'],
		coverage: {
			provider: 'v8',
			reporter: ['text', 'html', 'json'],
			exclude: [
				'node_modules/',
				'tests/',
				'**/*.spec.ts',
				'**/*.test.ts',
				'**/types/**',
				'vite.config.ts',
				'vitest.setup.ts',
				'playwright.config.ts',
			],
		},
	},
});
