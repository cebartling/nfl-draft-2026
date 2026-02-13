import { describe, it, expect, vi, afterEach } from 'vitest';
import { formatDate, formatDateTime, formatRelativeTime } from './formatters';

describe('formatDate', () => {
	it('should format a valid ISO string', () => {
		const result = formatDate('2026-04-25T20:00:00Z');
		expect(result).toBeTruthy();
		// Just verify it returns a non-empty string (locale-dependent)
		expect(typeof result).toBe('string');
		expect(result.length).toBeGreaterThan(0);
	});

	it('should return empty string for null', () => {
		expect(formatDate(null)).toBe('');
	});

	it('should return empty string for undefined', () => {
		expect(formatDate(undefined)).toBe('');
	});

	it('should return empty string for empty string', () => {
		expect(formatDate('')).toBe('');
	});
});

describe('formatDateTime', () => {
	it('should format a valid ISO string with time', () => {
		const result = formatDateTime('2026-04-25T20:30:00Z');
		expect(result).toBeTruthy();
		expect(typeof result).toBe('string');
		expect(result.length).toBeGreaterThan(0);
	});

	it('should return empty string for null', () => {
		expect(formatDateTime(null)).toBe('');
	});

	it('should return empty string for undefined', () => {
		expect(formatDateTime(undefined)).toBe('');
	});
});

describe('formatRelativeTime', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('should return "just now" for less than 60 seconds ago', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-25T20:00:30Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('just now');
	});

	it('should return "X minutes ago"', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-25T20:05:00Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('5 minutes ago');
	});

	it('should return "1 minute ago" for singular', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-25T20:01:30Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('1 minute ago');
	});

	it('should return "X hours ago"', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-25T23:00:00Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('3 hours ago');
	});

	it('should return "1 hour ago" for singular', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-25T21:00:00Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('1 hour ago');
	});

	it('should return "X days ago"', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-28T20:00:00Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('3 days ago');
	});

	it('should return "1 day ago" for singular', () => {
		vi.useFakeTimers();
		const now = new Date('2026-04-26T20:00:00Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		expect(result).toBe('1 day ago');
	});

	it('should fall back to formatDate after 7 days', () => {
		vi.useFakeTimers();
		const now = new Date('2026-05-10T20:00:00Z');
		vi.setSystemTime(now);

		const result = formatRelativeTime('2026-04-25T20:00:00Z');
		// Should return formatDate output (non-empty, not a relative time string)
		expect(result).toBeTruthy();
		expect(result).not.toContain('ago');
	});

	it('should return empty string for null', () => {
		expect(formatRelativeTime(null)).toBe('');
	});

	it('should return empty string for undefined', () => {
		expect(formatRelativeTime(undefined)).toBe('');
	});
});
