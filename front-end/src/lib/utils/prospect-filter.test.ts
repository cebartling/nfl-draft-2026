import { describe, it, expect } from 'vitest';
import { filterProspects, getPositionsForGroup } from './prospect-filter';
import type { Player, Position } from '$lib/types';

function makePlayer(overrides: Partial<Player> & { id: string; position: Position }): Player {
	return {
		first_name: 'John',
		last_name: 'Doe',
		college: 'State U',
		height_inches: 72,
		weight_pounds: 220,
		draft_year: 2026,
		draft_eligible: true,
		...overrides,
	};
}

function buildPlayerMap(players: Player[]): Map<string, Player> {
	const map = new Map<string, Player>();
	for (const p of players) {
		map.set(p.id, p);
	}
	return map;
}

describe('getPositionsForGroup', () => {
	it('should return all positions for "all"', () => {
		const result = getPositionsForGroup('all');
		expect(result).toHaveLength(14);
		expect(result).toContain('QB');
		expect(result).toContain('DE');
		expect(result).toContain('K');
	});

	it('should return offense positions', () => {
		const result = getPositionsForGroup('offense');
		expect(result).toEqual(['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C']);
	});

	it('should return defense positions', () => {
		const result = getPositionsForGroup('defense');
		expect(result).toEqual(['DE', 'DT', 'LB', 'CB', 'S']);
	});

	it('should return special teams positions', () => {
		const result = getPositionsForGroup('special_teams');
		expect(result).toEqual(['K', 'P']);
	});

	it('should return all positions for unknown group', () => {
		const result = getPositionsForGroup('unknown');
		expect(result).toHaveLength(14);
	});
});

describe('filterProspects', () => {
	const qb = makePlayer({ id: 'p1', first_name: 'Patrick', last_name: 'Mahomes', position: 'QB', college: 'Texas Tech' });
	const wr = makePlayer({ id: 'p2', first_name: 'Ja\'Marr', last_name: 'Chase', position: 'WR', college: 'LSU' });
	const cb = makePlayer({ id: 'p3', first_name: 'Sauce', last_name: 'Gardner', position: 'CB', college: 'Cincinnati' });
	const k = makePlayer({ id: 'p4', first_name: 'Tyler', last_name: 'Bass', position: 'K', college: 'Georgia Southern' });
	const de = makePlayer({ id: 'p5', first_name: 'Myles', last_name: 'Garrett', position: 'DE', college: 'Texas A&M' });

	const allPlayers = [qb, wr, cb, k, de];
	const playerMap = buildPlayerMap(allPlayers);
	const sortedIds = ['p1', 'p2', 'p3', 'p4', 'p5'];

	it('should return all IDs when no filters are applied', () => {
		const result = filterProspects(sortedIds, playerMap, '', 'all', 'all');
		expect(result).toEqual(sortedIds);
	});

	it('should filter by search (first name, case-insensitive)', () => {
		const result = filterProspects(sortedIds, playerMap, 'patrick', 'all', 'all');
		expect(result).toEqual(['p1']);
	});

	it('should filter by search (last name, case-insensitive)', () => {
		const result = filterProspects(sortedIds, playerMap, 'CHASE', 'all', 'all');
		expect(result).toEqual(['p2']);
	});

	it('should filter by search (college, case-insensitive)', () => {
		const result = filterProspects(sortedIds, playerMap, 'lsu', 'all', 'all');
		expect(result).toEqual(['p2']);
	});

	it('should filter by position group (offense)', () => {
		const result = filterProspects(sortedIds, playerMap, '', 'offense', 'all');
		expect(result).toEqual(['p1', 'p2']);
	});

	it('should filter by position group (defense)', () => {
		const result = filterProspects(sortedIds, playerMap, '', 'defense', 'all');
		expect(result).toEqual(['p3', 'p5']);
	});

	it('should filter by position group (special_teams)', () => {
		const result = filterProspects(sortedIds, playerMap, '', 'special_teams', 'all');
		expect(result).toEqual(['p4']);
	});

	it('should filter by specific position', () => {
		const result = filterProspects(sortedIds, playerMap, '', 'all', 'CB');
		expect(result).toEqual(['p3']);
	});

	it('should combine group and position filters (AND logic)', () => {
		const result = filterProspects(sortedIds, playerMap, '', 'defense', 'CB');
		expect(result).toEqual(['p3']);
	});

	it('should combine search, group, and position filters', () => {
		const result = filterProspects(sortedIds, playerMap, 'sauce', 'defense', 'CB');
		expect(result).toEqual(['p3']);
	});

	it('should return empty when search matches no one', () => {
		const result = filterProspects(sortedIds, playerMap, 'zzzzz', 'all', 'all');
		expect(result).toEqual([]);
	});

	it('should return empty for empty input list', () => {
		const result = filterProspects([], playerMap, '', 'all', 'all');
		expect(result).toEqual([]);
	});

	it('should skip IDs not found in player map', () => {
		const result = filterProspects(['missing-id', 'p1'], playerMap, '', 'all', 'all');
		expect(result).toEqual(['p1']);
	});

	it('should preserve original sort order', () => {
		const reversed = ['p5', 'p4', 'p3', 'p2', 'p1'];
		const result = filterProspects(reversed, playerMap, '', 'all', 'all');
		expect(result).toEqual(reversed);
	});
});
