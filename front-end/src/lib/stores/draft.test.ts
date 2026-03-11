import { describe, it, expect, vi, beforeEach } from 'vitest';
import { DraftState } from './draft.svelte';
import type { DraftSession, DraftPick } from '$lib/types';

// Mock the API modules
vi.mock('$lib/api', () => ({
	draftsApi: {
		get: vi.fn(),
		getPicks: vi.fn(),
	},
	sessionsApi: {
		get: vi.fn(),
		start: vi.fn(),
		pause: vi.fn(),
	},
}));

vi.mock('$lib/utils/logger', () => ({
	logger: {
		error: vi.fn(),
		warn: vi.fn(),
		info: vi.fn(),
		debug: vi.fn(),
	},
}));

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

function makePick(overrides: Partial<DraftPick> = {}): DraftPick {
	return {
		id: 'pick-1',
		draft_id: 'draft-1',
		round: 1,
		pick_number: 1,
		overall_pick: 1,
		team_id: 'team-1',
		is_compensatory: false,
		is_traded: false,
		...overrides,
	};
}

describe('DraftState', () => {
	let state: DraftState;

	beforeEach(() => {
		state = new DraftState();
	});

	describe('controlledTeamIds', () => {
		it('should return empty array when no session', () => {
			expect(state.controlledTeamIds).toEqual([]);
		});

		it('should return session controlled_team_ids', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1', 'team-2'],
			});
			expect(state.controlledTeamIds).toEqual(['team-1', 'team-2']);
		});

		it('should return empty array when session has no controlled teams', () => {
			state.session = makeSession({ controlled_team_ids: [] });
			expect(state.controlledTeamIds).toEqual([]);
		});
	});

	describe('hasControlledTeams', () => {
		it('should return false when no session', () => {
			expect(state.hasControlledTeams).toBe(false);
		});

		it('should return false when controlled_team_ids is empty', () => {
			state.session = makeSession({ controlled_team_ids: [] });
			expect(state.hasControlledTeams).toBe(false);
		});

		it('should return true when controlled_team_ids has entries', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1'],
			});
			expect(state.hasControlledTeams).toBe(true);
		});
	});

	describe('isTeamControlled', () => {
		it('should return false when no session', () => {
			expect(state.isTeamControlled('team-1')).toBe(false);
		});

		it('should return true for controlled team', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1', 'team-2'],
			});
			expect(state.isTeamControlled('team-1')).toBe(true);
			expect(state.isTeamControlled('team-2')).toBe(true);
		});

		it('should return false for non-controlled team', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1'],
			});
			expect(state.isTeamControlled('team-3')).toBe(false);
		});
	});

	describe('isCurrentPickUserControlled', () => {
		it('should return false when no current pick', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1'],
				current_pick_number: 1,
			});
			// No picks loaded
			expect(state.isCurrentPickUserControlled).toBe(false);
		});

		it('should return true when current pick team is controlled', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1'],
				current_pick_number: 1,
			});
			state.picks = [makePick({ overall_pick: 1, team_id: 'team-1' })];
			expect(state.isCurrentPickUserControlled).toBe(true);
		});

		it('should return false when current pick team is not controlled', () => {
			state.session = makeSession({
				controlled_team_ids: ['team-1'],
				current_pick_number: 1,
			});
			state.picks = [makePick({ overall_pick: 1, team_id: 'team-2' })];
			expect(state.isCurrentPickUserControlled).toBe(false);
		});

		it('should return false when no teams are controlled', () => {
			state.session = makeSession({
				controlled_team_ids: [],
				current_pick_number: 1,
			});
			state.picks = [makePick({ overall_pick: 1, team_id: 'team-1' })];
			expect(state.isCurrentPickUserControlled).toBe(false);
		});
	});

	describe('currentPickNumber', () => {
		it('should return 1 when no session', () => {
			expect(state.currentPickNumber).toBe(1);
		});

		it('should return session current_pick_number', () => {
			state.session = makeSession({ current_pick_number: 5 });
			expect(state.currentPickNumber).toBe(5);
		});
	});

	describe('currentPick', () => {
		it('should return null when no picks', () => {
			state.session = makeSession({ current_pick_number: 1 });
			expect(state.currentPick).toBeNull();
		});

		it('should return pick matching current pick number', () => {
			state.session = makeSession({ current_pick_number: 2 });
			state.picks = [
				makePick({ id: 'pick-1', overall_pick: 1 }),
				makePick({ id: 'pick-2', overall_pick: 2 }),
				makePick({ id: 'pick-3', overall_pick: 3 }),
			];
			expect(state.currentPick?.id).toBe('pick-2');
		});

		it('should return null when no pick matches current number', () => {
			state.session = makeSession({ current_pick_number: 99 });
			state.picks = [makePick({ overall_pick: 1 })];
			expect(state.currentPick).toBeNull();
		});
	});

	describe('updatePickFromWS', () => {
		it('should update pick and advance current_pick_number', () => {
			state.session = makeSession({ current_pick_number: 1 });
			state.picks = [
				makePick({ id: 'pick-1', overall_pick: 1, team_id: 'team-1' }),
				makePick({ id: 'pick-2', overall_pick: 2, team_id: 'team-2' }),
			];

			state.updatePickFromWS({
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
			});

			expect(state.picks[0].player_id).toBe('player-1');
			expect(state.picks[0].picked_at).toBeDefined();
			expect(state.session?.current_pick_number).toBe(2);
		});

		it('should advance pick number based on actual pick position', () => {
			// Simulates resume: session says pick 11, WS confirms pick 11 was made
			state.session = makeSession({ current_pick_number: 11 });
			state.picks = [
				makePick({ id: 'pick-11', overall_pick: 11, team_id: 'team-3' }),
				makePick({ id: 'pick-12', overall_pick: 12, team_id: 'team-4' }),
			];

			state.updatePickFromWS({
				pick_id: 'pick-11',
				player_id: 'player-11',
				team_id: 'team-3',
			});

			expect(state.session?.current_pick_number).toBe(12);
		});

		it('should not go backwards if messages arrive out of order', () => {
			state.session = makeSession({ current_pick_number: 15 });
			state.picks = [makePick({ id: 'pick-10', overall_pick: 10, team_id: 'team-1' })];

			// Late message for an earlier pick
			state.updatePickFromWS({
				pick_id: 'pick-10',
				player_id: 'player-10',
				team_id: 'team-1',
			});

			// Should NOT go backwards from 15 to 11
			expect(state.session?.current_pick_number).toBe(15);
		});

		it('should do nothing when pick_id not found', () => {
			state.session = makeSession({ current_pick_number: 1 });
			state.picks = [makePick({ id: 'pick-1', overall_pick: 1 })];

			state.updatePickFromWS({
				pick_id: 'nonexistent',
				player_id: 'player-1',
				team_id: 'team-1',
			});

			expect(state.session?.current_pick_number).toBe(1);
			expect(state.picks[0].player_id).toBeUndefined();
		});

		it('should work without a session', () => {
			state.picks = [makePick({ id: 'pick-1', overall_pick: 1 })];

			state.updatePickFromWS({
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
			});

			// Pick should still be updated even without session
			expect(state.picks[0].player_id).toBe('player-1');
		});
	});

	describe('completedPicks', () => {
		it('should return picks with player_id', () => {
			state.picks = [
				makePick({ id: 'p1', player_id: 'player-1' }),
				makePick({ id: 'p2', player_id: undefined }),
				makePick({ id: 'p3', player_id: 'player-2' }),
			];
			expect(state.completedPicks).toHaveLength(2);
			expect(state.completedPicks[0].id).toBe('p1');
			expect(state.completedPicks[1].id).toBe('p3');
		});
	});

	describe('availablePicks', () => {
		it('should return picks without player_id', () => {
			state.picks = [
				makePick({ id: 'p1', player_id: 'player-1' }),
				makePick({ id: 'p2', player_id: undefined }),
				makePick({ id: 'p3', player_id: null }),
			];
			expect(state.availablePicks).toHaveLength(2);
		});
	});

	describe('addPickNotification', () => {
		it('should add a notification to the list', () => {
			state.addPickNotification({
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
				player_name: 'John Doe',
				team_name: 'Team A',
				round: 1,
				pick_number: 1,
			});
			expect(state.pickNotifications).toHaveLength(1);
			expect(state.pickNotifications[0].player_name).toBe('John Doe');
		});

		it('should accumulate multiple notifications', () => {
			state.addPickNotification({
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
				player_name: 'John Doe',
				team_name: 'Team A',
				round: 1,
				pick_number: 1,
			});
			state.addPickNotification({
				pick_id: 'pick-2',
				player_id: 'player-2',
				team_id: 'team-2',
				player_name: 'Jane Smith',
				team_name: 'Team B',
				round: 1,
				pick_number: 2,
			});
			expect(state.pickNotifications).toHaveLength(2);
			expect(state.pickNotifications[1].player_name).toBe('Jane Smith');
		});
	});

	describe('reset', () => {
		it('should clear all state including notifications', () => {
			state.session = makeSession();
			state.picks = [makePick()];
			state.isLoading = true;
			state.error = 'some error';
			state.isAutoPickRunning = true;
			state.addPickNotification({
				pick_id: 'pick-1',
				player_id: 'player-1',
				team_id: 'team-1',
				player_name: 'John Doe',
				team_name: 'Team A',
				round: 1,
				pick_number: 1,
			});

			state.reset();

			expect(state.draft).toBeNull();
			expect(state.session).toBeNull();
			expect(state.picks).toEqual([]);
			expect(state.isLoading).toBe(false);
			expect(state.error).toBeNull();
			expect(state.isAutoPickRunning).toBe(false);
			expect(state.pickNotifications).toEqual([]);
		});
	});
});
