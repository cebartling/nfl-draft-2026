import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ServerMessage } from '$lib/types';

// Use vi.hoisted to create mock objects that can be referenced in vi.mock factories
const { mockWsClient, mockDraftState, WebSocketState } = vi.hoisted(() => {
	const WebSocketState = {
		Disconnected: 'disconnected',
		Connecting: 'connecting',
		Connected: 'connected',
		Reconnecting: 'reconnecting',
	} as const;

	return {
		mockWsClient: {
			connect: vi.fn(),
			disconnect: vi.fn(),
			isConnected: vi.fn(() => false),
			send: vi.fn(),
			on: vi.fn((_handler: any) => vi.fn()),
			onStateChange: vi.fn((_handler: any) => vi.fn()),
		},
		mockDraftState: {
			session: null as any,
			draft: null as any,
			isAutoPickRunning: false,
			updatePickFromWS: vi.fn(),
			advancePick: vi.fn(),
			loadDraft: vi.fn(),
		},
		WebSocketState,
	};
});

// Capture handlers registered during construction
let capturedMessageHandler: ((msg: ServerMessage) => void) | null = null;
let _capturedStateHandler: ((state: string) => void) | null = null;

// Override on/onStateChange to capture handlers
mockWsClient.on.mockImplementation((handler: any) => {
	capturedMessageHandler = handler;
	return vi.fn();
});

mockWsClient.onStateChange.mockImplementation((handler: any) => {
	_capturedStateHandler = handler;
	return vi.fn();
});

// Mock modules before importing the module under test
vi.mock('$lib/api', () => ({
	wsClient: mockWsClient,
	WebSocketState,
}));

vi.mock('./draft.svelte', () => ({
	draftState: mockDraftState,
}));

vi.mock('$lib/utils/logger', () => ({
	logger: {
		error: vi.fn(),
		warn: vi.fn(),
		info: vi.fn(),
		debug: vi.fn(),
	},
}));

// Import after mocks are set up
import { WebSocketStateManager } from './websocket.svelte';

describe('WebSocketStateManager', () => {
	let manager: WebSocketStateManager;

	beforeEach(() => {
		vi.clearAllMocks();
		capturedMessageHandler = null;
		_capturedStateHandler = null;
		mockDraftState.session = null;
		mockDraftState.draft = null;
		mockDraftState.isAutoPickRunning = false;

		// Re-setup handler capture for each test
		mockWsClient.on.mockImplementation((handler: any) => {
			capturedMessageHandler = handler;
			return vi.fn();
		});
		mockWsClient.onStateChange.mockImplementation((handler: any) => {
			_capturedStateHandler = handler;
			return vi.fn();
		});

		manager = new WebSocketStateManager();
	});

	describe('connect', () => {
		it('should call wsClient.connect()', () => {
			manager.connect();
			expect(mockWsClient.connect).toHaveBeenCalledOnce();
		});
	});

	describe('disconnect', () => {
		it('should call wsClient.disconnect()', () => {
			manager.disconnect();
			expect(mockWsClient.disconnect).toHaveBeenCalledOnce();
		});
	});

	describe('subscribeToSession', () => {
		it('should send subscribe message when connected', () => {
			mockWsClient.isConnected.mockReturnValueOnce(true);
			manager.subscribeToSession('session-1');

			expect(mockWsClient.send).toHaveBeenCalledWith({
				type: 'subscribe',
				session_id: 'session-1',
			});
		});

		it('should not send when disconnected', () => {
			mockWsClient.isConnected.mockReturnValueOnce(false);
			manager.subscribeToSession('session-1');

			expect(mockWsClient.send).not.toHaveBeenCalled();
		});
	});

	describe('isConnected', () => {
		it('should return true when state is Connected', () => {
			manager.connectionState = WebSocketState.Connected as any;
			expect(manager.isConnected).toBe(true);
		});

		it('should return false otherwise', () => {
			manager.connectionState = WebSocketState.Disconnected as any;
			expect(manager.isConnected).toBe(false);
		});
	});

	describe('handleMessage pick_made', () => {
		it('should call draftState.updatePickFromWS and advancePick', () => {
			expect(capturedMessageHandler).not.toBeNull();

			capturedMessageHandler!({
				type: 'pick_made',
				session_id: 'session-1',
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
				round: 1,
				pick_number: 1,
				player_name: 'John Doe',
				team_name: 'Team A',
			});

			expect(mockDraftState.updatePickFromWS).toHaveBeenCalledWith({
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
			});
			expect(mockDraftState.advancePick).toHaveBeenCalled();
		});

		it('should not advance pick when auto-pick is running', () => {
			mockDraftState.isAutoPickRunning = true;

			capturedMessageHandler!({
				type: 'pick_made',
				session_id: 'session-1',
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
				round: 1,
				pick_number: 1,
				player_name: 'John Doe',
				team_name: 'Team A',
			});

			expect(mockDraftState.updatePickFromWS).toHaveBeenCalled();
			expect(mockDraftState.advancePick).not.toHaveBeenCalled();
		});
	});

	describe('handleMessage draft_status', () => {
		it('should update session status', () => {
			mockDraftState.session = {
				id: 'session-1',
				draft_id: 'draft-1',
				status: 'InProgress',
				current_pick_number: 1,
				time_per_pick_seconds: 300,
				auto_pick_enabled: false,
				chart_type: 'JimmyJohnson',
				controlled_team_ids: [],
			};

			capturedMessageHandler!({
				type: 'draft_status',
				session_id: 'session-1',
				status: 'Paused',
			});

			expect(mockDraftState.session.status).toBe('Paused');
		});
	});

	describe('handleMessage error', () => {
		it('should set error state', () => {
			capturedMessageHandler!({
				type: 'error',
				message: 'Something went wrong',
			});

			expect(manager.error).toBe('Something went wrong');
		});
	});
});
