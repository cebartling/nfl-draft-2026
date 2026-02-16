import {
	OFFENSE_POSITIONS,
	DEFENSE_POSITIONS,
	SPECIAL_POSITIONS,
	POSITION_GROUPS,
} from '$lib/types';
import type { Player, Position, PositionGroup } from '$lib/types';

const ALL_POSITIONS: Position[] = [
	...OFFENSE_POSITIONS,
	...DEFENSE_POSITIONS,
	...SPECIAL_POSITIONS,
];

/**
 * Returns the positions belonging to a group, or all positions for 'all'.
 */
export function getPositionsForGroup(group: string): Position[] {
	if (group === 'all') return ALL_POSITIONS;
	const positions = POSITION_GROUPS[group as PositionGroup];
	return positions ?? ALL_POSITIONS;
}

/**
 * Pure filter function for prospect rankings.
 * Applies search, position group, and position filters with AND logic.
 */
export function filterProspects(
	sortedPlayerIds: string[],
	playerMap: Map<string, Player>,
	searchQuery: string,
	selectedGroup: string,
	selectedPosition: string,
): string[] {
	const hasSearch = searchQuery.length > 0;
	const hasGroup = selectedGroup !== 'all';
	const hasPosition = selectedPosition !== 'all';

	if (!hasSearch && !hasGroup && !hasPosition) {
		// Still need to filter out IDs not in the map
		return sortedPlayerIds.filter((id) => playerMap.has(id));
	}

	const query = hasSearch ? searchQuery.toLowerCase() : '';
	const groupPositions = hasGroup ? getPositionsForGroup(selectedGroup) : null;

	return sortedPlayerIds.filter((id) => {
		const player = playerMap.get(id);
		if (!player) return false;

		if (hasSearch) {
			const matchesName =
				player.first_name.toLowerCase().includes(query) ||
				player.last_name.toLowerCase().includes(query);
			const matchesCollege = player.college?.toLowerCase().includes(query);
			if (!matchesName && !matchesCollege) return false;
		}

		if (groupPositions && !groupPositions.includes(player.position)) {
			return false;
		}

		if (hasPosition && player.position !== selectedPosition) {
			return false;
		}

		return true;
	});
}
