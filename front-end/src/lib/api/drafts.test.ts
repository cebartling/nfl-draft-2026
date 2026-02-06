import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { draftsApi } from './drafts';
import * as client from './client';
import type { Draft, DraftPick, Player } from '$lib/types';

describe('draftsApi', () => {
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

	describe('list', () => {
		it('should fetch all drafts', async () => {
			const mockDrafts: Draft[] = [
				{
					id: '1',
					year: 2026,
					status: 'NotStarted',
					rounds: 7,
					picks_per_round: 32,
				},
				{
					id: '2',
					year: 2025,
					status: 'Completed',
					rounds: 7,
					picks_per_round: 32,
				},
			];

			mockGet.mockResolvedValueOnce(mockDrafts);

			const result = await draftsApi.list();

			expect(mockGet).toHaveBeenCalledWith('/drafts', expect.any(Object));
			expect(result).toEqual(mockDrafts);
		});

		it('should handle empty draft list', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await draftsApi.list();

			expect(result).toEqual([]);
		});
	});

	describe('get', () => {
		it('should fetch a single draft by ID', async () => {
			const mockDraft: Draft = {
				id: '123',
				year: 2026,
				status: 'NotStarted',
				rounds: 7,
				picks_per_round: 32,
			};

			mockGet.mockResolvedValueOnce(mockDraft);

			const result = await draftsApi.get('123');

			expect(mockGet).toHaveBeenCalledWith('/drafts/123', expect.any(Object));
			expect(result).toEqual(mockDraft);
		});

		it('should propagate errors when draft not found', async () => {
			mockGet.mockRejectedValueOnce(new Error('Draft not found'));

			await expect(draftsApi.get('invalid-id')).rejects.toThrow('Draft not found');
		});
	});

	describe('create', () => {
		it('should create a new draft', async () => {
			const newDraft = {
				year: 2027,
				rounds: 7,
				picks_per_round: 32,
			};

			const mockCreatedDraft: Draft = {
				id: 'new-draft-id',
				year: 2027,
				status: 'NotStarted',
				rounds: 7,
				picks_per_round: 32,
			};

			mockPost.mockResolvedValueOnce(mockCreatedDraft);

			const result = await draftsApi.create(newDraft);

			expect(mockPost).toHaveBeenCalledWith('/drafts', newDraft, expect.any(Object));
			expect(result).toEqual(mockCreatedDraft);
		});

		it('should propagate errors when draft already exists', async () => {
			mockPost.mockRejectedValueOnce(new Error('Draft for year 2026 already exists'));

			await expect(
				draftsApi.create({ year: 2026, rounds: 7, picks_per_round: 32 })
			).rejects.toThrow('Draft for year 2026 already exists');
		});
	});

	describe('getPicks', () => {
		it('should fetch all picks for a draft', async () => {
			const mockPicks: DraftPick[] = [
				{
					id: 'pick-1',
					draft_id: 'draft-1',
					round: 1,
					pick_number: 1,
					overall_pick: 1,
					team_id: 'team-1',
					player_id: null,
					picked_at: null,
				},
				{
					id: 'pick-2',
					draft_id: 'draft-1',
					round: 1,
					pick_number: 2,
					overall_pick: 2,
					team_id: 'team-2',
					player_id: null,
					picked_at: null,
				},
			];

			mockGet.mockResolvedValueOnce(mockPicks);

			const result = await draftsApi.getPicks('draft-1');

			expect(mockGet).toHaveBeenCalledWith('/drafts/draft-1/picks', expect.any(Object));
			expect(result).toEqual(mockPicks);
		});

		it('should return empty array when no picks', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await draftsApi.getPicks('draft-1');

			expect(result).toEqual([]);
		});
	});

	describe('makePick', () => {
		it('should make a pick in the draft', async () => {
			const mockPick: DraftPick = {
				id: 'pick-1',
				draft_id: 'draft-1',
				round: 1,
				pick_number: 1,
				overall_pick: 1,
				team_id: 'team-1',
				player_id: 'player-1',
				picked_at: '2026-04-25T20:00:00Z',
			};

			mockPost.mockResolvedValueOnce(mockPick);

			const result = await draftsApi.makePick('draft-1', 'pick-1', 'player-1');

			expect(mockPost).toHaveBeenCalledWith(
				'/drafts/draft-1/picks/pick-1',
				{ player_id: 'player-1' },
				expect.any(Object)
			);
			expect(result).toEqual(mockPick);
		});

		it('should propagate errors when player already drafted', async () => {
			mockPost.mockRejectedValueOnce(new Error('Player already drafted'));

			await expect(draftsApi.makePick('draft-1', 'pick-1', 'player-1')).rejects.toThrow(
				'Player already drafted'
			);
		});
	});

	describe('getAvailablePlayers', () => {
		it('should fetch available players for a draft', async () => {
			const mockPlayers: Player[] = [
				{
					id: 'player-1',
					first_name: 'John',
					last_name: 'Quarterback',
					position: 'QB',
					college: 'Alabama',
					height_inches: 76,
					weight_pounds: 220,
					draft_year: 2026,
					draft_eligible: true,
				},
				{
					id: 'player-2',
					first_name: 'Mike',
					last_name: 'Receiver',
					position: 'WR',
					college: 'Georgia',
					height_inches: 73,
					weight_pounds: 200,
					draft_year: 2026,
					draft_eligible: true,
				},
			];

			mockGet.mockResolvedValueOnce(mockPlayers);

			const result = await draftsApi.getAvailablePlayers('draft-1');

			expect(mockGet).toHaveBeenCalledWith('/drafts/draft-1/available-players', expect.any(Object));
			expect(result).toEqual(mockPlayers);
		});
	});

	describe('initializePicks', () => {
		it('should initialize draft picks', async () => {
			const mockPicks: DraftPick[] = [
				{
					id: 'pick-1',
					draft_id: 'draft-1',
					round: 1,
					pick_number: 1,
					overall_pick: 1,
					team_id: 'team-1',
					player_id: null,
					picked_at: null,
				},
				{
					id: 'pick-2',
					draft_id: 'draft-1',
					round: 1,
					pick_number: 2,
					overall_pick: 2,
					team_id: 'team-2',
					player_id: null,
					picked_at: null,
				},
			];

			mockPost.mockResolvedValueOnce(mockPicks);

			const result = await draftsApi.initializePicks('draft-1');

			expect(mockPost).toHaveBeenCalledWith('/drafts/draft-1/initialize', {}, expect.any(Object));
			expect(result).toEqual(mockPicks);
		});

		it('should handle large number of picks', async () => {
			// Generate 224 picks (7 rounds * 32 teams)
			const mockPicks: DraftPick[] = [];
			for (let round = 1; round <= 7; round++) {
				for (let pick = 1; pick <= 32; pick++) {
					mockPicks.push({
						id: `pick-${(round - 1) * 32 + pick}`,
						draft_id: 'draft-1',
						round,
						pick_number: pick,
						overall_pick: (round - 1) * 32 + pick,
						team_id: `team-${pick}`,
						player_id: null,
						picked_at: null,
					});
				}
			}

			mockPost.mockResolvedValueOnce(mockPicks);

			const result = await draftsApi.initializePicks('draft-1');

			expect(result).toHaveLength(224);
			expect(result[0].round).toBe(1);
			expect(result[223].round).toBe(7);
		});

		it('should propagate errors when draft not found', async () => {
			mockPost.mockRejectedValueOnce(new Error('Draft not found'));

			await expect(draftsApi.initializePicks('invalid-id')).rejects.toThrow('Draft not found');
		});

		it('should propagate errors when picks already initialized', async () => {
			mockPost.mockRejectedValueOnce(new Error('Picks already initialized'));

			await expect(draftsApi.initializePicks('draft-1')).rejects.toThrow(
				'Picks already initialized'
			);
		});
	});
});
