import { z } from 'zod';
import { apiClient } from './client';
import { TradeSchema, TradeProposalSchema, type Trade, type TradeProposal } from '$lib/types';

/**
 * Parameters for proposing a trade
 */
export interface ProposeTradeParams {
	session_id: string;
	from_team_id: string;
	to_team_id: string;
	from_team_pick_ids: string[];
	to_team_pick_ids: string[];
}

/**
 * Trades API module
 */
export const tradesApi = {
	/**
	 * Propose a new trade
	 */
	async propose(params: ProposeTradeParams): Promise<TradeProposal> {
		return apiClient.post('/trades', params, TradeProposalSchema);
	},

	/**
	 * Accept a trade
	 */
	async accept(tradeId: string, teamId: string): Promise<Trade> {
		return apiClient.post(`/trades/${tradeId}/accept`, { team_id: teamId }, TradeSchema);
	},

	/**
	 * Reject a trade
	 */
	async reject(tradeId: string, teamId: string): Promise<Trade> {
		return apiClient.post(`/trades/${tradeId}/reject`, { team_id: teamId }, TradeSchema);
	},

	/**
	 * Get all trades for a session
	 */
	async getBySession(sessionId: string): Promise<TradeProposal[]> {
		return apiClient.get(`/sessions/${sessionId}/trades`, z.array(TradeProposalSchema));
	},
};
