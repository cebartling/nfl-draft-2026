import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { combineApi } from './combine';
import * as client from './client';
import type { CombineResultsWithPlayer, CombinePercentile } from '$lib/types';

describe('combineApi', () => {
	let mockGet: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockGet = vi.fn();
		vi.spyOn(client.apiClient, 'get').mockImplementation(mockGet as any);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('listAll', () => {
		it('should fetch all combine results with player info', async () => {
			const mockResults: CombineResultsWithPlayer[] = [
				{
					id: '1',
					player_id: '10',
					player_first_name: 'John',
					player_last_name: 'Doe',
					position: 'QB',
					college: 'Alabama',
					year: 2026,
					source: 'combine',
					forty_yard_dash: 4.5,
					bench_press: 20,
					vertical_jump: 36,
					broad_jump: 120,
				},
			];

			mockGet.mockResolvedValueOnce(mockResults);

			const result = await combineApi.listAll();

			expect(mockGet).toHaveBeenCalledWith('/combine-results', expect.any(Object));
			expect(result).toEqual(mockResults);
		});

		it('should handle empty results', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await combineApi.listAll();

			expect(result).toEqual([]);
		});
	});

	describe('getPercentiles', () => {
		it('should fetch combine percentile data', async () => {
			const mockPercentiles: CombinePercentile[] = [
				{
					id: '1',
					position: 'QB',
					measurement: 'forty_yard_dash',
					sample_size: 100,
					min_value: 4.3,
					p10: 4.4,
					p20: 4.5,
					p30: 4.55,
					p40: 4.6,
					p50: 4.65,
					p60: 4.7,
					p70: 4.75,
					p80: 4.8,
					p90: 4.9,
					max_value: 5.0,
					years_start: 2020,
					years_end: 2025,
				},
			];

			mockGet.mockResolvedValueOnce(mockPercentiles);

			const result = await combineApi.getPercentiles();

			expect(mockGet).toHaveBeenCalledWith('/combine-percentiles', expect.any(Object));
			expect(result).toEqual(mockPercentiles);
		});

		it('should handle empty percentile data', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await combineApi.getPercentiles();

			expect(result).toEqual([]);
		});
	});
});
