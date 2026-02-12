import { z } from 'zod';
import { apiClient } from './client';
import {
	DraftSchema,
	DraftPickSchema,
	AvailablePlayerSchema,
	type Draft,
	type DraftPick,
	type AvailablePlayer,
} from '$lib/types';

/**
 * Drafts API module
 */
export const draftsApi = {
	/**
	 * List all drafts
	 */
	async list(): Promise<Draft[]> {
		return apiClient.get('/drafts', z.array(DraftSchema));
	},

	/**
	 * Get a single draft by ID
	 */
	async get(id: string): Promise<Draft> {
		return apiClient.get(`/drafts/${id}`, DraftSchema);
	},

	/**
	 * Create a new draft
	 */
	async create(draft: { name: string; year: number; rounds: number }): Promise<Draft> {
		return apiClient.post('/drafts', draft, DraftSchema);
	},

	/**
	 * Get all picks for a draft
	 */
	async getPicks(draftId: string): Promise<DraftPick[]> {
		return apiClient.get(`/drafts/${draftId}/picks`, z.array(DraftPickSchema));
	},

	/**
	 * Make a pick in the draft
	 */
	async makePick(draftId: string, pickId: string, playerId: string): Promise<DraftPick> {
		return apiClient.post(`/picks/${pickId}/make`, { player_id: playerId }, DraftPickSchema);
	},

	/**
	 * Get consolidated available players with scouting grades and rankings.
	 * Replaces separate calls to /players, /rankings, /ranking-sources, and /scouting-reports.
	 */
	async getAvailablePlayers(draftId: string, teamId?: string): Promise<AvailablePlayer[]> {
		const params = teamId ? `?team_id=${teamId}` : '';
		return apiClient.get(
			`/drafts/${draftId}/available-players${params}`,
			z.array(AvailablePlayerSchema)
		);
	},

	/**
	 * Initialize draft picks based on team draft order
	 */
	async initializePicks(draftId: string): Promise<DraftPick[]> {
		return apiClient.post(`/drafts/${draftId}/initialize`, {}, z.array(DraftPickSchema));
	},
};
