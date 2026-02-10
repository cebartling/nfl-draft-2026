import { logger } from '$lib/utils/logger';
import { playersApi } from '$lib/api';
import type { Player, Position } from '$lib/types';

/**
 * Players state management using Svelte 5 runes
 */
export class PlayersState {
	// Reactive state
	allPlayers = $state<Player[]>([]);
	selectedPlayer = $state<Player | null>(null);
	isLoading = $state(false);
	error = $state<string | null>(null);

	/**
	 * Get players organized by position
	 */
	get byPosition(): Map<Position, Player[]> {
		const map = new Map<Position, Player[]>();

		for (const player of this.allPlayers) {
			const existing = map.get(player.position) ?? [];
			map.set(player.position, [...existing, player]);
		}

		return map;
	}

	/**
	 * Get players by a specific position
	 */
	getByPosition(position: Position): Player[] {
		return this.allPlayers.filter((player) => player.position === position);
	}

	/**
	 * Get draft eligible players only
	 */
	get draftEligible(): Player[] {
		return this.allPlayers.filter((player) => player.draft_eligible);
	}

	/**
	 * Load all players
	 */
	async loadAll(): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const players = await playersApi.list();
			this.allPlayers = players;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load players';
			logger.error('Failed to load players:', err);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Load a specific player
	 */
	async loadPlayer(playerId: string): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const player = await playersApi.get(playerId);
			this.selectedPlayer = player;

			// Also update in allPlayers if exists
			const index = this.allPlayers.findIndex((p) => p.id === playerId);
			if (index !== -1) {
				this.allPlayers[index] = player;
			} else {
				this.allPlayers = [...this.allPlayers, player];
			}
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load player';
			logger.error('Failed to load player:', err);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Filter available players (not in the picked players list)
	 */
	filterAvailable(pickedPlayerIds: Set<string>): Player[] {
		return this.allPlayers.filter((player) => !pickedPlayerIds.has(player.id));
	}

	/**
	 * Search players by name
	 */
	searchByName(query: string): Player[] {
		const lowerQuery = query.toLowerCase();
		return this.allPlayers.filter(
			(player) =>
				player.first_name.toLowerCase().includes(lowerQuery) ||
				player.last_name.toLowerCase().includes(lowerQuery)
		);
	}

	/**
	 * Get player full name
	 */
	getPlayerFullName(player: Player): string {
		return `${player.first_name} ${player.last_name}`;
	}

	/**
	 * Reset state
	 */
	reset(): void {
		this.allPlayers = [];
		this.selectedPlayer = null;
		this.isLoading = false;
		this.error = null;
	}
}

/**
 * Singleton players state instance
 */
export const playersState = new PlayersState();
