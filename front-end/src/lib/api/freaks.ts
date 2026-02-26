import { z } from 'zod';
import { apiClient } from './client';
import { FeldmanFreakSchema, type FeldmanFreak } from '$lib/types';
import { UUIDSchema } from '$lib/types/common';

// Schema for the list endpoint response (includes player_id)
const FeldmanFreakWithPlayerSchema = z.object({
	player_id: UUIDSchema,
	rank: z.number(),
	description: z.string(),
	article_url: z.string().nullable().optional(),
});

/**
 * Feldman Freaks API module
 */
export const freaksApi = {
	/**
	 * Load all Feldman Freaks for a given year, returning a Map keyed by player_id.
	 */
	async loadByYear(year: number): Promise<Map<string, FeldmanFreak>> {
		const entries = await apiClient.get(
			`/feldman-freaks?year=${year}`,
			z.array(FeldmanFreakWithPlayerSchema),
		);

		const map = new Map<string, FeldmanFreak>();
		for (const entry of entries) {
			map.set(entry.player_id, {
				rank: entry.rank,
				description: entry.description,
				article_url: entry.article_url,
			});
		}
		return map;
	},
};
