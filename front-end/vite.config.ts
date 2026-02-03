import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [
		tailwindcss(), // IMPORTANT: Must come BEFORE sveltekit()
		sveltekit(),
	],
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
		exclude: [
			'**/node_modules/**',
			'**/dist/**',
			'**/.svelte-kit/**',
			'**/tests/**', // Exclude Playwright E2E tests
			'**/*.spec.ts', // Exclude Playwright spec files
		],
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
