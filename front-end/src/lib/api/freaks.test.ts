import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { freaksApi } from './freaks';
import * as client from './client';

describe('freaksApi', () => {
	let mockGet: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockGet = vi.fn();
		vi.spyOn(client.apiClient, 'get').mockImplementation(mockGet as any);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('loadByYear', () => {
		it('should fetch freaks for a given year and return a map keyed by player_id', async () => {
			const mockResponse = [
				{
					player_id: 'p-1',
					rank: 1,
					description: 'Vertical jumped 41.5 inches',
					article_url: 'https://example.com/freaks',
				},
				{
					player_id: 'p-2',
					rank: 5,
					description: 'Bench pressed 425 lbs',
					article_url: null,
				},
			];

			mockGet.mockResolvedValueOnce(mockResponse);

			const result = await freaksApi.loadByYear(2026);

			expect(mockGet).toHaveBeenCalledWith(
				'/feldman-freaks?year=2026',
				expect.any(Object),
			);
			expect(result).toBeInstanceOf(Map);
			expect(result.size).toBe(2);

			const freak1 = result.get('p-1');
			expect(freak1).toEqual({
				rank: 1,
				description: 'Vertical jumped 41.5 inches',
				article_url: 'https://example.com/freaks',
			});

			const freak2 = result.get('p-2');
			expect(freak2).toEqual({
				rank: 5,
				description: 'Bench pressed 425 lbs',
				article_url: null,
			});
		});

		it('should return empty map when no freaks exist for the year', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await freaksApi.loadByYear(2025);

			expect(mockGet).toHaveBeenCalledWith(
				'/feldman-freaks?year=2025',
				expect.any(Object),
			);
			expect(result).toBeInstanceOf(Map);
			expect(result.size).toBe(0);
		});

		it('should handle freaks without article_url', async () => {
			const mockResponse = [
				{
					player_id: 'p-1',
					rank: 3,
					description: 'Elite speed',
					article_url: undefined,
				},
			];

			mockGet.mockResolvedValueOnce(mockResponse);

			const result = await freaksApi.loadByYear(2026);
			const freak = result.get('p-1');
			expect(freak).toBeDefined();
			expect(freak!.article_url).toBeUndefined();
		});

		it('should propagate API errors', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Internal Server Error', 500));

			await expect(freaksApi.loadByYear(2026)).rejects.toThrow('Internal Server Error');
		});

		it('should use the last entry when duplicate player_ids exist', async () => {
			const mockResponse = [
				{
					player_id: 'p-1',
					rank: 1,
					description: 'First entry',
					article_url: null,
				},
				{
					player_id: 'p-1',
					rank: 2,
					description: 'Second entry',
					article_url: null,
				},
			];

			mockGet.mockResolvedValueOnce(mockResponse);

			const result = await freaksApi.loadByYear(2026);
			expect(result.size).toBe(1);
			expect(result.get('p-1')!.description).toBe('Second entry');
		});
	});
});
