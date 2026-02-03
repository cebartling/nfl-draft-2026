<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { draftsApi } from '$lib/api';
	import DraftBoard from '$components/draft/DraftBoard.svelte';
	import Card from '$components/ui/Card.svelte';
	import Badge from '$components/ui/Badge.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import type { Draft, DraftPick } from '$lib/types';

	let draftId = $derived($page.params.id!);
	let draft = $state<Draft | null>(null);
	let picks = $state<DraftPick[]>([]);
	let loading = $state(true);
	let picksLoading = $state(true);
	let error = $state<string | null>(null);

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

	async function handleCreateSession() {
		if (!draft) return;
		// Navigate to session - the session layout will handle creation
		await goto(`/sessions/${draft.id}`);
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
		<!-- Draft Header -->
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-start justify-between mb-4">
				<div>
					<h1 class="text-3xl font-bold text-gray-800 mb-2">
						{draft.year} NFL Draft
					</h1>
					<div class="flex items-center gap-2">
						<Badge variant={getStatusVariant(draft.status)}>
							{draft.status}
						</Badge>
					</div>
				</div>
				<div class="flex gap-2">
					{#if draft.status === 'NotStarted'}
						<button
							type="button"
							onclick={handleCreateSession}
							class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
						>
							Start Draft
						</button>
					{:else if draft.status === 'InProgress'}
						<button
							type="button"
							onclick={handleCreateSession}
							class="bg-green-600 hover:bg-green-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
						>
							Join Session
						</button>
					{/if}
				</div>
			</div>

			<!-- Draft Details Grid -->
			<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
				<div>
					<div class="text-sm text-gray-600">Year</div>
					<div class="text-lg font-semibold text-gray-800">{draft.year}</div>
				</div>
				<div>
					<div class="text-sm text-gray-600">Rounds</div>
					<div class="text-lg font-semibold text-gray-800">{draft.rounds}</div>
				</div>
				<div>
					<div class="text-sm text-gray-600">Picks per Round</div>
					<div class="text-lg font-semibold text-gray-800">{draft.picks_per_round}</div>
				</div>
				<div>
					<div class="text-sm text-gray-600">Total Picks</div>
					<div class="text-lg font-semibold text-gray-800">
						{draft.rounds * draft.picks_per_round}
					</div>
				</div>
			</div>

			<div class="mt-4 pt-4 border-t border-gray-200">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
					<div>
						<span class="text-gray-600">Created:</span>
						<span class="font-medium text-gray-800 ml-2">
							{new Date(draft.created_at).toLocaleString()}
						</span>
					</div>
					{#if draft.updated_at}
						<div>
							<span class="text-gray-600">Last Updated:</span>
							<span class="font-medium text-gray-800 ml-2">
								{new Date(draft.updated_at).toLocaleString()}
							</span>
						</div>
					{/if}
				</div>
			</div>
		</div>

		<!-- Draft Progress -->
		{#if picks.length > 0}
			<Card>
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<h2 class="text-xl font-bold text-gray-800">Draft Progress</h2>
						<span class="text-sm text-gray-600">
							{picks.length} / {draft.rounds * draft.picks_per_round} picks made
						</span>
					</div>
					<div class="w-full bg-gray-200 rounded-full h-2">
						<div
							class="bg-blue-600 h-2 rounded-full transition-all"
							style={`width: ${(picks.length / (draft.rounds * draft.picks_per_round)) * 100}%`}
						></div>
					</div>
				</div>
			</Card>
		{/if}

		<!-- Draft Board -->
		<div class="bg-white rounded-lg shadow p-6">
			<h2 class="text-2xl font-bold text-gray-800 mb-4">Draft Board</h2>
			{#if picksLoading}
				<div class="flex justify-center py-8">
					<LoadingSpinner />
				</div>
			{:else if picks.length === 0}
				<div class="text-center py-8 text-gray-600">
					<p>No picks have been made yet.</p>
					{#if draft.status === 'NotStarted'}
						<p class="text-sm mt-2">Start the draft to begin making picks.</p>
					{/if}
				</div>
			{:else}
				<DraftBoard {picks} />
			{/if}
		</div>

		<!-- Draft Statistics -->
		{#if picks.length > 0}
			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-blue-600">
							{Math.ceil(picks.length / draft.picks_per_round)}
						</div>
						<div class="text-sm text-gray-600 mt-1">Rounds Completed</div>
					</div>
				</Card>
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-green-600">
							{picks.length}
						</div>
						<div class="text-sm text-gray-600 mt-1">Picks Made</div>
					</div>
				</Card>
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-gray-600">
							{draft.rounds * draft.picks_per_round - picks.length}
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
