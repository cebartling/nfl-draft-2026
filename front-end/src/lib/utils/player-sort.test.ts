import { describe, it, expect } from 'vitest';
import { sortByScoutingGrade } from './player-sort';
import type { Player } from '$lib/types';

function makePlayer(overrides: Partial<Player> & { id: string; first_name: string; last_name: string }): Player {
	return {
		position: 'QB',
		college: 'Test U',
		height_inches: 74,
		weight_lbs: 220,
		...overrides,
	} as Player;
}

describe('sortByScoutingGrade', () => {
	it('should sort graded players before ungraded players', () => {
		const players = [
			makePlayer({ id: '1', first_name: 'Alpha', last_name: 'Ungraded' }),
			makePlayer({ id: '2', first_name: 'Beta', last_name: 'Graded' }),
		];
		const grades = new Map([['2', 85]]);

		const result = sortByScoutingGrade(players, grades);

		expect(result[0].id).toBe('2');
		expect(result[1].id).toBe('1');
	});

	it('should sort graded players by grade descending', () => {
		const players = [
			makePlayer({ id: '1', first_name: 'Low', last_name: 'Grade' }),
			makePlayer({ id: '2', first_name: 'High', last_name: 'Grade' }),
			makePlayer({ id: '3', first_name: 'Mid', last_name: 'Grade' }),
		];
		const grades = new Map([
			['1', 70],
			['2', 95],
			['3', 85],
		]);

		const result = sortByScoutingGrade(players, grades);

		expect(result.map((p) => p.id)).toEqual(['2', '3', '1']);
	});

	it('should use alphabetical tiebreaker by last name then first name', () => {
		const players = [
			makePlayer({ id: '1', first_name: 'Zach', last_name: 'Wilson' }),
			makePlayer({ id: '2', first_name: 'Aaron', last_name: 'Wilson' }),
			makePlayer({ id: '3', first_name: 'Tom', last_name: 'Brady' }),
		];
		const grades = new Map([
			['1', 90],
			['2', 90],
			['3', 90],
		]);

		const result = sortByScoutingGrade(players, grades);

		// Brady first (alphabetical last name), then Aaron Wilson, then Zach Wilson
		expect(result.map((p) => p.id)).toEqual(['3', '2', '1']);
	});

	it('should sort all ungraded players alphabetically', () => {
		const players = [
			makePlayer({ id: '1', first_name: 'Zach', last_name: 'Wilson' }),
			makePlayer({ id: '2', first_name: 'Tom', last_name: 'Brady' }),
			makePlayer({ id: '3', first_name: 'Aaron', last_name: 'Rodgers' }),
		];
		const grades = new Map<string, number>();

		const result = sortByScoutingGrade(players, grades);

		// Brady, Rodgers, Wilson (alphabetical by last name)
		expect(result.map((p) => p.id)).toEqual(['2', '3', '1']);
	});

	it('should return empty array for empty input', () => {
		const result = sortByScoutingGrade([], new Map());
		expect(result).toEqual([]);
	});

	it('should not mutate the original array', () => {
		const players = [
			makePlayer({ id: '1', first_name: 'Zach', last_name: 'Wilson' }),
			makePlayer({ id: '2', first_name: 'Tom', last_name: 'Brady' }),
		];
		const original = [...players];

		sortByScoutingGrade(players, new Map());

		expect(players.map((p) => p.id)).toEqual(original.map((p) => p.id));
	});
});
