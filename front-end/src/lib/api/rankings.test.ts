import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { rankingsApi } from './rankings';
import * as client from './client';
import type { RankingSource, PlayerRanking, SourceRanking, AllRankingEntry } from '$lib/types';

describe('rankingsApi', () => {
	let mockGet: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockGet = vi.fn();
		vi.spyOn(client.apiClient, 'get').mockImplementation(mockGet as any);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('listSources', () => {
		it('should fetch all ranking sources', async () => {
			const mockSources: RankingSource[] = [
				{ id: 'src-1', name: 'Tankathon', url: 'https://tankathon.com', description: null },
				{ id: 'src-2', name: 'Walter Football', url: 'https://walterfootball.com' },
			];

			mockGet.mockResolvedValueOnce(mockSources);

			const result = await rankingsApi.listSources();

			expect(mockGet).toHaveBeenCalledWith('/ranking-sources', expect.any(Object));
			expect(result).toEqual(mockSources);
		});

		it('should handle empty sources list', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await rankingsApi.listSources();

			expect(result).toEqual([]);
		});
	});

	describe('getPlayerRankings', () => {
		it('should fetch rankings for a player', async () => {
			const playerId = 'player-123';
			const mockRankings: PlayerRanking[] = [
				{ source_name: 'Tankathon', source_id: 'src-1', rank: 5, scraped_at: '2026-02-01' },
				{
					source_name: 'Walter Football',
					source_id: 'src-2',
					rank: 8,
					scraped_at: '2026-02-01',
				},
			];

			mockGet.mockResolvedValueOnce(mockRankings);

			const result = await rankingsApi.getPlayerRankings(playerId);

			expect(mockGet).toHaveBeenCalledWith(
				`/players/${playerId}/rankings`,
				expect.any(Object),
			);
			expect(result).toEqual(mockRankings);
		});

		it('should handle player with no rankings', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await rankingsApi.getPlayerRankings('player-999');

			expect(result).toEqual([]);
		});
	});

	describe('getSourceRankings', () => {
		it('should fetch big board for a source', async () => {
			const sourceId = 'src-1';
			const mockRankings: SourceRanking[] = [
				{ player_id: 'p-1', rank: 1, scraped_at: '2026-02-01' },
				{ player_id: 'p-2', rank: 2, scraped_at: '2026-02-01' },
			];

			mockGet.mockResolvedValueOnce(mockRankings);

			const result = await rankingsApi.getSourceRankings(sourceId);

			expect(mockGet).toHaveBeenCalledWith(
				`/ranking-sources/${sourceId}/rankings`,
				expect.any(Object),
			);
			expect(result).toEqual(mockRankings);
		});
	});

	describe('loadAllPlayerRankings', () => {
		it('should build badge map from all ranking entries', async () => {
			const mockEntries: AllRankingEntry[] = [
				{
					player_id: 'p-1',
					source_name: 'Tankathon',
					rank: 3,
					scraped_at: '2026-02-01',
				},
				{
					player_id: 'p-1',
					source_name: 'Walter Football',
					rank: 7,
					scraped_at: '2026-02-01',
				},
				{
					player_id: 'p-2',
					source_name: 'Tankathon',
					rank: 1,
					scraped_at: '2026-02-01',
				},
			];

			mockGet.mockResolvedValueOnce(mockEntries);

			const result = await rankingsApi.loadAllPlayerRankings();

			expect(mockGet).toHaveBeenCalledWith('/rankings', expect.any(Object));
			expect(result).toBeInstanceOf(Map);

			// Player 1 has two badges, sorted by rank (best first)
			const p1Badges = result.get('p-1');
			expect(p1Badges).toHaveLength(2);
			expect(p1Badges![0]).toEqual({ source_name: 'Tankathon', abbreviation: 'TK', rank: 3 });
			expect(p1Badges![1]).toEqual({
				source_name: 'Walter Football',
				abbreviation: 'WF',
				rank: 7,
			});

			// Player 2 has one badge
			const p2Badges = result.get('p-2');
			expect(p2Badges).toHaveLength(1);
			expect(p2Badges![0]).toEqual({ source_name: 'Tankathon', abbreviation: 'TK', rank: 1 });
		});

		it('should return empty map when no rankings exist', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await rankingsApi.loadAllPlayerRankings();

			expect(result).toBeInstanceOf(Map);
			expect(result.size).toBe(0);
		});

		it('should abbreviate known sources correctly', async () => {
			const mockEntries: AllRankingEntry[] = [
				{
					player_id: 'p-1',
					source_name: 'ESPN Draft Board',
					rank: 1,
					scraped_at: '2026-02-01',
				},
				{
					player_id: 'p-1',
					source_name: 'NFL.com',
					rank: 2,
					scraped_at: '2026-02-01',
				},
				{
					player_id: 'p-1',
					source_name: 'PFF Rankings',
					rank: 3,
					scraped_at: '2026-02-01',
				},
				{
					player_id: 'p-1',
					source_name: 'Custom Source',
					rank: 4,
					scraped_at: '2026-02-01',
				},
			];

			mockGet.mockResolvedValueOnce(mockEntries);

			const result = await rankingsApi.loadAllPlayerRankings();
			const badges = result.get('p-1')!;

			expect(badges[0].abbreviation).toBe('ESPN');
			expect(badges[1].abbreviation).toBe('NFL');
			expect(badges[2].abbreviation).toBe('PFF');
			expect(badges[3].abbreviation).toBe('CU'); // fallback: first 2 chars
		});
	});

	describe('error handling', () => {
		it('should propagate API errors from listSources', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Internal Server Error', 500));

			await expect(rankingsApi.listSources()).rejects.toThrow('Internal Server Error');
		});

		it('should propagate API errors from getPlayerRankings', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Not found', 404));

			await expect(rankingsApi.getPlayerRankings('bad-id')).rejects.toThrow('Not found');
		});

		it('should propagate API errors from loadAllPlayerRankings', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Internal Server Error', 500));

			await expect(rankingsApi.loadAllPlayerRankings()).rejects.toThrow(
				'Internal Server Error',
			);
		});
	});
});
