import { z } from 'zod';
import { apiClient } from './client';
import {
	RankingSourceSchema,
	PlayerRankingSchema,
	SourceRankingSchema,
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
	 * Build a map of player_id -> RankingBadge[] by fetching all sources and their rankings.
	 * This is more efficient than fetching per-player rankings.
	 */
	async loadAllPlayerRankings(): Promise<Map<string, RankingBadge[]>> {
		const sources = await this.listSources();
		const map = new Map<string, RankingBadge[]>();

		const results = await Promise.all(
			sources.map(async (source) => {
				const rankings = await this.getSourceRankings(source.id);
				return { source, rankings };
			})
		);

		for (const { source, rankings } of results) {
			const abbr = abbreviateSource(source.name);
			for (const ranking of rankings) {
				const badge: RankingBadge = {
					source_name: source.name,
					abbreviation: abbr,
					rank: ranking.rank,
				};
				const existing = map.get(ranking.player_id);
				if (existing) {
					existing.push(badge);
				} else {
					map.set(ranking.player_id, [badge]);
				}
			}
		}

		// Sort each player's badges by rank (best first)
		for (const badges of map.values()) {
			badges.sort((a, b) => a.rank - b.rank);
		}

		return map;
	},
};
