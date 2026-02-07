import { z } from 'zod';
import { apiClient } from './client';
import {
	DraftSchema,
	DraftPickSchema,
	PlayerSchema,
	type Draft,
	type DraftPick,
	type Player,
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
		return apiClient.post(
			`/drafts/${draftId}/picks/${pickId}`,
			{ player_id: playerId },
			DraftPickSchema
		);
	},

	/**
	 * Get available players for a draft (not yet picked)
	 */
	async getAvailablePlayers(draftId: string): Promise<Player[]> {
		return apiClient.get(`/drafts/${draftId}/available-players`, z.array(PlayerSchema));
	},

	/**
	 * Initialize draft picks based on team draft order
	 */
	async initializePicks(draftId: string): Promise<DraftPick[]> {
		return apiClient.post(`/drafts/${draftId}/initialize`, {}, z.array(DraftPickSchema));
	},
};
