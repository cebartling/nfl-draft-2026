import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { sessionsApi } from './sessions';
import * as client from './client';
import type { DraftSession, DraftEvent, DraftPick } from '$lib/types';

describe('sessionsApi', () => {
	let mockGet: ReturnType<typeof vi.fn>;
	let mockPost: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockGet = vi.fn();
		mockPost = vi.fn();

		vi.spyOn(client.apiClient, 'get').mockImplementation(mockGet as any);
		vi.spyOn(client.apiClient, 'post').mockImplementation(mockPost as any);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	function makeSession(overrides: Partial<DraftSession> = {}): DraftSession {
		return {
			id: 'session-1',
			draft_id: 'draft-1',
			status: 'InProgress',
			current_pick_number: 1,
			time_per_pick_seconds: 300,
			auto_pick_enabled: false,
			chart_type: 'JimmyJohnson',
			controlled_team_ids: [],
			...overrides,
		};
	}

	describe('create', () => {
		it('should send POST /sessions with params', async () => {
			const params = {
				draft_id: 'draft-1',
				time_per_pick_seconds: 300,
				auto_pick_enabled: true,
				chart_type: 'JimmyJohnson' as const,
				controlled_team_ids: ['team-1'],
			};
			const mockSession = makeSession({
				auto_pick_enabled: true,
				controlled_team_ids: ['team-1'],
			});

			mockPost.mockResolvedValueOnce(mockSession);

			const result = await sessionsApi.create(params);

			expect(mockPost).toHaveBeenCalledWith('/sessions', params, expect.any(Object));
			expect(result).toEqual(mockSession);
		});

		it('should propagate errors', async () => {
			mockPost.mockRejectedValueOnce(new Error('Draft not found'));

			await expect(
				sessionsApi.create({
					draft_id: 'invalid',
					time_per_pick_seconds: 300,
					auto_pick_enabled: false,
					chart_type: 'JimmyJohnson',
				})
			).rejects.toThrow('Draft not found');
		});
	});

	describe('get', () => {
		it('should send GET /sessions/{id}', async () => {
			const mockSession = makeSession();
			mockGet.mockResolvedValueOnce(mockSession);

			const result = await sessionsApi.get('session-1');

			expect(mockGet).toHaveBeenCalledWith('/sessions/session-1', expect.any(Object));
			expect(result).toEqual(mockSession);
		});

		it('should propagate errors for non-existent session', async () => {
			mockGet.mockRejectedValueOnce(new Error('Session not found'));

			await expect(sessionsApi.get('invalid-id')).rejects.toThrow('Session not found');
		});
	});

	describe('getByDraftId', () => {
		it('should send GET /drafts/{draftId}/session', async () => {
			const mockSession = makeSession();
			mockGet.mockResolvedValueOnce(mockSession);

			const result = await sessionsApi.getByDraftId('draft-1');

			expect(mockGet).toHaveBeenCalledWith('/drafts/draft-1/session', expect.any(Object));
			expect(result).toEqual(mockSession);
		});
	});

	describe('start', () => {
		it('should send POST /sessions/{id}/start', async () => {
			const mockSession = makeSession({ status: 'InProgress' });
			mockPost.mockResolvedValueOnce(mockSession);

			const result = await sessionsApi.start('session-1');

			expect(mockPost).toHaveBeenCalledWith(
				'/sessions/session-1/start',
				{},
				expect.any(Object)
			);
			expect(result.status).toBe('InProgress');
		});

		it('should propagate errors', async () => {
			mockPost.mockRejectedValueOnce(new Error('Session already started'));

			await expect(sessionsApi.start('session-1')).rejects.toThrow('Session already started');
		});
	});

	describe('pause', () => {
		it('should send POST /sessions/{id}/pause', async () => {
			const mockSession = makeSession({ status: 'Paused' });
			mockPost.mockResolvedValueOnce(mockSession);

			const result = await sessionsApi.pause('session-1');

			expect(mockPost).toHaveBeenCalledWith(
				'/sessions/session-1/pause',
				{},
				expect.any(Object)
			);
			expect(result.status).toBe('Paused');
		});
	});

	describe('getEvents', () => {
		it('should send GET /sessions/{id}/events', async () => {
			const mockEvents: DraftEvent[] = [
				{
					id: 'event-1',
					session_id: 'session-1',
					event_type: 'pick_made',
					event_data: { player_id: 'p1' },
					created_at: '2026-04-25T20:00:00Z',
				},
			];
			mockGet.mockResolvedValueOnce(mockEvents);

			const result = await sessionsApi.getEvents('session-1');

			expect(mockGet).toHaveBeenCalledWith(
				'/sessions/session-1/events',
				expect.any(Object)
			);
			expect(result).toEqual(mockEvents);
		});
	});

	describe('autoPickRun', () => {
		it('should send POST /sessions/{id}/auto-pick-run', async () => {
			const mockPick: DraftPick = {
				id: 'pick-1',
				draft_id: 'draft-1',
				round: 1,
				pick_number: 1,
				overall_pick: 1,
				team_id: 'team-1',
				is_compensatory: false,
				is_traded: false,
				player_id: 'player-1',
				picked_at: '2026-04-25T20:00:00Z',
			};
			const mockResponse = {
				session: makeSession({ current_pick_number: 2 }),
				picks_made: [mockPick],
			};
			mockPost.mockResolvedValueOnce(mockResponse);

			const result = await sessionsApi.autoPickRun('session-1');

			expect(mockPost).toHaveBeenCalledWith(
				'/sessions/session-1/auto-pick-run',
				{},
				expect.any(Object)
			);
			expect(result.picks_made).toHaveLength(1);
			expect(result.session.current_pick_number).toBe(2);
		});

		it('should handle empty picks_made array', async () => {
			const mockResponse = {
				session: makeSession(),
				picks_made: [],
			};
			mockPost.mockResolvedValueOnce(mockResponse);

			const result = await sessionsApi.autoPickRun('session-1');

			expect(result.picks_made).toEqual([]);
		});
	});

	describe('advancePick', () => {
		it('should send POST /sessions/{id}/advance-pick', async () => {
			const mockSession = makeSession({ current_pick_number: 2 });
			mockPost.mockResolvedValueOnce(mockSession);

			const result = await sessionsApi.advancePick('session-1');

			expect(mockPost).toHaveBeenCalledWith(
				'/sessions/session-1/advance-pick',
				{},
				expect.any(Object)
			);
			expect(result.current_pick_number).toBe(2);
		});
	});
});
