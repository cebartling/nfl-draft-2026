import type { Player } from '$lib/types';

/**
 * Sort players by scouting grade (graded first, descending),
 * with alphabetical tiebreaker (last name, then first name).
 */
export function sortByScoutingGrade(
	players: Player[],
	grades: Map<string, number>
): Player[] {
	return [...players].sort((a, b) => {
		const gradeA = grades.get(a.id);
		const gradeB = grades.get(b.id);

		// Players with grades sort before players without
		if (gradeA !== undefined && gradeB === undefined) return -1;
		if (gradeA === undefined && gradeB !== undefined) return 1;

		// Both have grades: sort descending
		if (gradeA !== undefined && gradeB !== undefined) {
			if (gradeB !== gradeA) return gradeB - gradeA;
		}

		// Tiebreaker: alphabetical by last name, then first name
		const lastCmp = a.last_name.localeCompare(b.last_name);
		if (lastCmp !== 0) return lastCmp;
		return a.first_name.localeCompare(b.first_name);
	});
}
