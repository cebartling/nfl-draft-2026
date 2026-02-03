import { draftsApi, sessionsApi } from '$lib/api';
import type { Draft, DraftSession, DraftPick } from '$lib/types';

/**
 * Draft state management using Svelte 5 runes
 */
export class DraftState {
	// Reactive state
	draft = $state<Draft | null>(null);
	session = $state<DraftSession | null>(null);
	picks = $state<DraftPick[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);

	/**
	 * Get the current pick number from the session
	 */
	get currentPickNumber(): number {
		return this.session?.current_pick_number ?? 1;
	}

	/**
	 * Get the current pick object
	 */
	get currentPick(): DraftPick | null {
		if (!this.picks.length) return null;
		return this.picks.find((pick) => pick.overall_pick === this.currentPickNumber) ?? null;
	}

	/**
	 * Get all completed picks (picks with a player selected)
	 */
	get completedPicks(): DraftPick[] {
		return this.picks.filter((pick) => pick.player_id !== undefined);
	}

	/**
	 * Get all available picks (picks without a player selected)
	 */
	get availablePicks(): DraftPick[] {
		return this.picks.filter((pick) => pick.player_id === undefined);
	}

	/**
	 * Get picks by round
	 */
	getPicksByRound(round: number): DraftPick[] {
		return this.picks.filter((pick) => pick.round === round);
	}

	/**
	 * Get picks by team
	 */
	getPicksByTeam(teamId: string): DraftPick[] {
		return this.picks.filter((pick) => pick.current_team_id === teamId);
	}

	/**
	 * Load draft data
	 */
	async loadDraft(draftId: string): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const [draft, picks] = await Promise.all([
				draftsApi.get(draftId),
				draftsApi.getPicks(draftId),
			]);

			this.draft = draft;
			this.picks = picks;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load draft';
			console.error('Failed to load draft:', err);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Load session data
	 */
	async loadSession(sessionId: string): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const session = await sessionsApi.get(sessionId);
			this.session = session;

			// Load the associated draft if not already loaded
			if (!this.draft || this.draft.id !== session.draft_id) {
				await this.loadDraft(session.draft_id);
			}
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load session';
			console.error('Failed to load session:', err);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Start a draft session
	 */
	async startSession(sessionId: string): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const session = await sessionsApi.start(sessionId);
			this.session = session;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to start session';
			console.error('Failed to start session:', err);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Pause a draft session
	 */
	async pauseSession(sessionId: string): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const session = await sessionsApi.pause(sessionId);
			this.session = session;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to pause session';
			console.error('Failed to pause session:', err);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Update a pick from WebSocket message
	 */
	updatePickFromWS(pickData: {
		pick_id: string;
		player_id: string;
		team_id: string;
	}): void {
		const pickIndex = this.picks.findIndex((pick) => pick.id === pickData.pick_id);
		if (pickIndex !== -1) {
			// Update the pick with the player
			this.picks[pickIndex] = {
				...this.picks[pickIndex],
				player_id: pickData.player_id,
				picked_at: new Date().toISOString(),
			};
		}
	}

	/**
	 * Advance to the next pick
	 */
	advancePick(): void {
		if (this.session) {
			this.session = {
				...this.session,
				current_pick_number: this.session.current_pick_number + 1,
			};
		}
	}

	/**
	 * Reset state
	 */
	reset(): void {
		this.draft = null;
		this.session = null;
		this.picks = [];
		this.isLoading = false;
		this.error = null;
	}
}

/**
 * Singleton draft state instance
 */
export const draftState = new DraftState();
