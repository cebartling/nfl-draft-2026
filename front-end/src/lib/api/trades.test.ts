import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { tradesApi } from './trades';
import * as client from './client';
import type { Trade, TradeProposal } from '$lib/types';

describe('tradesApi', () => {
	let mockGet: ReturnType<typeof vi.fn>;
	let mockPost: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockGet = vi.fn();
		mockPost = vi.fn();

		vi.spyOn(client.apiClient, 'get').mockImplementation(mockGet as any);
		vi.spyOn(client.apiClient, 'post').mockImplementation(mockPost as any);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	function makeTrade(overrides: Partial<Trade> = {}): Trade {
		return {
			id: 'trade-1',
			session_id: 'session-1',
			from_team_id: 'team-a',
			to_team_id: 'team-b',
			status: 'Proposed',
			from_team_value: 3000,
			to_team_value: 2600,
			value_difference: 400,
			...overrides,
		};
	}

	function makeProposal(overrides: Partial<TradeProposal> = {}): TradeProposal {
		return {
			trade: makeTrade(),
			from_team_picks: ['pick-1'],
			to_team_picks: ['pick-2'],
			...overrides,
		};
	}

	describe('propose', () => {
		it('should send POST /trades with params', async () => {
			const params = {
				session_id: 'session-1',
				from_team_id: 'team-a',
				to_team_id: 'team-b',
				from_team_pick_ids: ['pick-1'],
				to_team_pick_ids: ['pick-2'],
			};
			const mockProposal = makeProposal();
			mockPost.mockResolvedValueOnce(mockProposal);

			const result = await tradesApi.propose(params);

			expect(mockPost).toHaveBeenCalledWith('/trades', params, expect.any(Object));
			expect(result).toEqual(mockProposal);
		});

		it('should propagate errors for unfair trade', async () => {
			mockPost.mockRejectedValueOnce(new Error('Trade is not fair'));

			await expect(
				tradesApi.propose({
					session_id: 'session-1',
					from_team_id: 'team-a',
					to_team_id: 'team-b',
					from_team_pick_ids: ['pick-1'],
					to_team_pick_ids: ['pick-32'],
				})
			).rejects.toThrow('Trade is not fair');
		});
	});

	describe('accept', () => {
		it('should send POST /trades/{id}/accept with team_id', async () => {
			const mockAccepted = makeTrade({ status: 'Accepted' });
			mockPost.mockResolvedValueOnce(mockAccepted);

			const result = await tradesApi.accept('trade-1', 'team-b');

			expect(mockPost).toHaveBeenCalledWith(
				'/trades/trade-1/accept',
				{ team_id: 'team-b' },
				expect.any(Object)
			);
			expect(result.status).toBe('Accepted');
		});

		it('should propagate errors', async () => {
			mockPost.mockRejectedValueOnce(new Error('Only the receiving team can accept'));

			await expect(tradesApi.accept('trade-1', 'wrong-team')).rejects.toThrow(
				'Only the receiving team can accept'
			);
		});
	});

	describe('reject', () => {
		it('should send POST /trades/{id}/reject with team_id', async () => {
			const mockRejected = makeTrade({ status: 'Rejected' });
			mockPost.mockResolvedValueOnce(mockRejected);

			const result = await tradesApi.reject('trade-1', 'team-b');

			expect(mockPost).toHaveBeenCalledWith(
				'/trades/trade-1/reject',
				{ team_id: 'team-b' },
				expect.any(Object)
			);
			expect(result.status).toBe('Rejected');
		});

		it('should propagate errors', async () => {
			mockPost.mockRejectedValueOnce(new Error('Only the receiving team can reject'));

			await expect(tradesApi.reject('trade-1', 'wrong-team')).rejects.toThrow(
				'Only the receiving team can reject'
			);
		});
	});

	describe('getBySession', () => {
		it('should send GET /sessions/{id}/trades', async () => {
			const mockProposals = [makeProposal()];
			mockGet.mockResolvedValueOnce(mockProposals);

			const result = await tradesApi.getBySession('session-1');

			expect(mockGet).toHaveBeenCalledWith(
				'/sessions/session-1/trades',
				expect.any(Object)
			);
			expect(result).toEqual(mockProposals);
		});

		it('should return empty array when no trades', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await tradesApi.getBySession('session-1');

			expect(result).toEqual([]);
		});
	});
});
