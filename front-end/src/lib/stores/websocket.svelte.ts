import { wsClient, WebSocketState } from '$lib/api';
import type { ServerMessage, SessionStatus } from '$lib/types';
import { draftState } from './draft.svelte';
import { logger } from '$lib/utils/logger';

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
	private pendingSessionId: string | null = null;

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
	 * Subscribe to a draft session.
	 * If the WebSocket is not yet connected, stores the session ID
	 * and auto-subscribes once the connection is established.
	 */
	subscribeToSession(sessionId: string): void {
		this.pendingSessionId = sessionId;

		if (!wsClient.isConnected()) {
			logger.info('WebSocket not yet connected, will subscribe on connect');
			return;
		}

		this.sendSubscribe(sessionId);
	}

	private sendSubscribe(sessionId: string): void {
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

			// Auto-subscribe to pending session when connection is established
			if (state === WebSocketState.Connected && this.pendingSessionId) {
				this.sendSubscribe(this.pendingSessionId);
			}
		});
	}

	/**
	 * Handle incoming WebSocket messages
	 */
	private handleMessage(message: ServerMessage): void {
		switch (message.type) {
			case 'subscribed':
				logger.info('Subscribed to session:', message.session_id);
				break;

			case 'pick_made':
				logger.info('Pick made:', message);
				// Update draft state with the new pick
				draftState.updatePickFromWS({
					pick_id: message.pick_id,
					player_id: message.player_id,
					team_id: message.team_id,
				});
				// Add notification for the activity feed
				draftState.addPickNotification({
					pick_id: message.pick_id,
					player_id: message.player_id,
					team_id: message.team_id,
					player_name: message.player_name,
					team_name: message.team_name,
					round: message.round,
					pick_number: message.pick_number,
				});
				// Always advance pick on WS pick_made — the HTTP response will
				// reconcile with the authoritative session state afterwards.
				draftState.advancePick();
				break;

			case 'clock_update':
				// Clock updates are handled by UI components that need to display the timer
				break;

			case 'draft_status':
				logger.info('Draft status changed:', message.status);
				// Update session status if needed
				if (draftState.session) {
					draftState.session = {
						...draftState.session,
						status: message.status as SessionStatus,
					};
				}
				break;

			case 'trade_proposed':
				logger.info('Trade proposed:', message);
				// Trade proposals are handled by trade-specific UI
				break;

			case 'trade_executed':
				logger.info('Trade executed:', message);
				// Reload draft picks to reflect trade
				if (draftState.draft) {
					draftState.loadDraft(draftState.draft.id);
				}
				break;

			case 'trade_rejected':
				logger.info('Trade rejected:', message);
				break;

			case 'error':
				logger.error('WebSocket error:', message.message);
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
		this.pendingSessionId = null;
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
