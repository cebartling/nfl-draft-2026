type _LogLevel = 'debug' | 'info' | 'warn' | 'error';

class Logger {
	private isDevelopment = import.meta.env.DEV;

	debug(message: string, ...args: unknown[]): void {
		if (this.isDevelopment) {
			console.debug(`[DEBUG] ${message}`, ...args);
		}
	}

	info(message: string, ...args: unknown[]): void {
		if (this.isDevelopment) {
			console.log(`[INFO] ${message}`, ...args);
		}
	}

	warn(message: string, ...args: unknown[]): void {
		console.warn(`[WARN] ${message}`, ...args);
	}

	error(message: string, ...args: unknown[]): void {
		console.error(`[ERROR] ${message}`, ...args);
	}
}

export const logger = new Logger();
