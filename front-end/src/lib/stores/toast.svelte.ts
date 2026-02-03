/**
 * Toast notification types
 */
export enum ToastType {
	Success = 'success',
	Error = 'error',
	Info = 'info',
	Warning = 'warning',
}

/**
 * Toast notification interface
 */
export interface Toast {
	id: string;
	type: ToastType;
	message: string;
	duration: number;
}

/**
 * Toast state management using Svelte 5 runes
 */
export class ToastState {
	// Reactive state
	toasts = $state<Toast[]>([]);
	private nextId = 0;

	/**
	 * Show a toast notification
	 */
	show(type: ToastType, message: string, duration: number = 5000): string {
		const id = `toast-${this.nextId++}`;
		const toast: Toast = { id, type, message, duration };

		this.toasts = [...this.toasts, toast];

		// Auto-dismiss after duration
		if (duration > 0) {
			setTimeout(() => {
				this.remove(id);
			}, duration);
		}

		return id;
	}

	/**
	 * Show a success toast
	 */
	success(message: string, duration?: number): string {
		return this.show(ToastType.Success, message, duration);
	}

	/**
	 * Show an error toast
	 */
	error(message: string, duration?: number): string {
		return this.show(ToastType.Error, message, duration);
	}

	/**
	 * Show an info toast
	 */
	info(message: string, duration?: number): string {
		return this.show(ToastType.Info, message, duration);
	}

	/**
	 * Show a warning toast
	 */
	warning(message: string, duration?: number): string {
		return this.show(ToastType.Warning, message, duration);
	}

	/**
	 * Remove a toast by ID
	 */
	remove(id: string): void {
		this.toasts = this.toasts.filter((toast) => toast.id !== id);
	}

	/**
	 * Clear all toasts
	 */
	clear(): void {
		this.toasts = [];
	}
}

/**
 * Singleton toast state instance
 */
export const toastState = new ToastState();
