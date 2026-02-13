<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { draftsApi, sessionsApi, teamsApi } from '$lib/api';
	import { ApiClientError } from '$lib/api/client';
	import DraftBoard from '$components/draft/DraftBoard.svelte';
	import TeamSelector from '$components/draft/TeamSelector.svelte';
	import Card from '$components/ui/Card.svelte';
	import Badge from '$components/ui/Badge.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import type { Draft, DraftPick, Team } from '$lib/types';

	let draftId = $derived($page.params.id!);
	let draft = $state<Draft | null>(null);
	let picks = $state<DraftPick[]>([]);
	let loading = $state(true);
	let picksLoading = $state(true);
	let error = $state<string | null>(null);
	// Team selector state
	let selectedTeamIds = $state<string[]>([]);
	let allTeams = $state<Team[]>([]);
	let teamsLoading = $state(false);

	// Count only picks that have been made (have a player assigned)
	let completedPicks = $derived(picks.filter((p) => p.player_id != null).length);
	let totalPicks = $derived(draft?.total_picks ?? picks.length);
	// Count rounds where all picks in that round have been completed
	let roundsCompleted = $derived(() => {
		if (completedPicks === 0) return 0;
		const roundPickCounts = new Map<number, { total: number; completed: number }>();
		for (const p of picks) {
			const entry = roundPickCounts.get(p.round) ?? { total: 0, completed: 0 };
			entry.total++;
			if (p.player_id != null) entry.completed++;
			roundPickCounts.set(p.round, entry);
		}
		let count = 0;
		for (const { total, completed } of roundPickCounts.values()) {
			if (completed === total) count++;
		}
		return count;
	});

	onMount(async () => {
		// Load draft details
		try {
			draft = await draftsApi.get(draftId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load draft';
			logger.error('Failed to load draft:', e);
		} finally {
			loading = false;
		}

		// Load draft picks
		try {
			picks = await draftsApi.getPicks(draftId);
		} catch (e) {
			logger.error('Failed to load picks:', e);
		} finally {
			picksLoading = false;
		}

		// Eagerly load teams for NotStarted drafts
		if (draft && draft.status === 'NotStarted') {
			teamsLoading = true;
			try {
				allTeams = await teamsApi.list();
			} catch (e) {
				logger.error('Failed to load teams:', e);
				error = e instanceof Error ? e.message : 'Failed to load teams';
			} finally {
				teamsLoading = false;
			}
		}
	});

	function getStatusVariant(
		status: string
	): 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info' {
		switch (status) {
			case 'NotStarted':
				return 'primary';
			case 'InProgress':
				return 'success';
			case 'Completed':
				return 'default';
			case 'Paused':
				return 'warning';
			default:
				return 'default';
		}
	}

	async function handleCreateSession(controlledTeamIds: string[] = []) {
		if (!draft) return;
		try {
			const session = await sessionsApi.create({
				draft_id: draft.id,
				time_per_pick_seconds: 120,
				auto_pick_enabled: true,
				chart_type: 'JimmyJohnson',
				controlled_team_ids: controlledTeamIds
			});
			await goto(`/sessions/${session.id}`);
		} catch (e) {
			if (e instanceof ApiClientError && e.status === 409) {
				// Session already exists — find it and redirect
				try {
					const existing = await sessionsApi.getByDraftId(draft.id);
					await goto(`/sessions/${existing.id}`);
				} catch (innerErr) {
					logger.error('Failed to find existing session:', innerErr);
					error = 'A session already exists for this draft but could not be loaded';
				}
			} else {
				logger.error('Failed to create session:', e);
				error = e instanceof Error ? e.message : 'Failed to create session';
			}
		}
	}

</script>

<div class="space-y-6">
	<!-- Back Button -->
	<div>
		<button
			type="button"
			onclick={async () => {
				await goto('/drafts');
			}}
			class="inline-flex items-center text-blue-600 hover:text-blue-700 font-medium"
		>
			<svg class="w-5 h-5 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Drafts
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if error}
		<div class="bg-white rounded-lg shadow p-8 text-center">
			<div class="text-red-600 mb-4">
				<svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
					/>
				</svg>
			</div>
			<h2 class="text-xl font-semibold text-gray-800 mb-2">Draft Not Found</h2>
			<p class="text-gray-600 mb-4">{error}</p>
			<button
				type="button"
				onclick={async () => {
					await goto('/drafts');
				}}
				class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
			>
				Back to Drafts
			</button>
		</div>
	{:else if draft}
		{#if draft.status === 'NotStarted'}
			<!-- Unified Setup Panel for NotStarted drafts -->
			<div class="bg-white rounded-lg shadow p-6 space-y-6">
				<!-- Draft Header -->
				<div class="flex items-start justify-between">
					<div>
						<h1 class="text-3xl font-bold text-gray-800 mb-2">
							{draft.name}
						</h1>
						<div class="flex items-center gap-4">
							<Badge variant={getStatusVariant(draft.status)}>
								{draft.status}
							</Badge>
							<span class="text-sm text-gray-600">
								Rounds: {draft.rounds}
							</span>
							<span class="text-sm text-gray-600">
								Pick Timer: 120s
							</span>
						</div>
					</div>
				</div>

				<!-- Divider + Team Selector -->
				<div class="border-t border-gray-200 pt-4">
					<h2 class="text-lg font-bold text-gray-800 mb-1">Select Teams to Control</h2>
					<p class="text-sm text-gray-600 mb-4">
						Choose teams you want to manually control. Unselected teams will be managed by AI.
					</p>

					{#if teamsLoading}
						<div class="flex justify-center py-8">
							<LoadingSpinner />
						</div>
					{:else}
						<TeamSelector
							teams={allTeams}
							{selectedTeamIds}
							onSelectionChange={(ids) => (selectedTeamIds = ids)}
						/>
					{/if}
				</div>

				<!-- Action Buttons -->
				<div class="flex flex-col sm:flex-row gap-3 pt-2 border-t border-gray-200">
					<button
						type="button"
						data-testid="start-with-teams"
						onclick={() => handleCreateSession(selectedTeamIds)}
						disabled={selectedTeamIds.length === 0}
						class="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-white font-semibold py-2.5 px-6 rounded-lg transition-colors"
					>
						Start with {selectedTeamIds.length} Team{selectedTeamIds.length !== 1 ? 's' : ''}
					</button>
					<button
						type="button"
						data-testid="auto-pick-all"
						onclick={() => handleCreateSession([])}
						class="flex-1 bg-gray-600 hover:bg-gray-700 text-white font-semibold py-2.5 px-6 rounded-lg transition-colors"
					>
						Auto-pick All Teams
					</button>
					<button
						type="button"
						data-testid="cancel-draft"
						onclick={async () => {
							await goto('/drafts');
						}}
						class="px-4 py-2.5 text-gray-600 hover:text-gray-800 font-medium transition-colors"
					>
						Cancel
					</button>
				</div>
			</div>
		{:else}
			<!-- Draft Header for non-NotStarted drafts -->
			<div class="bg-white rounded-lg shadow p-6">
				<div class="flex items-start justify-between mb-4">
					<div>
						<h1 class="text-3xl font-bold text-gray-800 mb-2">
							{draft.name}
						</h1>
						<div class="flex items-center gap-2">
							<Badge variant={getStatusVariant(draft.status)}>
								{draft.status}
							</Badge>
						</div>
					</div>
					<div class="flex gap-2">
						{#if draft.status === 'InProgress'}
							<button
								type="button"
								onclick={async () => {
									if (!draft) return;
									try {
										const session = await sessionsApi.getByDraftId(draft.id);
										await goto(`/sessions/${session.id}`);
									} catch (err) {
										logger.error('Failed to find session:', err);
										error = err instanceof Error ? err.message : 'Failed to find session';
									}
								}}
								class="bg-green-600 hover:bg-green-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
							>
								Join Session
							</button>
						{/if}
					</div>
				</div>

				<!-- Draft Details Grid -->
				<div class="grid grid-cols-2 md:grid-cols-3 gap-4">
					<div>
						<div class="text-sm text-gray-600">Year</div>
						<div class="text-lg font-semibold text-gray-800">{draft.year}</div>
					</div>
					<div>
						<div class="text-sm text-gray-600">Rounds</div>
						<div class="text-lg font-semibold text-gray-800">{draft.rounds}</div>
					</div>
					<div>
						<div class="text-sm text-gray-600">Total Picks</div>
						<div class="text-lg font-semibold text-gray-800">
							{totalPicks}
						</div>
					</div>
				</div>
			</div>
		{/if}

		<!-- Draft Progress -->
		{#if picks.length > 0}
			<Card>
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<h2 class="text-xl font-bold text-gray-800">Draft Progress</h2>
						<span class="text-sm text-gray-600">
							{completedPicks} / {totalPicks} picks made
						</span>
					</div>
					<div class="w-full bg-gray-200 rounded-full h-2">
						<div
							class="bg-blue-600 h-2 rounded-full transition-all"
							style={`width: ${totalPicks > 0 ? (completedPicks / totalPicks) * 100 : 0}%`}
						></div>
					</div>
					{#if picks.length > 0 && completedPicks === 0}
						<p class="text-xs text-gray-500 text-center">
							{picks.length} picks initialized, ready to start drafting
						</p>
					{/if}
				</div>
			</Card>
		{/if}

		<!-- Draft Board -->
		{#if picksLoading}
			<div class="flex justify-center py-8">
				<LoadingSpinner />
			</div>
		{:else if picks.length === 0}
			<div class="text-center py-8 text-gray-600">
				<p>No picks available for this draft.</p>
			</div>
		{:else}
			<DraftBoard {picks} />
		{/if}

		<!-- Draft Statistics -->
		{#if picks.length > 0}
			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-blue-600">
							{roundsCompleted()}
						</div>
						<div class="text-sm text-gray-600 mt-1">Rounds Completed</div>
					</div>
				</Card>
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-green-600">
							{completedPicks}
						</div>
						<div class="text-sm text-gray-600 mt-1">Picks Made</div>
					</div>
				</Card>
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-gray-600">
							{totalPicks - completedPicks}
						</div>
						<div class="text-sm text-gray-600 mt-1">Picks Remaining</div>
					</div>
				</Card>
			</div>
		{/if}
	{:else}
		<div class="bg-white rounded-lg shadow p-8 text-center">
			<p class="text-gray-600">Draft not found.</p>
		</div>
	{/if}
</div>
