import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { WebSocketClient, WebSocketState } from './websocket';
import type { ClientMessage, ServerMessage } from '$lib/types';

// Mock WebSocket
class MockWebSocket {
	static CONNECTING = 0;
	static OPEN = 1;
	static CLOSING = 2;
	static CLOSED = 3;

	readyState: number = MockWebSocket.CONNECTING;
	onopen: ((event: Event) => void) | null = null;
	onclose: ((event: CloseEvent) => void) | null = null;
	onmessage: ((event: MessageEvent) => void) | null = null;
	onerror: ((event: Event) => void) | null = null;

	constructor(public url: string) {}

	send(data: string): void {
		if (this.readyState !== MockWebSocket.OPEN) {
			throw new Error('WebSocket is not open');
		}
	}

	close(): void {
		this.readyState = MockWebSocket.CLOSED;
		if (this.onclose) {
			this.onclose(new CloseEvent('close'));
		}
	}

	// Helper method for tests
	simulateOpen(): void {
		this.readyState = MockWebSocket.OPEN;
		if (this.onopen) {
			this.onopen(new Event('open'));
		}
	}

	// Helper method for tests
	simulateMessage(data: object): void {
		if (this.onmessage) {
			this.onmessage(new MessageEvent('message', { data: JSON.stringify(data) }));
		}
	}

	// Helper method for tests
	simulateError(): void {
		if (this.onerror) {
			this.onerror(new Event('error'));
		}
	}

	// Helper method for tests
	simulateClose(): void {
		this.readyState = MockWebSocket.CLOSED;
		if (this.onclose) {
			this.onclose(new CloseEvent('close'));
		}
	}
}

describe('WebSocketClient', () => {
	let client: WebSocketClient;
	let mockWebSocket: MockWebSocket;

	beforeEach(() => {
		// Mock window.location
		Object.defineProperty(window, 'location', {
			value: {
				protocol: 'http:',
				host: 'localhost:5173',
			},
			writable: true,
		});

		// Mock WebSocket constructor
		globalThis.WebSocket = class {
			constructor(url: string) {
				mockWebSocket = new MockWebSocket(url);
				return mockWebSocket as any;
			}
		} as any;

		client = new WebSocketClient('/ws');
	});

	afterEach(() => {
		vi.restoreAllMocks();
		vi.unstubAllGlobals();
		if (client) {
			client.disconnect();
		}
	});

	describe('constructor', () => {
		it('should convert relative URL to WebSocket URL', () => {
			const wsClient = new WebSocketClient('/ws');
			expect(wsClient).toBeInstanceOf(WebSocketClient);
		});

		it('should use ws:// protocol for http://', () => {
			const wsClient = new WebSocketClient('/ws');
			wsClient.connect();
			expect(mockWebSocket.url).toBe('ws://localhost:5173/ws');
		});

		it('should use wss:// protocol for https://', () => {
			Object.defineProperty(window, 'location', {
				value: {
					protocol: 'https:',
					host: 'example.com',
				},
				writable: true,
			});

			const wsClient = new WebSocketClient('/ws');
			wsClient.connect();
			expect(mockWebSocket.url).toBe('wss://example.com/ws');
		});

		it('should accept absolute WebSocket URL', () => {
			const wsClient = new WebSocketClient('ws://localhost:8000/ws');
			wsClient.connect();
			expect(mockWebSocket.url).toBe('ws://localhost:8000/ws');
		});
	});

	describe('connect', () => {
		it('should create WebSocket connection', () => {
			client.connect();
			expect(WebSocket).toHaveBeenCalledWith('ws://localhost:5173/ws');
			expect(client.getState()).toBe(WebSocketState.Connecting);
		});

		it('should update state to connected on open', () => {
			client.connect();
			mockWebSocket.simulateOpen();
			expect(client.getState()).toBe(WebSocketState.Connected);
			expect(client.isConnected()).toBe(true);
		});

		it('should not reconnect if already connecting', () => {
			const connectSpy = vi.spyOn(WebSocket.prototype as any, 'constructor');
			client.connect();
			client.connect(); // Try to connect again
			expect(connectSpy).toHaveBeenCalledTimes(1);
		});

		it('should not reconnect if already open', () => {
			const connectSpy = vi.spyOn(WebSocket.prototype as any, 'constructor');
			client.connect();
			mockWebSocket.simulateOpen();
			client.connect(); // Try to connect again
			expect(connectSpy).toHaveBeenCalledTimes(1);
		});
	});

	describe('disconnect', () => {
		it('should close WebSocket connection', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const closeSpy = vi.spyOn(mockWebSocket, 'close');
			client.disconnect();

			expect(closeSpy).toHaveBeenCalled();
			expect(client.getState()).toBe(WebSocketState.Disconnected);
		});

		it('should prevent reconnection after disconnect', () => {
			client.connect();
			mockWebSocket.simulateOpen();
			mockWebSocket.simulateClose();

			// Disconnect should prevent auto-reconnection
			client.disconnect();

			// Wait for reconnection timeout (should not happen)
			expect(client.getState()).toBe(WebSocketState.Disconnected);
		});
	});

	describe('send', () => {
		it('should send valid message when connected', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const sendSpy = vi.spyOn(mockWebSocket, 'send');
			const message: ClientMessage = { type: 'ping' };

			client.send(message);

			expect(sendSpy).toHaveBeenCalledWith(JSON.stringify(message));
		});

		it('should not send message when disconnected', () => {
			const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
			const message: ClientMessage = { type: 'ping' };

			client.send(message);

			expect(consoleErrorSpy).toHaveBeenCalledWith('WebSocket is not connected');
		});

		it('should validate message before sending', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
			const invalidMessage = { type: 'invalid' } as unknown as ClientMessage;

			client.send(invalidMessage);

			expect(consoleErrorSpy).toHaveBeenCalledWith('Invalid message format:', expect.any(Object));
		});
	});

	describe('message handling', () => {
		it('should receive and parse valid server messages', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const handler = vi.fn();
			client.on(handler);

			const message: ServerMessage = {
				type: 'subscribed',
				session_id: '123e4567-e89b-12d3-a456-426614174000',
			};

			mockWebSocket.simulateMessage(message);

			expect(handler).toHaveBeenCalledWith(message);
		});

		it('should ignore pong messages', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const handler = vi.fn();
			client.on(handler);

			const pongMessage: ServerMessage = { type: 'pong' };

			mockWebSocket.simulateMessage(pongMessage);

			expect(handler).not.toHaveBeenCalled();
		});

		it('should log error for invalid messages', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
			const handler = vi.fn();
			client.on(handler);

			const invalidMessage = { type: 'invalid' };
			mockWebSocket.simulateMessage(invalidMessage);

			expect(consoleErrorSpy).toHaveBeenCalledWith('Invalid message format:', expect.any(Object));
			expect(handler).not.toHaveBeenCalled();
		});

		it('should handle malformed JSON', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
			const handler = vi.fn();
			client.on(handler);

			if (mockWebSocket.onmessage) {
				mockWebSocket.onmessage(new MessageEvent('message', { data: 'not json' }));
			}

			expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to parse WebSocket message:', expect.any(Error));
			expect(handler).not.toHaveBeenCalled();
		});
	});

	describe('event handlers', () => {
		it('should register message handler', () => {
			const handler = vi.fn();
			const unsubscribe = client.on(handler);

			expect(typeof unsubscribe).toBe('function');
		});

		it('should unregister message handler', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const handler = vi.fn();
			const unsubscribe = client.on(handler);

			const message: ServerMessage = {
				type: 'subscribed',
				session_id: '123e4567-e89b-12d3-a456-426614174000',
			};

			mockWebSocket.simulateMessage(message);
			expect(handler).toHaveBeenCalledTimes(1);

			unsubscribe();
			mockWebSocket.simulateMessage(message);
			expect(handler).toHaveBeenCalledTimes(1); // Should not be called again
		});

		it('should register state change handler', () => {
			const handler = vi.fn();
			const unsubscribe = client.onStateChange(handler);

			expect(typeof unsubscribe).toBe('function');

			client.connect();
			expect(handler).toHaveBeenCalledWith(WebSocketState.Connecting);

			mockWebSocket.simulateOpen();
			expect(handler).toHaveBeenCalledWith(WebSocketState.Connected);

			unsubscribe();
		});

		it('should unregister state change handler', () => {
			const handler = vi.fn();
			const unsubscribe = client.onStateChange(handler);

			client.connect();
			expect(handler).toHaveBeenCalledTimes(1);

			unsubscribe();
			mockWebSocket.simulateOpen();
			expect(handler).toHaveBeenCalledTimes(1); // Should not be called again
		});

		it('should handle errors in message handlers', () => {
			client.connect();
			mockWebSocket.simulateOpen();

			const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
			const handler = vi.fn(() => {
				throw new Error('Handler error');
			});
			client.on(handler);

			const message: ServerMessage = {
				type: 'subscribed',
				session_id: '123e4567-e89b-12d3-a456-426614174000',
			};

			mockWebSocket.simulateMessage(message);

			expect(consoleErrorSpy).toHaveBeenCalledWith('Error in message handler:', expect.any(Error));
		});
	});

	describe('reconnection', () => {
		it('should attempt reconnection on close', async () => {
			vi.useFakeTimers();

			client.connect();
			mockWebSocket.simulateOpen();
			mockWebSocket.simulateClose();

			expect(client.getState()).toBe(WebSocketState.Disconnected);

			// Wait for first reconnection attempt (1s)
			vi.advanceTimersByTime(1000);
			expect(client.getState()).toBe(WebSocketState.Reconnecting);

			vi.useRealTimers();
		});

		it('should use exponential backoff for reconnection', async () => {
			vi.useFakeTimers();
			const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

			client.connect();
			mockWebSocket.simulateOpen();
			mockWebSocket.simulateClose();

			// First attempt: 1s
			vi.advanceTimersByTime(1000);
			expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Reconnecting in 1000ms'));

			// Simulate failure
			mockWebSocket.simulateClose();

			// Second attempt: 2s
			vi.advanceTimersByTime(2000);
			expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Reconnecting in 2000ms'));

			vi.useRealTimers();
		});

		it('should stop reconnecting after max attempts', () => {
			vi.useFakeTimers();
			const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

			client.connect();
			mockWebSocket.simulateOpen();

			// Simulate 5 failed connections
			for (let i = 0; i < 5; i++) {
				mockWebSocket.simulateClose();
				vi.advanceTimersByTime(Math.pow(2, i) * 1000);
			}

			expect(consoleErrorSpy).toHaveBeenCalledWith('Max reconnection attempts reached');

			vi.useRealTimers();
		});
	});

	describe('ping/pong', () => {
		it('should send ping messages periodically', () => {
			vi.useFakeTimers();

			client.connect();
			mockWebSocket.simulateOpen();

			const sendSpy = vi.spyOn(mockWebSocket, 'send');

			// Advance time by 30 seconds (ping interval)
			vi.advanceTimersByTime(30000);

			expect(sendSpy).toHaveBeenCalledWith(JSON.stringify({ type: 'ping' }));

			vi.useRealTimers();
		});

		it('should stop sending pings after disconnect', () => {
			vi.useFakeTimers();

			client.connect();
			mockWebSocket.simulateOpen();

			const sendSpy = vi.spyOn(mockWebSocket, 'send');

			client.disconnect();

			// Advance time by 30 seconds
			vi.advanceTimersByTime(30000);

			expect(sendSpy).not.toHaveBeenCalled();

			vi.useRealTimers();
		});
	});

	describe('state management', () => {
		it('should return current state', () => {
			expect(client.getState()).toBe(WebSocketState.Disconnected);

			client.connect();
			expect(client.getState()).toBe(WebSocketState.Connecting);

			mockWebSocket.simulateOpen();
			expect(client.getState()).toBe(WebSocketState.Connected);
		});

		it('should check if connected', () => {
			expect(client.isConnected()).toBe(false);

			client.connect();
			expect(client.isConnected()).toBe(false);

			mockWebSocket.simulateOpen();
			expect(client.isConnected()).toBe(true);
		});
	});
});
