import { logger } from '$lib/utils/logger';
import { tradesApi, type ProposeTradeParams } from '$lib/api/trades';
import type { Trade, TradeProposal } from '$lib/types';

/**
 * Trades state management using Svelte 5 runes.
 * Holds proposals for the current draft session and applies live
 * WebSocket events so the UI stays in sync with counterparty actions.
 */
export class TradesState {
	proposals = $state<TradeProposal[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	private sessionId: string | null = null;

	pendingForTeam(teamId: string): TradeProposal[] {
		return this.proposals.filter(
			(p) => p.trade.status === 'Proposed' && p.trade.to_team_id === teamId
		);
	}

	outboundForTeam(teamId: string): TradeProposal[] {
		return this.proposals.filter(
			(p) => p.trade.status === 'Proposed' && p.trade.from_team_id === teamId
		);
	}

	async load(sessionId: string): Promise<void> {
		this.sessionId = sessionId;
		this.isLoading = true;
		this.error = null;

		try {
			this.proposals = await tradesApi.getBySession(sessionId);
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load trades';
			logger.error('Failed to load session trades:', err);
		} finally {
			this.isLoading = false;
		}
	}

	async propose(params: ProposeTradeParams): Promise<TradeProposal> {
		const proposal = await tradesApi.propose(params);
		this.upsertProposal(proposal);
		return proposal;
	}

	async accept(tradeId: string, teamId: string): Promise<Trade> {
		const previous = this.proposals.find((p) => p.trade.id === tradeId);
		this.updateStatus(tradeId, 'Accepted');
		try {
			return await tradesApi.accept(tradeId, teamId);
		} catch (err) {
			if (previous) this.upsertProposal(previous);
			throw err;
		}
	}

	async reject(tradeId: string, teamId: string): Promise<Trade> {
		const previous = this.proposals.find((p) => p.trade.id === tradeId);
		this.updateStatus(tradeId, 'Rejected');
		try {
			return await tradesApi.reject(tradeId, teamId);
		} catch (err) {
			if (previous) this.upsertProposal(previous);
			throw err;
		}
	}

	onTradeProposed(payload: {
		trade_id: string;
		session_id: string;
		from_team_id: string;
		to_team_id: string;
		from_team_picks: string[];
		to_team_picks: string[];
		from_team_value: number;
		to_team_value: number;
	}): void {
		if (this.sessionId && payload.session_id !== this.sessionId) return;
		if (this.proposals.some((p) => p.trade.id === payload.trade_id)) return;
		this.proposals = [
			{
				trade: {
					id: payload.trade_id,
					session_id: payload.session_id,
					from_team_id: payload.from_team_id,
					to_team_id: payload.to_team_id,
					status: 'Proposed',
					from_team_value: payload.from_team_value,
					to_team_value: payload.to_team_value,
					value_difference: Math.abs(payload.from_team_value - payload.to_team_value),
				},
				from_team_picks: payload.from_team_picks,
				to_team_picks: payload.to_team_picks,
			},
			...this.proposals,
		];
	}

	onTradeExecuted(tradeId: string): void {
		this.updateStatus(tradeId, 'Accepted');
	}

	onTradeRejected(tradeId: string): void {
		this.updateStatus(tradeId, 'Rejected');
	}

	reset(): void {
		this.proposals = [];
		this.isLoading = false;
		this.error = null;
		this.sessionId = null;
	}

	private upsertProposal(proposal: TradeProposal): void {
		const idx = this.proposals.findIndex((p) => p.trade.id === proposal.trade.id);
		if (idx === -1) {
			this.proposals = [proposal, ...this.proposals];
		} else {
			this.proposals[idx] = proposal;
		}
	}

	private updateStatus(tradeId: string, status: string): void {
		const idx = this.proposals.findIndex((p) => p.trade.id === tradeId);
		if (idx === -1) return;
		this.proposals[idx] = {
			...this.proposals[idx],
			trade: { ...this.proposals[idx].trade, status },
		};
	}
}

export const tradesState = new TradesState();
