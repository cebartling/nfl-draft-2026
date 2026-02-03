import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/svelte';
import * as matchers from '@testing-library/jest-dom/matchers';

// Extend Vitest's expect with Testing Library matchers
expect.extend(matchers);

// Cleanup after each test
afterEach(() => {
	cleanup();
});

// Mock window.matchMedia (used by some UI components)
Object.defineProperty(window, 'matchMedia', {
	writable: true,
	value: (query: string) => ({
		matches: false,
		media: query,
		onchange: null,
		addListener: () => {}, // Deprecated
		removeListener: () => {}, // Deprecated
		addEventListener: () => {},
		removeEventListener: () => {},
		dispatchEvent: () => true,
	}),
});

// Mock IntersectionObserver (used by lazy loading components)
global.IntersectionObserver = class IntersectionObserver {
	constructor() {}
	disconnect() {}
	observe() {}
	takeRecords() {
		return [];
	}
	unobserve() {}
} as any;

// Mock ResizeObserver (used by some UI components)
global.ResizeObserver = class ResizeObserver {
	constructor() {}
	disconnect() {}
	observe() {}
	unobserve() {}
} as any;

// Suppress console errors during tests (optional)
// const originalError = console.error;
// beforeAll(() => {
// 	console.error = (...args: any[]) => {
// 		if (
// 			typeof args[0] === 'string' &&
// 			args[0].includes('Not implemented: HTMLFormElement.prototype.submit')
// 		) {
// 			return;
// 		}
// 		originalError.call(console, ...args);
// 	};
// });

// afterAll(() => {
// 	console.error = originalError;
// });
