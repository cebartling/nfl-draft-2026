import { describe, it, expect } from 'vitest';
import { computeConsensusRankings, sortByConsensusRank } from './prospect-ranking';
import type { RankingBadge } from '$lib/types';

function badge(source: string, rank: number): RankingBadge {
	return { source_name: source, abbreviation: source.slice(0, 2).toUpperCase(), rank };
}

describe('computeConsensusRankings', () => {
	it('should compute average rank for a single source', () => {
		const input = new Map([['p1', [badge('ESPN', 5)]]]);

		const result = computeConsensusRankings(input);

		expect(result.get('p1')).toEqual({
			playerId: 'p1',
			consensusRank: 5,
			sourceCount: 1,
		});
	});

	it('should compute average rank across multiple sources', () => {
		const input = new Map([
			['p1', [badge('ESPN', 3), badge('Tankathon', 7), badge('Walter', 5)]],
		]);

		const result = computeConsensusRankings(input);

		expect(result.get('p1')).toEqual({
			playerId: 'p1',
			consensusRank: 5,
			sourceCount: 3,
		});
	});

	it('should handle multiple players', () => {
		const input = new Map([
			['p1', [badge('ESPN', 1), badge('Tankathon', 3)]],
			['p2', [badge('ESPN', 10)]],
		]);

		const result = computeConsensusRankings(input);

		expect(result.size).toBe(2);
		expect(result.get('p1')?.consensusRank).toBe(2);
		expect(result.get('p2')?.consensusRank).toBe(10);
	});

	it('should return empty map for empty input', () => {
		const result = computeConsensusRankings(new Map());
		expect(result.size).toBe(0);
	});

	it('should skip players with empty badge arrays', () => {
		const input = new Map<string, RankingBadge[]>([['p1', []]]);

		const result = computeConsensusRankings(input);

		expect(result.size).toBe(0);
	});

	it('should compute non-integer average correctly', () => {
		const input = new Map([['p1', [badge('ESPN', 1), badge('Tankathon', 2)]]]);

		const result = computeConsensusRankings(input);

		expect(result.get('p1')?.consensusRank).toBe(1.5);
	});
});

describe('sortByConsensusRank', () => {
	it('should sort by consensus rank ascending', () => {
		const rankings = new Map([
			['p1', { playerId: 'p1', consensusRank: 10, sourceCount: 2 }],
			['p2', { playerId: 'p2', consensusRank: 1, sourceCount: 2 }],
			['p3', { playerId: 'p3', consensusRank: 5, sourceCount: 2 }],
		]);

		const result = sortByConsensusRank(rankings);

		expect(result).toEqual(['p2', 'p3', 'p1']);
	});

	it('should break ties by source count descending (more sources first)', () => {
		const rankings = new Map([
			['p1', { playerId: 'p1', consensusRank: 5, sourceCount: 1 }],
			['p2', { playerId: 'p2', consensusRank: 5, sourceCount: 3 }],
		]);

		const result = sortByConsensusRank(rankings);

		expect(result).toEqual(['p2', 'p1']);
	});

	it('should break further ties by player ID for stability', () => {
		const rankings = new Map([
			['p2', { playerId: 'p2', consensusRank: 5, sourceCount: 2 }],
			['p1', { playerId: 'p1', consensusRank: 5, sourceCount: 2 }],
		]);

		const result = sortByConsensusRank(rankings);

		expect(result).toEqual(['p1', 'p2']);
	});

	it('should return empty array for empty input', () => {
		const result = sortByConsensusRank(new Map());
		expect(result).toEqual([]);
	});

	it('should handle single player', () => {
		const rankings = new Map([
			['p1', { playerId: 'p1', consensusRank: 1, sourceCount: 5 }],
		]);

		const result = sortByConsensusRank(rankings);

		expect(result).toEqual(['p1']);
	});
});
