<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { playersState } from '$stores/players.svelte';
	import { rankingsApi } from '$lib/api';
	import { computeConsensusRankings, sortByConsensusRank } from '$lib/utils/prospect-ranking';
	import type { ProspectRanking } from '$lib/utils/prospect-ranking';
	import ProspectRankingsTable from '$components/player/ProspectRankingsTable.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import { OFFENSE_POSITIONS, DEFENSE_POSITIONS, SPECIAL_POSITIONS } from '$lib/types';
	import type { Player, RankingBadge, RankingSource } from '$lib/types';

	let loading = $state(true);
	let rankingsLoading = $state(true);
	let searchQuery = $state('');
	let selectedPosition = $state<string>('all');
	let playerRankings = $state<Map<string, RankingBadge[]>>(new Map());
	let sources = $state<RankingSource[]>([]);
	let consensusRankings = $state<Map<string, ProspectRanking>>(new Map());
	let sortedPlayerIds = $state<string[]>([]);

	const allPositions = [...OFFENSE_POSITIONS, ...DEFENSE_POSITIONS, ...SPECIAL_POSITIONS];

	onMount(async () => {
		try {
			await playersState.loadAll();
		} catch (error) {
			logger.error('Failed to load players:', error);
		} finally {
			loading = false;
		}

		// Load rankings and sources in parallel (non-blocking)
		Promise.all([rankingsApi.loadAllPlayerRankings(), rankingsApi.listSources()])
			.then(([rankings, rankingSources]) => {
				playerRankings = rankings;
				sources = rankingSources;
				consensusRankings = computeConsensusRankings(rankings);
				sortedPlayerIds = sortByConsensusRank(consensusRankings);
			})
			.catch((error) => {
				logger.error('Failed to load rankings:', error);
			})
			.finally(() => {
				rankingsLoading = false;
			});
	});

	// Build player lookup map once (not inside filter derivation)
	let playerMap = $derived(
		(() => {
			const map = new Map<string, Player>();
			for (const p of playersState.allPlayers) {
				map.set(p.id, p);
			}
			return map;
		})(),
	);

	// Filter to only ranked players, then apply search/position filters
	let filteredSortedIds = $derived(
		(() => {
			let ids = sortedPlayerIds;

			if (searchQuery || selectedPosition !== 'all') {
				ids = ids.filter((id) => {
					const player = playerMap.get(id);
					if (!player) return false;

					if (searchQuery) {
						const query = searchQuery.toLowerCase();
						const matchesName =
							player.first_name.toLowerCase().includes(query) ||
							player.last_name.toLowerCase().includes(query);
						const matchesCollege = player.college?.toLowerCase().includes(query);
						if (!matchesName && !matchesCollege) return false;
					}

					if (selectedPosition !== 'all' && player.position !== selectedPosition) {
						return false;
					}

					return true;
				});
			}

			return ids;
		})(),
	);

	let unrankedCount = $derived(playersState.allPlayers.length - sortedPlayerIds.length);

	async function handleSelectPlayer(player: Player) {
		await goto(`/players/${player.id}`);
	}
</script>

<svelte:head>
	<title>Prospect Rankings - NFL Draft 2026</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold text-gray-800">Prospect Rankings</h1>
		<div class="text-sm text-gray-600">
			{sortedPlayerIds.length} ranked prospects | {sources.length} sources
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else}
		<!-- Filters -->
		<div class="bg-white rounded-lg shadow p-4">
			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<!-- Search -->
				<div>
					<label for="search" class="block text-sm font-medium text-gray-700 mb-1">Search</label>
					<input
						id="search"
						type="text"
						bind:value={searchQuery}
						placeholder="Search by name or college..."
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					/>
				</div>

				<!-- Position -->
				<div>
					<label for="position" class="block text-sm font-medium text-gray-700 mb-1">
						Position
					</label>
					<select
						id="position"
						bind:value={selectedPosition}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					>
						<option value="all">All Positions</option>
						{#each allPositions as position (position)}
							<option value={position}>{position}</option>
						{/each}
					</select>
				</div>
			</div>

			<!-- Active Filters -->
			{#if searchQuery || selectedPosition !== 'all'}
				<div class="mt-4 flex items-center gap-2">
					<span class="text-sm text-gray-600">Active filters:</span>
					{#if searchQuery}
						<span
							class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm"
						>
							Search: "{searchQuery}"
							<button type="button" onclick={() => (searchQuery = '')} class="hover:text-blue-900">
								&times;
							</button>
						</span>
					{/if}
					{#if selectedPosition !== 'all'}
						<span
							class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm"
						>
							Position: {selectedPosition}
							<button
								type="button"
								onclick={() => (selectedPosition = 'all')}
								class="hover:text-blue-900"
							>
								&times;
							</button>
						</span>
					{/if}
					<button
						type="button"
						onclick={() => {
							searchQuery = '';
							selectedPosition = 'all';
						}}
						class="text-sm text-blue-600 hover:text-blue-700"
					>
						Clear all
					</button>
				</div>
			{/if}
		</div>

		<!-- Rankings Table -->
		<div class="bg-white rounded-lg shadow">
			{#if rankingsLoading}
				<div class="flex justify-center py-12">
					<LoadingSpinner size="md" />
					<span class="ml-3 text-sm text-gray-500">Loading rankings...</span>
				</div>
			{:else if filteredSortedIds.length === 0}
				<div class="text-center py-8 text-gray-600">No ranked prospects found matching your filters.</div>
			{:else}
				<ProspectRankingsTable
					players={playersState.allPlayers}
					sortedPlayerIds={filteredSortedIds}
					{playerRankings}
					{consensusRankings}
					onSelectPlayer={handleSelectPlayer}
				/>
			{/if}
		</div>

		<!-- Footer: unranked players -->
		{#if !rankingsLoading && unrankedCount > 0}
			<div class="text-center text-sm text-gray-500">
				{unrankedCount} additional players not yet ranked.
				<a href="/players" class="text-blue-600 hover:text-blue-700 underline">View all players</a>
			</div>
		{/if}
	{/if}
</div>
