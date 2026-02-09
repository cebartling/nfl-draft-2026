import { z } from 'zod';
import { apiClient } from './client';
import {
	DraftSessionSchema,
	DraftEventSchema,
	DraftPickSchema,
	type DraftSession,
	type DraftEvent,
	type DraftPick,
	type ChartType,
} from '$lib/types';

/**
 * Parameters for creating a draft session
 */
export interface CreateSessionParams {
	draft_id: string;
	time_per_pick_seconds: number;
	auto_pick_enabled: boolean;
	chart_type: ChartType;
	controlled_team_ids?: string[];
}

/**
 * Draft Sessions API module
 */
export const sessionsApi = {
	/**
	 * Create a new draft session
	 */
	async create(params: CreateSessionParams): Promise<DraftSession> {
		return apiClient.post('/sessions', params, DraftSessionSchema);
	},

	/**
	 * Get a draft session by ID
	 */
	async get(id: string): Promise<DraftSession> {
		return apiClient.get(`/sessions/${id}`, DraftSessionSchema);
	},

	/**
	 * Get a draft session by draft ID
	 */
	async getByDraftId(draftId: string): Promise<DraftSession> {
		return apiClient.get(`/drafts/${draftId}/session`, DraftSessionSchema);
	},

	/**
	 * Start a draft session
	 */
	async start(id: string): Promise<DraftSession> {
		return apiClient.post(`/sessions/${id}/start`, {}, DraftSessionSchema);
	},

	/**
	 * Pause a draft session
	 */
	async pause(id: string): Promise<DraftSession> {
		return apiClient.post(`/sessions/${id}/pause`, {}, DraftSessionSchema);
	},

	/**
	 * Get all events for a session
	 */
	async getEvents(id: string): Promise<DraftEvent[]> {
		return apiClient.get(`/sessions/${id}/events`, z.array(DraftEventSchema));
	},

	/**
	 * Run auto-picks for AI-controlled teams until a user-controlled team's turn
	 */
	async autoPickRun(
		id: string
	): Promise<{ session: DraftSession; picks_made: DraftPick[] }> {
		return apiClient.post(
			`/sessions/${id}/auto-pick-run`,
			{},
			z.object({
				session: DraftSessionSchema,
				picks_made: z.array(DraftPickSchema),
			})
		);
	},

	/**
	 * Advance the session's current pick number
	 */
	async advancePick(id: string): Promise<DraftSession> {
		return apiClient.post(`/sessions/${id}/advance-pick`, {}, DraftSessionSchema);
	},
};
