<script lang="ts">
	import { TradeProposalCard } from '$components/trade';
	import { LoadingSpinner } from '$components/ui';
	import type { TradeProposal, TradeStatus } from '$types';

	interface Props {
		proposals: TradeProposal[];
		isLoading?: boolean;
		currentTeamIds?: string[];
		onRespond?: (tradeId: string, teamId: string, action: 'accept' | 'reject') => Promise<void>;
	}

	let { proposals, isLoading = false, currentTeamIds = [], onRespond }: Props = $props();

	let selectedStatus = $state<TradeStatus | 'all'>('all');

	const filteredTrades = $derived(
		selectedStatus === 'all'
			? proposals
			: proposals.filter((t) => t.trade.status === selectedStatus)
	);

	const statusOptions: Array<{ value: TradeStatus | 'all'; label: string }> = [
		{ value: 'all', label: 'All' },
		{ value: 'Proposed', label: 'Proposed' },
		{ value: 'Accepted', label: 'Accepted' },
		{ value: 'Rejected', label: 'Rejected' },
	];

	function respondingTeamFor(proposal: TradeProposal): string | undefined {
		return currentTeamIds.find((id) => id === proposal.trade.to_team_id);
	}
</script>

<div class="bg-gray-50 rounded-lg p-6">
	<h2 class="text-2xl font-bold text-gray-900 mb-6">Trade History</h2>

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

	<div class="mb-4">
		<p class="text-sm text-gray-600">
			{filteredTrades.length}
			{filteredTrades.length === 1 ? 'trade' : 'trades'}
		</p>
	</div>

	{#if isLoading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if filteredTrades.length === 0}
		<p class="text-center text-gray-500 py-12">No trades yet</p>
	{:else}
		<div class="space-y-4">
			{#each filteredTrades as proposal (proposal.trade.id)}
				<TradeProposalCard {proposal} currentTeamId={respondingTeamFor(proposal)} {onRespond} />
			{/each}
		</div>
	{/if}
</div>
