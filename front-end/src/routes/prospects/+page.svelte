<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { playersState } from '$stores/players.svelte';
	import { rankingsApi, freaksApi } from '$lib/api';
	import { computeConsensusRankings, sortByConsensusRank } from '$lib/utils/prospect-ranking';
	import { filterProspects, getPositionsForGroup } from '$lib/utils/prospect-filter';
	import type { ProspectRanking } from '$lib/utils/prospect-ranking';
	import ProspectRankingsTable from '$components/player/ProspectRankingsTable.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import type { Player, FeldmanFreak, RankingBadge, RankingSource } from '$lib/types';

	let loading = $state(true);
	let rankingsLoading = $state(true);
	let searchQuery = $state('');
	let selectedGroup = $state<string>('all');
	let selectedPosition = $state<string>('all');
	let playerRankings = $state<Map<string, RankingBadge[]>>(new Map());
	let sources = $state<RankingSource[]>([]);
	let consensusRankings = $state<Map<string, ProspectRanking>>(new Map());
	let sortedPlayerIds = $state<string[]>([]);
	let playerFreaks = $state<Map<string, FeldmanFreak>>(new Map());

	let availablePositions = $derived(getPositionsForGroup(selectedGroup));

	onMount(async () => {
		try {
			await playersState.loadAll();
		} catch (error) {
			logger.error('Failed to load players:', error);
		} finally {
			loading = false;
		}

		// Load rankings, sources, and freaks in parallel (non-blocking)
		Promise.all([
			rankingsApi.loadAllPlayerRankings(),
			rankingsApi.listSources(),
			freaksApi.loadByYear(2026),
		])
			.then(([rankings, rankingSources, freaks]) => {
				playerRankings = rankings;
				sources = rankingSources;
				consensusRankings = computeConsensusRankings(rankings);
				sortedPlayerIds = sortByConsensusRank(consensusRankings);
				playerFreaks = freaks;
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

	// Filter to only ranked players, then apply search/group/position filters
	let filteredSortedIds = $derived(
		filterProspects(sortedPlayerIds, playerMap, searchQuery, selectedGroup, selectedPosition),
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
			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
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

				<!-- Position Group -->
				<div>
					<label for="group" class="block text-sm font-medium text-gray-700 mb-1">
						Position Group
					</label>
					<select
						id="group"
						bind:value={selectedGroup}
						onchange={() => (selectedPosition = 'all')}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					>
						<option value="all">All Groups</option>
						<option value="offense">Offense</option>
						<option value="defense">Defense</option>
						<option value="special_teams">Special Teams</option>
					</select>
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
						{#each availablePositions as position (position)}
							<option value={position}>{position}</option>
						{/each}
					</select>
				</div>
			</div>

			<!-- Active Filters -->
			{#if searchQuery || selectedGroup !== 'all' || selectedPosition !== 'all'}
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
					{#if selectedGroup !== 'all'}
						<span
							class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm"
						>
							Group: {selectedGroup}
							<button
								type="button"
								onclick={() => (selectedGroup = 'all')}
								class="hover:text-blue-900"
							>
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
							selectedGroup = 'all';
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
					{playerFreaks}
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
