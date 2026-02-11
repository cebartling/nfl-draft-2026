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

/** Map a source name to a short abbreviation for badge display */
function abbreviateSource(name: string): string {
	const lower = name.toLowerCase();
	if (lower.includes('tankathon')) return 'TK';
	if (lower.includes('walter')) return 'WF';
	if (lower.includes('espn')) return 'ESPN';
	if (lower.includes('nfl')) return 'NFL';
	if (lower.includes('pff')) return 'PFF';
	// Fallback: first 2 chars uppercase
	return name.slice(0, 2).toUpperCase();
}

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
	 * Build a map of player_id -> RankingBadge[] using a single API request.
	 */
	async loadAllPlayerRankings(): Promise<Map<string, RankingBadge[]>> {
		const entries = await apiClient.get('/rankings', z.array(AllRankingEntrySchema));
		const map = new Map<string, RankingBadge[]>();

		for (const entry of entries) {
			const badge: RankingBadge = {
				source_name: entry.source_name,
				abbreviation: abbreviateSource(entry.source_name),
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
