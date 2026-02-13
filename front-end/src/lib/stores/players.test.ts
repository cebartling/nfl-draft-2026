import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PlayersState } from './players.svelte';
import type { Player, Position } from '$lib/types';

// Mock the API modules
vi.mock('$lib/api', () => ({
	playersApi: {
		list: vi.fn(),
		get: vi.fn(),
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

function makePlayer(overrides: Partial<Player> = {}): Player {
	return {
		id: 'player-1',
		first_name: 'John',
		last_name: 'Doe',
		position: 'QB' as Position,
		draft_year: 2026,
		draft_eligible: true,
		...overrides,
	};
}

describe('PlayersState', () => {
	let state: PlayersState;

	beforeEach(() => {
		vi.clearAllMocks();
		state = new PlayersState();
	});

	describe('initial state', () => {
		it('should have empty allPlayers', () => {
			expect(state.allPlayers).toEqual([]);
		});

		it('should have null selectedPlayer', () => {
			expect(state.selectedPlayer).toBeNull();
		});

		it('should have no error', () => {
			expect(state.error).toBeNull();
		});

		it('should not be loading', () => {
			expect(state.isLoading).toBe(false);
		});
	});

	describe('byPosition', () => {
		it('should group players by position', () => {
			state.allPlayers = [
				makePlayer({ id: 'p1', position: 'QB' }),
				makePlayer({ id: 'p2', position: 'WR' }),
				makePlayer({ id: 'p3', position: 'QB' }),
			];

			const grouped = state.byPosition;
			expect(grouped.get('QB' as Position)).toHaveLength(2);
			expect(grouped.get('WR' as Position)).toHaveLength(1);
		});

		it('should return empty map when no players', () => {
			const grouped = state.byPosition;
			expect(grouped.size).toBe(0);
		});
	});

	describe('getByPosition', () => {
		it('should filter by specific position', () => {
			state.allPlayers = [
				makePlayer({ id: 'p1', position: 'QB' }),
				makePlayer({ id: 'p2', position: 'WR' }),
				makePlayer({ id: 'p3', position: 'QB' }),
			];

			const qbs = state.getByPosition('QB' as Position);
			expect(qbs).toHaveLength(2);
			expect(qbs.every((p) => p.position === 'QB')).toBe(true);
		});
	});

	describe('draftEligible', () => {
		it('should filter to draft_eligible=true only', () => {
			state.allPlayers = [
				makePlayer({ id: 'p1', draft_eligible: true }),
				makePlayer({ id: 'p2', draft_eligible: false }),
				makePlayer({ id: 'p3', draft_eligible: true }),
			];

			expect(state.draftEligible).toHaveLength(2);
		});
	});

	describe('loadAll', () => {
		it('should set allPlayers from API on success', async () => {
			const { playersApi } = await import('$lib/api');
			const mockPlayers = [makePlayer({ id: 'p1' }), makePlayer({ id: 'p2' })];
			vi.mocked(playersApi.list).mockResolvedValueOnce(mockPlayers);

			await state.loadAll();

			expect(state.allPlayers).toEqual(mockPlayers);
			expect(state.error).toBeNull();
			expect(state.isLoading).toBe(false);
		});

		it('should set error message on failure', async () => {
			const { playersApi } = await import('$lib/api');
			vi.mocked(playersApi.list).mockRejectedValueOnce(new Error('Network error'));

			await state.loadAll();

			expect(state.error).toBe('Network error');
			expect(state.allPlayers).toEqual([]);
			expect(state.isLoading).toBe(false);
		});

		it('should set isLoading during fetch', async () => {
			const { playersApi } = await import('$lib/api');
			let capturedLoading = false;
			vi.mocked(playersApi.list).mockImplementationOnce(async () => {
				capturedLoading = state.isLoading;
				return [];
			});

			await state.loadAll();

			expect(capturedLoading).toBe(true);
			expect(state.isLoading).toBe(false);
		});
	});

	describe('loadPlayer', () => {
		it('should set selectedPlayer and update allPlayers on success', async () => {
			const { playersApi } = await import('$lib/api');
			const mockPlayer = makePlayer({ id: 'p1', first_name: 'Updated' });
			vi.mocked(playersApi.get).mockResolvedValueOnce(mockPlayer);

			// Start with an existing player in allPlayers
			state.allPlayers = [makePlayer({ id: 'p1', first_name: 'Old' })];

			await state.loadPlayer('p1');

			expect(state.selectedPlayer).toEqual(mockPlayer);
			expect(state.allPlayers[0].first_name).toBe('Updated');
			expect(state.isLoading).toBe(false);
		});

		it('should add to allPlayers if not present', async () => {
			const { playersApi } = await import('$lib/api');
			const mockPlayer = makePlayer({ id: 'p2' });
			vi.mocked(playersApi.get).mockResolvedValueOnce(mockPlayer);

			state.allPlayers = [makePlayer({ id: 'p1' })];

			await state.loadPlayer('p2');

			expect(state.allPlayers).toHaveLength(2);
		});

		it('should set error on failure', async () => {
			const { playersApi } = await import('$lib/api');
			vi.mocked(playersApi.get).mockRejectedValueOnce(new Error('Player not found'));

			await state.loadPlayer('invalid');

			expect(state.error).toBe('Player not found');
		});
	});

	describe('filterAvailable', () => {
		it('should exclude player IDs in the picked set', () => {
			state.allPlayers = [
				makePlayer({ id: 'p1' }),
				makePlayer({ id: 'p2' }),
				makePlayer({ id: 'p3' }),
			];

			const available = state.filterAvailable(new Set(['p1', 'p3']));
			expect(available).toHaveLength(1);
			expect(available[0].id).toBe('p2');
		});
	});

	describe('searchByName', () => {
		it('should perform case-insensitive match on first_name', () => {
			state.allPlayers = [
				makePlayer({ id: 'p1', first_name: 'John', last_name: 'Smith' }),
				makePlayer({ id: 'p2', first_name: 'Jane', last_name: 'Doe' }),
			];

			const results = state.searchByName('john');
			expect(results).toHaveLength(1);
			expect(results[0].id).toBe('p1');
		});

		it('should perform case-insensitive match on last_name', () => {
			state.allPlayers = [
				makePlayer({ id: 'p1', first_name: 'John', last_name: 'Smith' }),
				makePlayer({ id: 'p2', first_name: 'Jane', last_name: 'Doe' }),
			];

			const results = state.searchByName('DOE');
			expect(results).toHaveLength(1);
			expect(results[0].id).toBe('p2');
		});
	});

	describe('getPlayerFullName', () => {
		it('should return "first_name last_name"', () => {
			const player = makePlayer({ first_name: 'John', last_name: 'Doe' });
			expect(state.getPlayerFullName(player)).toBe('John Doe');
		});
	});

	describe('reset', () => {
		it('should clear all state', () => {
			state.allPlayers = [makePlayer()];
			state.selectedPlayer = makePlayer();
			state.isLoading = true;
			state.error = 'some error';

			state.reset();

			expect(state.allPlayers).toEqual([]);
			expect(state.selectedPlayer).toBeNull();
			expect(state.isLoading).toBe(false);
			expect(state.error).toBeNull();
		});
	});
});
