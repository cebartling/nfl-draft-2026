<script lang="ts">
	import { Badge, Button, LoadingSpinner } from '$components/ui';
	import { teamsApi, tradesApi } from '$api';
	import { toastState } from '$stores';
	import type { TradeProposal, Team } from '$types';
	import dayjs from 'dayjs';

	interface Props {
		proposal: TradeProposal;
		currentTeamId?: string;
		onUpdate?: () => void;
	}

	let { proposal, currentTeamId, onUpdate }: Props = $props();

	let fromTeam = $state<Team | null>(null);
	let toTeam = $state<Team | null>(null);
	let isLoading = $state(false);
	let isAccepting = $state(false);
	let isRejecting = $state(false);

	const canRespond = $derived(
		proposal.trade.status === 'Proposed' &&
			currentTeamId &&
			currentTeamId === proposal.trade.to_team_id
	);

	const valueDifference = $derived(
		Math.abs(proposal.from_team_total_value - proposal.to_team_total_value)
	);

	const isFairTrade = $derived(valueDifference < 100);

	// Load team data
	$effect(() => {
		isLoading = true;
		Promise.all([
			teamsApi.get(proposal.trade.from_team_id),
			teamsApi.get(proposal.trade.to_team_id),
		])
			.then(([from, to]) => {
				fromTeam = from;
				toTeam = to;
			})
			.catch((err) => {
				console.error('Failed to load teams:', err);
			})
			.finally(() => {
				isLoading = false;
			});
	});

	async function handleAccept() {
		if (!currentTeamId) return;

		isAccepting = true;

		try {
			await tradesApi.accept(proposal.trade.id, currentTeamId);
			toastState.success('Trade accepted');
			onUpdate?.();
		} catch (err) {
			toastState.error('Failed to accept trade');
			console.error('Failed to accept trade:', err);
		} finally {
			isAccepting = false;
		}
	}

	async function handleReject() {
		if (!currentTeamId) return;

		isRejecting = true;

		try {
			await tradesApi.reject(proposal.trade.id, currentTeamId);
			toastState.success('Trade rejected');
			onUpdate?.();
		} catch (err) {
			toastState.error('Failed to reject trade');
			console.error('Failed to reject trade:', err);
		} finally {
			isRejecting = false;
		}
	}

	function getStatusBadge() {
		const status = proposal.trade.status;
		switch (status) {
			case 'Proposed':
				return { variant: 'warning' as const, text: 'Proposed' };
			case 'Accepted':
				return { variant: 'success' as const, text: 'Accepted' };
			case 'Rejected':
				return { variant: 'danger' as const, text: 'Rejected' };
			case 'Expired':
				return { variant: 'default' as const, text: 'Expired' };
			default:
				return { variant: 'default' as const, text: 'Unknown' };
		}
	}

	const statusBadge = $derived(getStatusBadge());
</script>

<div class="bg-white rounded-lg shadow-md p-6">
	<div class="flex items-center justify-between mb-4">
		<h3 class="text-lg font-semibold text-gray-900">Trade Proposal</h3>
		<Badge variant={statusBadge.variant} size="lg">
			{statusBadge.text}
		</Badge>
	</div>

	{#if isLoading}
		<div class="flex justify-center py-8">
			<LoadingSpinner size="lg" />
		</div>
	{:else if fromTeam && toTeam}
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
			<!-- From Team -->
			<div class="space-y-3">
				<div class="text-center">
					<p class="text-sm font-medium text-gray-600 mb-2">From</p>
					<p class="text-lg font-bold text-gray-900">
						{fromTeam.city} {fromTeam.name}
					</p>
					<p class="text-sm text-gray-600">{fromTeam.abbreviation}</p>
				</div>
				<div class="border border-gray-200 rounded-lg p-3">
					<p class="text-xs font-medium text-gray-600 mb-2">Picks ({proposal.from_team_picks.length})</p>
					<div class="space-y-1">
						{#each proposal.from_team_picks as pick}
							<div class="text-sm text-gray-900">
								Round {pick.pick_id.slice(0, 8)}...
								<span class="text-xs text-gray-500">(Value: {pick.pick_value})</span>
							</div>
						{/each}
					</div>
					<div class="mt-3 pt-3 border-t border-gray-200">
						<p class="text-sm font-semibold text-gray-900">
							Total Value: {proposal.from_team_total_value}
						</p>
					</div>
				</div>
			</div>

			<!-- Arrow -->
			<div class="flex items-center justify-center">
				<svg
					class="w-8 h-8 text-gray-400"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M14 5l7 7m0 0l-7 7m7-7H3"
					/>
				</svg>
			</div>

			<!-- To Team -->
			<div class="space-y-3">
				<div class="text-center">
					<p class="text-sm font-medium text-gray-600 mb-2">To</p>
					<p class="text-lg font-bold text-gray-900">
						{toTeam.city} {toTeam.name}
					</p>
					<p class="text-sm text-gray-600">{toTeam.abbreviation}</p>
				</div>
				<div class="border border-gray-200 rounded-lg p-3">
					<p class="text-xs font-medium text-gray-600 mb-2">Picks ({proposal.to_team_picks.length})</p>
					<div class="space-y-1">
						{#each proposal.to_team_picks as pick}
							<div class="text-sm text-gray-900">
								Round {pick.pick_id.slice(0, 8)}...
								<span class="text-xs text-gray-500">(Value: {pick.pick_value})</span>
							</div>
						{/each}
					</div>
					<div class="mt-3 pt-3 border-t border-gray-200">
						<p class="text-sm font-semibold text-gray-900">
							Total Value: {proposal.to_team_total_value}
						</p>
					</div>
				</div>
			</div>
		</div>

		<!-- Value Difference Indicator -->
		<div class="mb-6">
			<div class="flex items-center justify-between mb-2">
				<p class="text-sm font-medium text-gray-600">Value Difference</p>
				<Badge variant={isFairTrade ? 'success' : 'warning'}>
					{valueDifference} points
				</Badge>
			</div>
			<p class="text-xs text-gray-500">
				{isFairTrade
					? 'This is a fair trade (difference < 100 points)'
					: 'This trade may be unbalanced (difference >= 100 points)'}
			</p>
		</div>

		<!-- Trade Details -->
		<div class="text-xs text-gray-500 mb-4">
			Proposed at {dayjs(proposal.trade.proposed_at).format('MMM D, YYYY h:mm A')}
			{#if proposal.trade.resolved_at}
				<br />
				Resolved at {dayjs(proposal.trade.resolved_at).format('MMM D, YYYY h:mm A')}
			{/if}
		</div>

		<!-- Action Buttons -->
		{#if canRespond}
			<div class="flex space-x-3">
				<Button
					variant="primary"
					onclick={handleAccept}
					disabled={isAccepting || isRejecting}
					loading={isAccepting}
				>
					Accept Trade
				</Button>
				<Button
					variant="danger"
					onclick={handleReject}
					disabled={isAccepting || isRejecting}
					loading={isRejecting}
				>
					Reject Trade
				</Button>
			</div>
		{/if}
	{/if}
</div>
