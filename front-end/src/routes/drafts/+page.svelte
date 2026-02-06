<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { draftsApi } from '$lib/api';
	import Card from '$components/ui/Card.svelte';
	import Badge from '$components/ui/Badge.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import type { Draft } from '$lib/types';

	let drafts = $state<Draft[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let filterStatus = $state<string>('all');

	onMount(async () => {
		try {
			drafts = await draftsApi.list();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load drafts';
			logger.error('Failed to load drafts:', e);
		} finally {
			loading = false;
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

	let filteredDrafts = $derived(() => {
		if (filterStatus === 'all') {
			return drafts;
		}
		return drafts.filter((d) => d.status === filterStatus);
	});

	$effect(() => {
		// Sort drafts by year (most recent first)
		drafts.sort((a, b) => b.year - a.year);
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold text-gray-800">Drafts</h1>
		<button
			type="button"
			onclick={async () => {
				await goto('/drafts/new');
			}}
			class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
		>
			Create New Draft
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if error}
		<Card>
			<div class="text-red-600">
				<p class="font-semibold">Error loading drafts</p>
				<p class="text-sm">{error}</p>
			</div>
		</Card>
	{:else}
		<!-- Filters -->
		<div class="bg-white rounded-lg shadow p-4">
			<div class="flex items-center gap-4">
				<span class="text-sm font-medium text-gray-700">Filter by status:</span>
				<div class="flex gap-2">
					<button
						type="button"
						onclick={() => (filterStatus = 'all')}
						class={`px-4 py-2 rounded-lg font-medium transition-colors ${
							filterStatus === 'all'
								? 'bg-blue-600 text-white'
								: 'bg-gray-200 text-gray-700 hover:bg-gray-300'
						}`}
					>
						All ({drafts.length})
					</button>
					<button
						type="button"
						onclick={() => (filterStatus = 'pending')}
						class={`px-4 py-2 rounded-lg font-medium transition-colors ${
							filterStatus === 'pending'
								? 'bg-blue-600 text-white'
								: 'bg-gray-200 text-gray-700 hover:bg-gray-300'
						}`}
					>
						Pending ({drafts.filter((d) => d.status === 'NotStarted').length})
					</button>
					<button
						type="button"
						onclick={() => (filterStatus = 'active')}
						class={`px-4 py-2 rounded-lg font-medium transition-colors ${
							filterStatus === 'active'
								? 'bg-green-600 text-white'
								: 'bg-gray-200 text-gray-700 hover:bg-gray-300'
						}`}
					>
						Active ({drafts.filter((d) => d.status === 'InProgress').length})
					</button>
					<button
						type="button"
						onclick={() => (filterStatus = 'completed')}
						class={`px-4 py-2 rounded-lg font-medium transition-colors ${
							filterStatus === 'completed'
								? 'bg-gray-600 text-white'
								: 'bg-gray-200 text-gray-700 hover:bg-gray-300'
						}`}
					>
						Completed ({drafts.filter((d) => d.status === 'Completed').length})
					</button>
				</div>
			</div>
		</div>

		<!-- Drafts List -->
		{#if filteredDrafts().length === 0}
			<Card>
				<div class="text-center py-8">
					<p class="text-gray-600 mb-4">
						{filterStatus === 'all'
							? 'No drafts yet. Create your first draft to get started!'
							: `No ${filterStatus} drafts found.`}
					</p>
					{#if filterStatus === 'all'}
						<button
							type="button"
							onclick={async () => {
								await goto('/drafts/new');
							}}
							class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
						>
							Create Draft
						</button>
					{:else}
						<button
							type="button"
							onclick={() => (filterStatus = 'all')}
							class="text-blue-600 hover:text-blue-700 font-medium"
						>
							View all drafts
						</button>
					{/if}
				</div>
			</Card>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each filteredDrafts() as draft (draft.id)}
					<Card
						clickable
						onclick={async () => {
							await goto(`/drafts/${draft.id}`);
						}}
					>
						<div class="space-y-3">
							<div class="flex items-start justify-between">
								<h3 class="text-xl font-semibold text-gray-800">
									{draft.year} Draft
								</h3>
								<Badge variant={getStatusVariant(draft.status)}>
									{draft.status}
								</Badge>
							</div>

							<div class="space-y-2 text-sm text-gray-600">
								<div class="flex items-center justify-between">
									<span>Rounds:</span>
									<span class="font-medium">{draft.rounds}</span>
								</div>
								<div class="flex items-center justify-between">
									<span>Picks per round:</span>
									<span class="font-medium">
										{draft.picks_per_round != null ? draft.picks_per_round : 'Variable'}
									</span>
								</div>
								<div class="flex items-center justify-between">
									<span>Total picks:</span>
									<span class="font-medium">
										{draft.total_picks ?? (draft.picks_per_round != null ? draft.rounds * draft.picks_per_round : 'N/A')}
									</span>
								</div>
								{#if draft.created_at}
								<div class="flex items-center justify-between">
									<span>Created:</span>
									<span class="font-medium">
										{new Date(draft.created_at).toLocaleDateString()}
									</span>
								</div>
							{/if}
								{#if draft.updated_at}
									<div class="flex items-center justify-between">
										<span>Updated:</span>
										<span class="font-medium">
											{new Date(draft.updated_at).toLocaleDateString()}
										</span>
									</div>
								{/if}
							</div>

							<div class="pt-3 border-t border-gray-200 space-y-2">
								{#if draft.status === 'InProgress'}
									<button
										type="button"
										onclick={async (e) => {
											e.stopPropagation();
											await goto(`/sessions/${draft.id}`);
										}}
										class="w-full bg-green-600 hover:bg-green-700 text-white font-medium py-2 px-4 rounded transition-colors"
									>
										Join Session
									</button>
								{:else if draft.status === 'NotStarted'}
									<button
										type="button"
										onclick={async (e) => {
											e.stopPropagation();
											await goto(`/sessions/${draft.id}`);
										}}
										class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors"
									>
										Start Draft
									</button>
								{:else}
									<button
										type="button"
										onclick={async (e) => {
											e.stopPropagation();
											await goto(`/drafts/${draft.id}`);
										}}
										class="w-full bg-gray-600 hover:bg-gray-700 text-white font-medium py-2 px-4 rounded transition-colors"
									>
										View Results
									</button>
								{/if}
							</div>
						</div>
					</Card>
				{/each}
			</div>
		{/if}
	{/if}
</div>
