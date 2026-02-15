import type { RankingBadge } from '$lib/types';

export interface ProspectRanking {
	playerId: string;
	consensusRank: number;
	sourceCount: number;
}

/**
 * Compute consensus rankings by averaging each player's rank across all sources.
 */
export function computeConsensusRankings(
	playerRankings: Map<string, RankingBadge[]>,
): Map<string, ProspectRanking> {
	const result = new Map<string, ProspectRanking>();

	for (const [playerId, badges] of playerRankings) {
		if (badges.length === 0) continue;

		const sum = badges.reduce((acc, b) => acc + b.rank, 0);
		const average = sum / badges.length;

		result.set(playerId, {
			playerId,
			consensusRank: average,
			sourceCount: badges.length,
		});
	}

	return result;
}

/**
 * Sort player IDs by consensus rank ascending, then source count descending
 * (more sources = higher confidence), then player ID for stability.
 */
export function sortByConsensusRank(
	rankings: Map<string, ProspectRanking>,
): string[] {
	return [...rankings.values()]
		.sort((a, b) => {
			if (a.consensusRank !== b.consensusRank) {
				return a.consensusRank - b.consensusRank;
			}
			if (a.sourceCount !== b.sourceCount) {
				return b.sourceCount - a.sourceCount;
			}
			return a.playerId.localeCompare(b.playerId);
		})
		.map((r) => r.playerId);
}
