<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { playersState } from '$stores/players.svelte';
	import PlayerList from '$components/player/PlayerList.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import type { Player } from '$lib/types';

	let loading = $state(true);
	let searchQuery = $state('');
	let selectedPosition = $state<string>('all');
	let selectedGroup = $state<string>('all');

	onMount(async () => {
		try {
			await playersState.loadAll();
		} catch (error) {
			logger.error('Failed to load players:', error);
		} finally {
			loading = false;
		}
	});

	// Position groups
	const positionGroups = {
		offense: ['QB', 'RB', 'WR', 'TE', 'OL', 'OT', 'OG', 'C'],
		defense: ['DL', 'DE', 'DT', 'LB', 'ILB', 'OLB', 'CB', 'S', 'FS', 'SS'],
		special_teams: ['K', 'P', 'LS'],
	};

	// All positions
	const allPositions = [
		...positionGroups.offense,
		...positionGroups.defense,
		...positionGroups.special_teams,
	];

	// Filter players
	let filteredPlayers = $derived(() => {
		let players = playersState.allPlayers;

		// Filter by search query
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			players = players.filter(
				(p) =>
					p.first_name.toLowerCase().includes(query) ||
					p.last_name.toLowerCase().includes(query) ||
					p.college?.toLowerCase().includes(query)
			);
		}

		// Filter by position group
		if (selectedGroup !== 'all') {
			const positions = positionGroups[selectedGroup as keyof typeof positionGroups] || [];
			players = players.filter((p) => positions.includes(p.position));
		}

		// Filter by specific position
		if (selectedPosition !== 'all') {
			players = players.filter((p) => p.position === selectedPosition);
		}

		return players;
	});

	async function handleSelectPlayer(player: Player) {
		await goto(`/players/${player.id}`);
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold text-gray-800">Players</h1>
		<div class="text-sm text-gray-600">
			{filteredPlayers().length} of {playersState.allPlayers.length} players
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
					<label for="search" class="block text-sm font-medium text-gray-700 mb-1"> Search </label>
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

				<!-- Specific Position -->
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
			{#if searchQuery || selectedGroup !== 'all' || selectedPosition !== 'all'}
				<div class="mt-4 flex items-center gap-2">
					<span class="text-sm text-gray-600">Active filters:</span>
					{#if searchQuery}
						<span
							class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm"
						>
							Search: "{searchQuery}"
							<button type="button" onclick={() => (searchQuery = '')} class="hover:text-blue-900">
								×
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
								×
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
								×
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

		<!-- Player List -->
		<div class="bg-white rounded-lg shadow p-4">
			{#if filteredPlayers().length === 0}
				<div class="text-center py-8 text-gray-600">No players found matching your filters.</div>
			{:else}
				<PlayerList
					players={filteredPlayers()}
					title="Available Players"
					onSelectPlayer={handleSelectPlayer}
				/>
			{/if}
		</div>
	{/if}
</div>
