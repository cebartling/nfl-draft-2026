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

	$effect(() => {
		// Sort drafts by year (most recent first), then by created date if available
		drafts.sort((a, b) => {
			if (a.year !== b.year) return b.year - a.year;
			const dateA = a.created_at ? new Date(a.created_at).getTime() : 0;
			const dateB = b.created_at ? new Date(b.created_at).getTime() : 0;
			return dateB - dateA;
		});
	});
</script>

<div class="space-y-8">
	<!-- Hero Section -->
	<div class="bg-gradient-to-r from-blue-600 to-blue-800 rounded-lg shadow-lg p-8 text-white">
		<h1 class="text-4xl font-bold mb-4">NFL Draft Simulator 2026</h1>
		<p class="text-xl mb-6">
			Experience the thrill of the NFL Draft with real-time simulations, AI-driven team
			decision-making, and comprehensive scouting systems.
		</p>
		<button
			type="button"
			onclick={async () => {
				await goto('/drafts/new');
			}}
			class="bg-white text-blue-600 hover:bg-gray-100 font-semibold py-3 px-6 rounded-lg transition-colors"
		>
			Create New Draft
		</button>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{/if}

	<!-- Error State -->
	{#if error}
		<Card>
			<div class="text-red-600">
				<p class="font-semibold">Error loading drafts</p>
				<p class="text-sm">{error}</p>
			</div>
		</Card>
	{/if}

	<!-- Recent Drafts Section -->
	{#if !loading && !error}
		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<h2 class="text-2xl font-bold text-gray-800">Recent Drafts</h2>
				<a
					href="/drafts"
					data-sveltekit-reload
					class="text-blue-600 hover:text-blue-700 font-medium"
				>
					View All
				</a>
			</div>

			{#if drafts.length === 0}
				<Card>
					<div class="text-center py-8">
						<p class="text-gray-600 mb-4">No drafts yet. Create your first draft to get started!</p>
						<button
							type="button"
							onclick={async () => {
								await goto('/drafts/new');
							}}
							class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
						>
							Create Draft
						</button>
					</div>
				</Card>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each drafts.slice(0, 5) as draft (draft.id)}
						<Card
							clickable
							onclick={async () => {
								await goto(`/drafts/${draft.id}`);
							}}
						>
							<div class="space-y-3">
								<div class="flex items-start justify-between">
									<h3 class="text-lg font-semibold text-gray-800">
										{draft.year} Draft
									</h3>
									<Badge variant={getStatusVariant(draft.status)}>
										{draft.status}
									</Badge>
								</div>

								<div class="space-y-1 text-sm text-gray-600">
									<div class="flex items-center justify-between">
										<span>Rounds:</span>
										<span class="font-medium">{draft.rounds}</span>
									</div>
									<div class="flex items-center justify-between">
										<span>Picks per round:</span>
										<span class="font-medium">{draft.picks_per_round}</span>
									</div>
									{#if draft.created_at}
									<div class="flex items-center justify-between">
										<span>Created:</span>
										<span class="font-medium">
											{new Date(draft.created_at).toLocaleDateString()}
										</span>
									</div>
									{/if}
								</div>

								{#if draft.status === 'InProgress'}
									<div class="pt-2 border-t border-gray-200">
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
									</div>
								{/if}
							</div>
						</Card>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Quick Stats -->
		{#if drafts.length > 0}
			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-blue-600">
							{drafts.length}
						</div>
						<div class="text-sm text-gray-600 mt-1">Total Drafts</div>
					</div>
				</Card>
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-green-600">
							{drafts.filter((d) => d.status === 'InProgress').length}
						</div>
						<div class="text-sm text-gray-600 mt-1">Active Drafts</div>
					</div>
				</Card>
				<Card>
					<div class="text-center">
						<div class="text-3xl font-bold text-gray-600">
							{drafts.filter((d) => d.status === 'Completed').length}
						</div>
						<div class="text-sm text-gray-600 mt-1">Completed Drafts</div>
					</div>
				</Card>
			</div>
		{/if}
	{/if}

	<!-- Features Overview -->
	<div class="space-y-4">
		<h2 class="text-2xl font-bold text-gray-800">Features</h2>
		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<Card>
				<h3 class="text-lg font-semibold text-gray-800 mb-2">Real-time Updates</h3>
				<p class="text-gray-600">
					Watch drafts unfold in real-time with WebSocket-powered live updates. See picks as they
					happen.
				</p>
			</Card>
			<Card>
				<h3 class="text-lg font-semibold text-gray-800 mb-2">AI Decision Making</h3>
				<p class="text-gray-600">
					Experience intelligent team drafting with AI-driven decision-making based on team needs
					and player value.
				</p>
			</Card>
			<Card>
				<h3 class="text-lg font-semibold text-gray-800 mb-2">Comprehensive Scouting</h3>
				<p class="text-gray-600">
					Access detailed player profiles, combine results, and scouting reports to make informed
					decisions.
				</p>
			</Card>
			<Card>
				<h3 class="text-lg font-semibold text-gray-800 mb-2">Trade System</h3>
				<p class="text-gray-600">
					Execute pick trades using configurable trade value charts for realistic draft dynamics.
				</p>
			</Card>
		</div>
	</div>
</div>
