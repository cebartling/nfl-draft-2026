import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { playersApi } from './players';
import * as client from './client';
import type { Player, ScoutingReport, CombineResults, Position } from '$lib/types';

describe('playersApi', () => {
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
		it('should fetch all players', async () => {
			const mockPlayers: Player[] = [
				{
					id: '1',
					first_name: 'John',
					last_name: 'Doe',
					position: 'QB',
					college: 'Alabama',
					height_inches: 76,
					weight_pounds: 220,
					draft_year: 2026,
					draft_eligible: true,
					projected_round: 1,
				},
				{
					id: '2',
					first_name: 'Jane',
					last_name: 'Smith',
					position: 'WR',
					college: 'Georgia',
					height_inches: 72,
					weight_pounds: 195,
					draft_year: 2026,
					draft_eligible: true,
					projected_round: 2,
				},
			];

			mockGet.mockResolvedValueOnce(mockPlayers);

			const result = await playersApi.list();

			expect(mockGet).toHaveBeenCalledWith('/players', expect.any(Object));
			expect(result).toEqual(mockPlayers);
		});

		it('should handle empty player list', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await playersApi.list();

			expect(result).toEqual([]);
		});
	});

	describe('get', () => {
		it('should fetch a single player by ID', async () => {
			const mockPlayer: Player = {
				id: '123',
				first_name: 'John',
				last_name: 'Doe',
				position: 'QB',
				college: 'Alabama',
				height_inches: 76,
				weight_pounds: 220,
				draft_year: 2026,
				draft_eligible: true,
				projected_round: 1,
			};

			mockGet.mockResolvedValueOnce(mockPlayer);

			const result = await playersApi.get('123');

			expect(mockGet).toHaveBeenCalledWith('/players/123', expect.any(Object));
			expect(result).toEqual(mockPlayer);
		});

		it('should throw error for non-existent player', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Not found', 404));

			await expect(playersApi.get('999')).rejects.toThrow('Not found');
		});
	});

	describe('create', () => {
		it('should create a new player', async () => {
			const newPlayer: Omit<Player, 'id'> = {
				first_name: 'John',
				last_name: 'Doe',
				position: 'QB',
				college: 'Alabama',
				height_inches: 76,
				weight_pounds: 220,
				draft_year: 2026,
				draft_eligible: true,
				projected_round: 1,
			};

			const createdPlayer: Player = {
				id: '123',
				...newPlayer,
			};

			mockPost.mockResolvedValueOnce(createdPlayer);

			const result = await playersApi.create(newPlayer);

			expect(mockPost).toHaveBeenCalledWith('/players', newPlayer, expect.any(Object));
			expect(result).toEqual(createdPlayer);
		});

		it('should throw error for invalid player data', async () => {
			const invalidPlayer = {
				first_name: 'John',
				// Missing required fields
			} as Omit<Player, 'id'>;

			mockPost.mockRejectedValueOnce(new client.ApiClientError('Bad Request', 400));

			await expect(playersApi.create(invalidPlayer)).rejects.toThrow('Bad Request');
		});
	});

	describe('getByPosition', () => {
		it('should fetch players by position', async () => {
			const mockQBs: Player[] = [
				{
					id: '1',
					first_name: 'John',
					last_name: 'Doe',
					position: 'QB',
					college: 'Alabama',
					height_inches: 76,
					weight_pounds: 220,
					draft_year: 2026,
					draft_eligible: true,
					projected_round: 1,
				},
			];

			mockGet.mockResolvedValueOnce(mockQBs);

			const result = await playersApi.getByPosition('QB');

			expect(mockGet).toHaveBeenCalledWith('/players/position/QB', expect.any(Object));
			expect(result).toEqual(mockQBs);
		});

		it('should handle position with no players', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await playersApi.getByPosition('P');

			expect(result).toEqual([]);
		});

		it('should work with all valid positions', async () => {
			const positions: Position[] = [
				'QB',
				'RB',
				'WR',
				'TE',
				'OT',
				'OG',
				'C',
				'DE',
				'DT',
				'LB',
				'CB',
				'S',
				'K',
				'P',
			];

			for (const position of positions) {
				mockGet.mockResolvedValueOnce([]);
				await playersApi.getByPosition(position);
				expect(mockGet).toHaveBeenCalledWith(`/players/position/${position}`, expect.any(Object));
			}
		});
	});

	describe('getScoutingReports', () => {
		it('should fetch scouting reports for a player', async () => {
			const playerId = '123';
			const mockReports: ScoutingReport[] = [
				{
					id: '1',
					player_id: playerId,
					team_id: '456',
					grade: 85,
					notes: 'Great arm talent',
					strengths: 'Strong arm, good accuracy',
					weaknesses: 'Needs to improve footwork',
					created_at: '2026-01-01T00:00:00Z',
					updated_at: '2026-01-01T00:00:00Z',
				},
				{
					id: '2',
					player_id: playerId,
					team_id: '789',
					grade: 80,
					notes: 'Solid prospect',
					created_at: '2026-01-02T00:00:00Z',
					updated_at: '2026-01-02T00:00:00Z',
				},
			];

			mockGet.mockResolvedValueOnce(mockReports);

			const result = await playersApi.getScoutingReports(playerId);

			expect(mockGet).toHaveBeenCalledWith(
				`/players/${playerId}/scouting-reports`,
				expect.any(Object)
			);
			expect(result).toEqual(mockReports);
		});

		it('should handle player with no scouting reports', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await playersApi.getScoutingReports('123');

			expect(result).toEqual([]);
		});
	});

	describe('createScoutingReport', () => {
		it('should create a new scouting report', async () => {
			const newReport: Omit<ScoutingReport, 'id' | 'created_at' | 'updated_at'> = {
				player_id: '123',
				team_id: '456',
				grade: 85,
				notes: 'Great arm talent',
				strengths: 'Strong arm, good accuracy',
				weaknesses: 'Needs to improve footwork',
			};

			const createdReport: ScoutingReport = {
				id: '789',
				...newReport,
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			};

			mockPost.mockResolvedValueOnce(createdReport);

			const result = await playersApi.createScoutingReport(newReport);

			expect(mockPost).toHaveBeenCalledWith('/scouting-reports', newReport, expect.any(Object));
			expect(result).toEqual(createdReport);
		});

		it('should throw error for invalid report data', async () => {
			const invalidReport = {
				player_id: '123',
				team_id: '456',
				// Missing grade
			} as Omit<ScoutingReport, 'id' | 'created_at' | 'updated_at'>;

			mockPost.mockRejectedValueOnce(new client.ApiClientError('Bad Request', 400));

			await expect(playersApi.createScoutingReport(invalidReport)).rejects.toThrow('Bad Request');
		});
	});

	describe('getCombineResults', () => {
		it('should fetch combine results for a player', async () => {
			const playerId = '123';
			const mockResults: CombineResults = {
				id: '456',
				player_id: playerId,
				forty_yard_dash: 4.5,
				bench_press: 20,
				vertical_jump: 36,
				broad_jump: 120,
				three_cone_drill: 7.0,
				shuttle_run: 4.2,
			};

			mockGet.mockResolvedValueOnce(mockResults);

			const result = await playersApi.getCombineResults(playerId);

			expect(mockGet).toHaveBeenCalledWith(
				`/players/${playerId}/combine-results`,
				expect.any(Object)
			);
			expect(result).toEqual(mockResults);
		});

		it('should return null for 404 (no combine results)', async () => {
			const mockError = new client.ApiClientError('Not found', 404);
			mockGet.mockRejectedValueOnce(mockError);

			const result = await playersApi.getCombineResults('123');

			expect(result).toBeNull();
		});

		it('should throw error for other error statuses', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Internal Server Error', 500));

			await expect(playersApi.getCombineResults('123')).rejects.toThrow('Internal Server Error');
		});

		it('should handle partial combine results', async () => {
			const playerId = '123';
			const mockResults: CombineResults = {
				id: '456',
				player_id: playerId,
				forty_yard_dash: 4.5,
				// Other fields undefined
			};

			mockGet.mockResolvedValueOnce(mockResults);

			const result = await playersApi.getCombineResults(playerId);

			expect(result).toEqual(mockResults);
			expect(result?.forty_yard_dash).toBe(4.5);
			expect(result?.bench_press).toBeUndefined();
		});
	});
});
