import { z } from 'zod';
import { apiClient } from './client';
import {
	RankingSourceSchema,
	PlayerRankingSchema,
	SourceRankingSchema,
	AllRankingEntrySchema,
	type RankingSource,
	type PlayerRanking,
	type SourceRanking,
	type RankingBadge,
} from '$lib/types';

/**
 * Rankings API module
 */
export const rankingsApi = {
	/**
	 * List all ranking sources
	 */
	async listSources(): Promise<RankingSource[]> {
		return apiClient.get('/ranking-sources', z.array(RankingSourceSchema));
	},

	/**
	 * Get all rankings for a player (across all sources)
	 */
	async getPlayerRankings(playerId: string): Promise<PlayerRanking[]> {
		return apiClient.get(`/players/${playerId}/rankings`, z.array(PlayerRankingSchema));
	},

	/**
	 * Get full big board for a source
	 */
	async getSourceRankings(sourceId: string): Promise<SourceRanking[]> {
		return apiClient.get(`/ranking-sources/${sourceId}/rankings`, z.array(SourceRankingSchema));
	},

	/**
	 * Build a map of player_id -> RankingBadge[] using two API requests:
	 * one for sources (abbreviations) and one for all rankings.
	 */
	async loadAllPlayerRankings(): Promise<Map<string, RankingBadge[]>> {
		const [sources, entries] = await Promise.all([
			apiClient.get('/ranking-sources', z.array(RankingSourceSchema)),
			apiClient.get('/rankings', z.array(AllRankingEntrySchema)),
		]);

		// Build abbreviation lookup from backend-provided values
		const abbreviations = new Map<string, string>();
		for (const source of sources) {
			abbreviations.set(source.name, source.abbreviation);
		}

		const map = new Map<string, RankingBadge[]>();

		for (const entry of entries) {
			const badge: RankingBadge = {
				source_name: entry.source_name,
				abbreviation: abbreviations.get(entry.source_name) ?? entry.source_name.slice(0, 2).toUpperCase(),
				rank: entry.rank,
			};
			const existing = map.get(entry.player_id);
			if (existing) {
				existing.push(badge);
			} else {
				map.set(entry.player_id, [badge]);
			}
		}

		// Sort each player's badges by rank (best first)
		for (const badges of map.values()) {
			badges.sort((a, b) => a.rank - b.rank);
		}

		return map;
	},
};
