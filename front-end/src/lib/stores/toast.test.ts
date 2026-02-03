import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ToastState, ToastType } from './toast.svelte';

describe('ToastState', () => {
	let toastState: ToastState;

	beforeEach(() => {
		toastState = new ToastState();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
		toastState.clear();
	});

	describe('show', () => {
		it('should add toast to state', () => {
			const id = toastState.show(ToastType.Success, 'Test message');

			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0]).toMatchObject({
				id,
				type: ToastType.Success,
				message: 'Test message',
			});
		});

		it('should auto-dismiss toast after duration', () => {
			toastState.show(ToastType.Success, 'Test message', 1000);

			expect(toastState.toasts).toHaveLength(1);

			vi.advanceTimersByTime(1000);

			expect(toastState.toasts).toHaveLength(0);
		});

		it('should not auto-dismiss when duration is 0', () => {
			toastState.show(ToastType.Success, 'Test message', 0);

			expect(toastState.toasts).toHaveLength(1);

			vi.advanceTimersByTime(10000);

			expect(toastState.toasts).toHaveLength(1);
		});

		it('should return unique IDs', () => {
			const id1 = toastState.show(ToastType.Success, 'Message 1');
			const id2 = toastState.show(ToastType.Success, 'Message 2');

			expect(id1).not.toBe(id2);
		});
	});

	describe('success', () => {
		it('should create success toast', () => {
			toastState.success('Success message');

			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0].type).toBe(ToastType.Success);
			expect(toastState.toasts[0].message).toBe('Success message');
		});
	});

	describe('error', () => {
		it('should create error toast', () => {
			toastState.error('Error message');

			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0].type).toBe(ToastType.Error);
			expect(toastState.toasts[0].message).toBe('Error message');
		});
	});

	describe('info', () => {
		it('should create info toast', () => {
			toastState.info('Info message');

			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0].type).toBe(ToastType.Info);
			expect(toastState.toasts[0].message).toBe('Info message');
		});
	});

	describe('warning', () => {
		it('should create warning toast', () => {
			toastState.warning('Warning message');

			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0].type).toBe(ToastType.Warning);
			expect(toastState.toasts[0].message).toBe('Warning message');
		});
	});

	describe('remove', () => {
		it('should remove toast by ID', () => {
			const id1 = toastState.success('Message 1');
			const id2 = toastState.success('Message 2');

			expect(toastState.toasts).toHaveLength(2);

			toastState.remove(id1);

			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0].id).toBe(id2);
		});

		it('should do nothing if ID does not exist', () => {
			toastState.success('Message');

			expect(toastState.toasts).toHaveLength(1);

			toastState.remove('non-existent-id');

			expect(toastState.toasts).toHaveLength(1);
		});
	});

	describe('clear', () => {
		it('should remove all toasts', () => {
			toastState.success('Message 1');
			toastState.error('Message 2');
			toastState.info('Message 3');

			expect(toastState.toasts).toHaveLength(3);

			toastState.clear();

			expect(toastState.toasts).toHaveLength(0);
		});
	});

	describe('multiple toasts', () => {
		it('should handle multiple toasts', () => {
			toastState.success('Message 1');
			toastState.error('Message 2');
			toastState.warning('Message 3');

			expect(toastState.toasts).toHaveLength(3);
			expect(toastState.toasts[0].type).toBe(ToastType.Success);
			expect(toastState.toasts[1].type).toBe(ToastType.Error);
			expect(toastState.toasts[2].type).toBe(ToastType.Warning);
		});

		it('should auto-dismiss multiple toasts independently', () => {
			toastState.show(ToastType.Success, 'Message 1', 1000);
			toastState.show(ToastType.Error, 'Message 2', 2000);

			expect(toastState.toasts).toHaveLength(2);

			vi.advanceTimersByTime(1000);
			expect(toastState.toasts).toHaveLength(1);
			expect(toastState.toasts[0].message).toBe('Message 2');

			vi.advanceTimersByTime(1000);
			expect(toastState.toasts).toHaveLength(0);
		});
	});
});
