<script lang="ts">
	import { Button, Badge, LoadingSpinner } from '$components/ui';
	import { teamsApi, tradesApi } from '$api';
	import { toastState } from '$stores';
	import { logger } from '$lib/utils/logger';
	import type { Team, DraftPick } from '$types';

	interface Props {
		sessionId: string;
		availablePicks: DraftPick[];
		onSuccess?: () => void;
	}

	let { sessionId, availablePicks, onSuccess }: Props = $props();

	let teams = $state<Team[]>([]);
	let fromTeamId = $state<string>('');
	let toTeamId = $state<string>('');
	let fromTeamPickIds = $state<string[]>([]);
	let toTeamPickIds = $state<string[]>([]);
	let isLoadingTeams = $state(false);
	let isSubmitting = $state(false);

	const fromTeamPicks = $derived(
		availablePicks.filter((p) => p.current_team_id === fromTeamId)
	);

	const toTeamPicks = $derived(
		availablePicks.filter((p) => p.current_team_id === toTeamId)
	);

	// Load teams
	$effect(() => {
		isLoadingTeams = true;
		teamsApi
			.list()
			.then((data) => {
				teams = data;
			})
			.catch((err) => {
				logger.error('Failed to load teams:', err);
				toastState.error('Failed to load teams');
			})
			.finally(() => {
				isLoadingTeams = false;
			});
	});

	function togglePick(pickId: string, side: 'from' | 'to') {
		if (side === 'from') {
			if (fromTeamPickIds.includes(pickId)) {
				fromTeamPickIds = fromTeamPickIds.filter((id) => id !== pickId);
			} else {
				fromTeamPickIds = [...fromTeamPickIds, pickId];
			}
		} else {
			if (toTeamPickIds.includes(pickId)) {
				toTeamPickIds = toTeamPickIds.filter((id) => id !== pickId);
			} else {
				toTeamPickIds = [...toTeamPickIds, pickId];
			}
		}
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!fromTeamId || !toTeamId) {
			toastState.error('Please select both teams');
			return;
		}

		if (fromTeamId === toTeamId) {
			toastState.error('Teams must be different');
			return;
		}

		if (fromTeamPickIds.length === 0 && toTeamPickIds.length === 0) {
			toastState.error('Please select at least one pick');
			return;
		}

		isSubmitting = true;

		try {
			await tradesApi.propose({
				session_id: sessionId,
				from_team_id: fromTeamId,
				to_team_id: toTeamId,
				from_team_pick_ids: fromTeamPickIds,
				to_team_pick_ids: toTeamPickIds,
			});

			toastState.success('Trade proposal created');

			// Reset form
			fromTeamId = '';
			toTeamId = '';
			fromTeamPickIds = [];
			toTeamPickIds = [];

			onSuccess?.();
		} catch (err) {
			toastState.error('Failed to create trade proposal');
			logger.error('Failed to create trade proposal:', err);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="bg-white rounded-lg shadow-md p-6">
	<h2 class="text-xl font-semibold text-gray-900 mb-6">Build Trade Proposal</h2>

	{#if isLoadingTeams}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else}
		<form onsubmit={handleSubmit} class="space-y-6">
			<!-- Team Selection -->
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<div>
					<label for="from-team" class="block text-sm font-medium text-gray-700 mb-2">
						From Team
					</label>
					<select
						id="from-team"
						bind:value={fromTeamId}
						class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
						required
					>
						<option value="">Select a team</option>
						{#each teams as team (team.id)}
							<option value={team.id}>
								{team.city} {team.name}
							</option>
						{/each}
					</select>
				</div>

				<div>
					<label for="to-team" class="block text-sm font-medium text-gray-700 mb-2">
						To Team
					</label>
					<select
						id="to-team"
						bind:value={toTeamId}
						class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
						required
					>
						<option value="">Select a team</option>
						{#each teams as team (team.id)}
							<option value={team.id}>
								{team.city} {team.name}
							</option>
						{/each}
					</select>
				</div>
			</div>

			<!-- Pick Selection -->
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<!-- From Team Picks -->
				<div>
					<p class="text-sm font-medium text-gray-700 mb-3">
						From Team Picks ({fromTeamPickIds.length} selected)
					</p>
					<div class="border border-gray-200 rounded-lg max-h-64 overflow-y-auto">
						{#if fromTeamId && fromTeamPicks.length > 0}
							<div class="divide-y divide-gray-200">
								{#each fromTeamPicks as pick (pick.id)}
									<button
										type="button"
										aria-label="Select pick Round {pick.round} Pick {pick.pick_number}"
										class="w-full p-3 text-left hover:bg-gray-50 transition-colors {fromTeamPickIds.includes(
											pick.id
										)
											? 'bg-blue-50'
											: ''}"
										onclick={() => togglePick(pick.id, 'from')}
									>
										<div class="flex items-center justify-between">
											<div>
												<div class="flex items-center space-x-2">
													<Badge variant="primary" size="sm">
														Round {pick.round}
													</Badge>
													<Badge variant="default" size="sm">
														Pick {pick.pick_number}
													</Badge>
												</div>
												<p class="text-xs text-gray-500 mt-1">
													Overall: #{pick.overall_pick}
												</p>
											</div>
											{#if fromTeamPickIds.includes(pick.id)}
												<svg
													class="w-5 h-5 text-blue-600"
													fill="currentColor"
													viewBox="0 0 20 20"
												>
													<path
														fill-rule="evenodd"
														d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
														clip-rule="evenodd"
													/>
												</svg>
											{/if}
										</div>
									</button>
								{/each}
							</div>
						{:else}
							<p class="text-center text-gray-500 py-8 text-sm">
								{fromTeamId ? 'No available picks' : 'Select a team'}
							</p>
						{/if}
					</div>
				</div>

				<!-- To Team Picks -->
				<div>
					<p class="text-sm font-medium text-gray-700 mb-3">
						To Team Picks ({toTeamPickIds.length} selected)
					</p>
					<div class="border border-gray-200 rounded-lg max-h-64 overflow-y-auto">
						{#if toTeamId && toTeamPicks.length > 0}
							<div class="divide-y divide-gray-200">
								{#each toTeamPicks as pick (pick.id)}
									<button
										type="button"
										aria-label="Select pick Round {pick.round} Pick {pick.pick_number}"
										class="w-full p-3 text-left hover:bg-gray-50 transition-colors {toTeamPickIds.includes(
											pick.id
										)
											? 'bg-blue-50'
											: ''}"
										onclick={() => togglePick(pick.id, 'to')}
									>
										<div class="flex items-center justify-between">
											<div>
												<div class="flex items-center space-x-2">
													<Badge variant="primary" size="sm">
														Round {pick.round}
													</Badge>
													<Badge variant="default" size="sm">
														Pick {pick.pick_number}
													</Badge>
												</div>
												<p class="text-xs text-gray-500 mt-1">
													Overall: #{pick.overall_pick}
												</p>
											</div>
											{#if toTeamPickIds.includes(pick.id)}
												<svg
													class="w-5 h-5 text-blue-600"
													fill="currentColor"
													viewBox="0 0 20 20"
												>
													<path
														fill-rule="evenodd"
														d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
														clip-rule="evenodd"
													/>
												</svg>
											{/if}
										</div>
									</button>
								{/each}
							</div>
						{:else}
							<p class="text-center text-gray-500 py-8 text-sm">
								{toTeamId ? 'No available picks' : 'Select a team'}
							</p>
						{/if}
					</div>
				</div>
			</div>

			<!-- Submit Button -->
			<div class="flex justify-end">
				<Button
					type="submit"
					variant="primary"
					disabled={isSubmitting}
					loading={isSubmitting}
				>
					Propose Trade
				</Button>
			</div>
		</form>
	{/if}
</div>
