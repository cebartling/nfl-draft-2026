import { browser } from '$app/environment';
import {
	ClientMessageSchema,
	ServerMessageSchema,
	type ClientMessage,
	type ServerMessage,
} from '$lib/types';
import { logger } from '$lib/utils/logger';

/**
 * WebSocket connection state
 */
export enum WebSocketState {
	Disconnected = 'disconnected',
	Connecting = 'connecting',
	Connected = 'connected',
	Reconnecting = 'reconnecting',
}

/**
 * Message handler callback type
 */
type MessageHandler = (message: ServerMessage) => void;

/**
 * State change handler callback type
 */
type StateChangeHandler = (state: WebSocketState) => void;

/**
 * WebSocket client with automatic reconnection and type-safe messaging
 */
export class WebSocketClient {
	private ws: WebSocket | null = null;
	private url: string;
	private state: WebSocketState = WebSocketState.Disconnected;
	private messageHandlers: Set<MessageHandler> = new Set();
	private stateChangeHandlers: Set<StateChangeHandler> = new Set();
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;
	private reconnectTimeoutId: ReturnType<typeof setTimeout> | null = null;
	private pingIntervalId: ReturnType<typeof setInterval> | null = null;
	private pingInterval = 30000; // 30 seconds

	constructor(url: string = '/ws') {
		// Store the URL - resolution to full WebSocket URL happens in connect()
		// This allows the class to be instantiated during SSR
		this.url = url;
	}

	/**
	 * Resolve relative URL to full WebSocket URL (must be called in browser)
	 */
	private resolveUrl(): string {
		if (!browser) {
			return this.url;
		}
		if (this.url.startsWith('/')) {
			const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
			return `${protocol}//${window.location.host}${this.url}`;
		}
		return this.url;
	}

	/**
	 * Connect to the WebSocket server
	 */
	connect(): void {
		// WebSocket is only available in browser environment
		if (!browser) {
			logger.warn('WebSocket connection skipped: not in browser environment');
			return;
		}

		if (
			this.ws &&
			(this.ws.readyState === WebSocket.CONNECTING || this.ws.readyState === WebSocket.OPEN)
		) {
			logger.warn('WebSocket is already connected or connecting');
			return;
		}

		this.setState(
			this.reconnectAttempts > 0 ? WebSocketState.Reconnecting : WebSocketState.Connecting
		);

		try {
			const resolvedUrl = this.resolveUrl();
			this.ws = new WebSocket(resolvedUrl);

			this.ws.onopen = () => {
				logger.info('WebSocket connected');
				this.reconnectAttempts = 0;
				this.setState(WebSocketState.Connected);
				this.startPingInterval();
			};

			this.ws.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					const result = ServerMessageSchema.safeParse(data);

					if (result.success) {
						// Handle pong messages internally
						if (result.data.type === 'pong') {
							return;
						}
						this.notifyMessageHandlers(result.data);
					} else {
						logger.error('Invalid message format:', result.error);
					}
				} catch (error) {
					logger.error('Failed to parse WebSocket message:', error);
				}
			};

			this.ws.onerror = (error) => {
				logger.error('WebSocket error:', error);
			};

			this.ws.onclose = () => {
				logger.info('WebSocket disconnected');
				this.setState(WebSocketState.Disconnected);
				this.stopPingInterval();
				this.attemptReconnect();
			};
		} catch (error) {
			logger.error('Failed to create WebSocket connection:', error);
			this.setState(WebSocketState.Disconnected);
			this.attemptReconnect();
		}
	}

	/**
	 * Disconnect from the WebSocket server
	 */
	disconnect(): void {
		this.reconnectAttempts = this.maxReconnectAttempts; // Prevent reconnection
		this.stopPingInterval();
		if (this.reconnectTimeoutId) {
			clearTimeout(this.reconnectTimeoutId);
			this.reconnectTimeoutId = null;
		}
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
		this.setState(WebSocketState.Disconnected);
	}

	/**
	 * Send a message to the server
	 */
	send(message: ClientMessage): void {
		if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
			logger.error('WebSocket is not connected');
			return;
		}

		// Validate message before sending
		const result = ClientMessageSchema.safeParse(message);
		if (!result.success) {
			logger.error('Invalid message format:', result.error);
			return;
		}

		this.ws.send(JSON.stringify(result.data));
	}

	/**
	 * Register a message handler
	 */
	on(handler: MessageHandler): () => void {
		this.messageHandlers.add(handler);
		// Return unsubscribe function
		return () => {
			this.messageHandlers.delete(handler);
		};
	}

	/**
	 * Register a state change handler
	 */
	onStateChange(handler: StateChangeHandler): () => void {
		this.stateChangeHandlers.add(handler);
		// Return unsubscribe function
		return () => {
			this.stateChangeHandlers.delete(handler);
		};
	}

	/**
	 * Get the current connection state
	 */
	getState(): WebSocketState {
		return this.state;
	}

	/**
	 * Check if the WebSocket is connected
	 */
	isConnected(): boolean {
		return this.state === WebSocketState.Connected;
	}

	/**
	 * Attempt to reconnect with exponential backoff
	 */
	private attemptReconnect(): void {
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			logger.error('Max reconnection attempts reached');
			return;
		}

		// Exponential backoff: 1s, 2s, 4s, 8s, 16s
		const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 16000);
		this.reconnectAttempts++;

		logger.info(
			`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
		);

		this.reconnectTimeoutId = setTimeout(() => {
			this.connect();
		}, delay);
	}

	/**
	 * Start ping interval to keep connection alive
	 */
	private startPingInterval(): void {
		this.stopPingInterval();
		this.pingIntervalId = setInterval(() => {
			if (this.isConnected()) {
				this.send({ type: 'ping' });
			}
		}, this.pingInterval);
	}

	/**
	 * Stop ping interval
	 */
	private stopPingInterval(): void {
		if (this.pingIntervalId) {
			clearInterval(this.pingIntervalId);
			this.pingIntervalId = null;
		}
	}

	/**
	 * Update state and notify handlers
	 */
	private setState(state: WebSocketState): void {
		this.state = state;
		this.notifyStateChangeHandlers(state);
	}

	/**
	 * Notify all message handlers
	 */
	private notifyMessageHandlers(message: ServerMessage): void {
		this.messageHandlers.forEach((handler) => {
			try {
				handler(message);
			} catch (error) {
				logger.error('Error in message handler:', error);
			}
		});
	}

	/**
	 * Notify all state change handlers
	 */
	private notifyStateChangeHandlers(state: WebSocketState): void {
		this.stateChangeHandlers.forEach((handler) => {
			try {
				handler(state);
			} catch (error) {
				logger.error('Error in state change handler:', error);
			}
		});
	}
}

/**
 * Singleton WebSocket client instance
 */
export const wsClient = new WebSocketClient();
