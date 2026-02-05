import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import prettier from 'eslint-config-prettier';

/** @type {import('eslint').Linter.Config[]} */
export default [
	// Ignore patterns
	{
		ignores: [
			'**/.svelte-kit/**',
			'**/build/**',
			'**/dist/**',
			'**/node_modules/**',
			'**/.DS_Store',
			'**/package-lock.json',
			'**/pnpm-lock.yaml',
			'**/yarn.lock',
			'.env',
			'.env.*',
			'!.env.example',
			'vite.config.*.timestamp-*',
		],
	},

	// Base JavaScript recommended rules
	js.configs.recommended,

	// TypeScript configuration for .ts files
	...ts.configs.recommended,

	// Svelte configuration
	...svelte.configs['flat/recommended'],

	// Global configuration for all files
	{
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node,
				...globals.es2022,
			},
			parserOptions: {
				ecmaVersion: 2022,
				sourceType: 'module',
				extraFileExtensions: ['.svelte'],
			},
		},
		rules: {
			// General code quality
			'no-console': ['warn', { allow: ['warn', 'error'] }],
			'no-debugger': 'warn',
			'no-unused-vars': 'off', // Turned off in favor of TypeScript version

			// Consistency
			'prefer-const': 'error',
			'no-var': 'error',
			eqeqeq: ['error', 'always', { null: 'ignore' }],
		},
	},

	// Logger utility - allow console statements
	{
		files: ['**/logger.ts', '**/logger.js'],
		rules: {
			'no-console': 'off',
		},
	},

	// Scripts - allow console statements for CLI output
	{
		files: ['scripts/**/*.ts', 'scripts/**/*.js'],
		rules: {
			'no-console': 'off',
		},
	},

	// TypeScript-specific overrides
	{
		files: ['**/*.ts', '**/*.tsx'],
		// Exclude config files that aren't in tsconfig.json
		ignores: ['*.config.ts', '*.config.js', 'playwright.config.ts', 'vitest.setup.ts', 'scripts/**'],
		languageOptions: {
			parser: ts.parser,
			parserOptions: {
				project: './tsconfig.json',
			},
		},
		rules: {
			'@typescript-eslint/no-unused-vars': [
				'warn',
				{
					argsIgnorePattern: '^_',
					varsIgnorePattern: '^_',
					caughtErrorsIgnorePattern: '^_',
				},
			],
			'@typescript-eslint/no-explicit-any': 'warn',
			'@typescript-eslint/explicit-function-return-type': 'off',
			'@typescript-eslint/explicit-module-boundary-types': 'off',
			'@typescript-eslint/no-non-null-assertion': 'warn',
		},
	},

	// Svelte-specific overrides
	{
		files: ['**/*.svelte'],
		languageOptions: {
			parser: svelte.parser,
			parserOptions: {
				parser: ts.parser,
				svelteFeatures: {
					experimentalGenerics: true,
				},
			},
		},
		rules: {
			// Svelte-specific rules
			'svelte/no-at-html-tags': 'warn',
			'svelte/no-target-blank': 'error',
			'svelte/no-unused-svelte-ignore': 'warn',
			'svelte/valid-compile': 'warn', // Changed to warn for accessibility issues
			'svelte/require-each-key': 'warn', // Warn instead of error
			'svelte/prefer-svelte-reactivity': 'off', // Too strict for Svelte 5 runes
			'svelte/no-navigation-without-resolve': 'off', // Disabled - using invalidateAll() pattern instead

			// Relax some rules for Svelte files
			'@typescript-eslint/no-unused-vars': 'off', // Svelte reactive declarations can look unused
			'no-undef': 'off', // Svelte compiler handles this
			'prefer-const': 'off', // Svelte 5 runes use `let` with $props() and $state()
		},
	},

	// Svelte TypeScript files (*.svelte.ts) - stores with runes
	{
		files: ['**/*.svelte.ts', '**/*.svelte.js'],
		rules: {
			'svelte/prefer-svelte-reactivity': 'off', // Allow standard JS classes in runes-based stores
		},
	},

	// Test files configuration
	{
		files: [
			'**/*.test.ts',
			'**/*.spec.ts',
			'**/*.test.js',
			'**/*.spec.js',
			'**/vitest.setup.ts',
			'**/test-utils/**',
		],
		languageOptions: {
			globals: {
				...globals.node,
				describe: 'readonly',
				it: 'readonly',
				expect: 'readonly',
				vi: 'readonly',
				beforeEach: 'readonly',
				afterEach: 'readonly',
				beforeAll: 'readonly',
				afterAll: 'readonly',
			},
		},
		rules: {
			// Relax rules for test files
			'@typescript-eslint/no-explicit-any': 'off',
			'@typescript-eslint/no-non-null-assertion': 'off',
			'no-console': 'off',
		},
	},

	// Playwright E2E test files
	{
		files: ['tests/**/*.ts', 'tests/**/*.js'],
		languageOptions: {
			globals: {
				...globals.node,
			},
		},
		rules: {
			'no-console': 'off',
		},
	},

	// Prettier integration (must be last to override conflicting rules)
	prettier,
];
