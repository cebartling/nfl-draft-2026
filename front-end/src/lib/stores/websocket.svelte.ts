import { wsClient, WebSocketState } from '$lib/api';
import type { ServerMessage } from '$lib/types';
import { draftState } from './draft.svelte';

/**
 * WebSocket state management using Svelte 5 runes
 * Integrates WebSocket client with draft state
 */
export class WebSocketStateManager {
	// Reactive state
	connectionState = $state<WebSocketState>(WebSocketState.Disconnected);
	lastMessage = $state<ServerMessage | null>(null);
	error = $state<string | null>(null);

	private unsubscribeMessage?: () => void;
	private unsubscribeState?: () => void;

	constructor() {
		this.setupListeners();
	}

	/**
	 * Connect to WebSocket server
	 */
	connect(): void {
		wsClient.connect();
	}

	/**
	 * Disconnect from WebSocket server
	 */
	disconnect(): void {
		wsClient.disconnect();
	}

	/**
	 * Subscribe to a draft session
	 */
	subscribeToSession(sessionId: string): void {
		if (!wsClient.isConnected()) {
			console.warn('WebSocket not connected, cannot subscribe');
			return;
		}

		wsClient.send({
			type: 'subscribe',
			session_id: sessionId,
		});
	}

	/**
	 * Check if connected
	 */
	get isConnected(): boolean {
		return this.connectionState === WebSocketState.Connected;
	}

	/**
	 * Setup WebSocket listeners
	 */
	private setupListeners(): void {
		// Listen for messages
		this.unsubscribeMessage = wsClient.on((message) => {
			this.lastMessage = message;
			this.handleMessage(message);
		});

		// Listen for state changes
		this.unsubscribeState = wsClient.onStateChange((state) => {
			this.connectionState = state;
		});
	}

	/**
	 * Handle incoming WebSocket messages
	 */
	private handleMessage(message: ServerMessage): void {
		switch (message.type) {
			case 'subscribed':
				console.log('Subscribed to session:', message.session_id);
				break;

			case 'pick_made':
				console.log('Pick made:', message);
				// Update draft state with the new pick
				draftState.updatePickFromWS({
					pick_id: message.pick_id,
					player_id: message.player_id,
					team_id: message.team_id,
				});
				// Advance to the next pick
				draftState.advancePick();
				break;

			case 'clock_update':
				// Clock updates are handled by UI components that need to display the timer
				break;

			case 'draft_status':
				console.log('Draft status changed:', message.status);
				// Update session status if needed
				if (draftState.session) {
					draftState.session = {
						...draftState.session,
						status: message.status as any,
					};
				}
				break;

			case 'trade_proposed':
				console.log('Trade proposed:', message);
				// Trade proposals are handled by trade-specific UI
				break;

			case 'trade_executed':
				console.log('Trade executed:', message);
				// Reload draft picks to reflect trade
				if (draftState.draft) {
					draftState.loadDraft(draftState.draft.id);
				}
				break;

			case 'trade_rejected':
				console.log('Trade rejected:', message);
				break;

			case 'error':
				console.error('WebSocket error:', message.message);
				this.error = message.message;
				break;

			case 'pong':
				// Pong messages are handled internally by WebSocket client
				break;
		}
	}

	/**
	 * Cleanup listeners
	 */
	destroy(): void {
		if (this.unsubscribeMessage) {
			this.unsubscribeMessage();
		}
		if (this.unsubscribeState) {
			this.unsubscribeState();
		}
		this.disconnect();
	}
}

/**
 * Singleton WebSocket state instance
 */
export const websocketState = new WebSocketStateManager();
