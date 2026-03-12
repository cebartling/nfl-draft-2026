import { z } from 'zod';
import { apiClient } from './client';
import {
	CombineResultsWithPlayerSchema,
	CombinePercentileSchema,
	RasScoreSchema,
	type CombineResultsWithPlayer,
	type CombinePercentile,
	type RasScore,
} from '$lib/types';

/**
 * Combine Results API module
 */
export const combineApi = {
	/**
	 * List all combine results with player info
	 */
	async listAll(): Promise<CombineResultsWithPlayer[]> {
		return apiClient.get('/combine-results', z.array(CombineResultsWithPlayerSchema));
	},

	/**
	 * Get combine percentile data
	 */
	async getPercentiles(): Promise<CombinePercentile[]> {
		return apiClient.get('/combine-percentiles', z.array(CombinePercentileSchema));
	},

	/**
	 * Get RAS scores for all players with combine data
	 */
	async listRasScores(): Promise<RasScore[]> {
		return apiClient.get('/combine-results/ras', z.array(RasScoreSchema));
	},
};
