<script lang="ts">
	import { TradeProposalCard } from '$components/trade';
	import { LoadingSpinner, Badge } from '$components/ui';
	import { tradesApi } from '$api';
	import { toastState } from '$stores';
	import { logger } from '$lib/utils/logger';
	import type { TradeProposal, TradeStatus } from '$types';

	interface Props {
		sessionId: string;
		currentTeamId?: string;
	}

	let { sessionId, currentTeamId }: Props = $props();

	let trades = $state<TradeProposal[]>([]);
	let isLoading = $state(false);
	let selectedStatus = $state<TradeStatus | 'all'>('all');

	const filteredTrades = $derived(
		selectedStatus === 'all' ? trades : trades.filter((t) => t.trade.status === selectedStatus)
	);

	// Load trades
	function loadTrades() {
		isLoading = true;
		tradesApi
			.getBySession(sessionId)
			.then((data) => {
				trades = data;
			})
			.catch((err) => {
				logger.error('Failed to load trades:', err);
				toastState.error('Failed to load trades');
			})
			.finally(() => {
				isLoading = false;
			});
	}

	// Load on mount
	$effect(() => {
		if (sessionId) {
			loadTrades();
		}
	});

	const statusOptions: Array<{ value: TradeStatus | 'all'; label: string }> = [
		{ value: 'all', label: 'All' },
		{ value: 'Proposed', label: 'Proposed' },
		{ value: 'Accepted', label: 'Accepted' },
		{ value: 'Rejected', label: 'Rejected' },
		{ value: 'Expired', label: 'Expired' },
	];
</script>

<div class="bg-gray-50 rounded-lg p-6">
	<h2 class="text-2xl font-bold text-gray-900 mb-6">Trade History</h2>

	<!-- Status Filter -->
	<div class="mb-6">
		<p class="text-sm font-medium text-gray-700 mb-3">Filter by Status</p>
		<div class="flex flex-wrap gap-2">
			{#each statusOptions as option (option.value)}
				<button
					type="button"
					class="px-4 py-2 rounded-lg text-sm font-medium transition-colors {selectedStatus ===
					option.value
						? 'bg-blue-600 text-white'
						: 'bg-white text-gray-700 hover:bg-gray-100'}"
					onclick={() => (selectedStatus = option.value)}
				>
					{option.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Results Count -->
	<div class="mb-4">
		<p class="text-sm text-gray-600">
			{filteredTrades.length}
			{filteredTrades.length === 1 ? 'trade' : 'trades'}
		</p>
	</div>

	<!-- Trade List -->
	{#if isLoading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if filteredTrades.length === 0}
		<p class="text-center text-gray-500 py-12">No trades found</p>
	{:else}
		<div class="space-y-4">
			{#each filteredTrades as proposal (proposal.trade.id)}
				<TradeProposalCard {proposal} {currentTeamId} onUpdate={loadTrades} />
			{/each}
		</div>
	{/if}
</div>
