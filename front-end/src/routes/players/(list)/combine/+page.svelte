<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { combineApi } from '$lib/api';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import CombineResultsTable from '$components/player/CombineResultsTable.svelte';
	import CombineComparisonPanel from '$components/player/CombineComparisonPanel.svelte';
	import { buildPercentilesMap, type PercentilesMap } from '$lib/utils/combine-percentile';
	import type { CombineResultsWithPlayer } from '$lib/types';
	import type { Position } from '$lib/types';

	let loading = $state(true);
	let allResults = $state<CombineResultsWithPlayer[]>([]);
	let percentilesMap = $state<PercentilesMap>(new Map());
	let searchQuery = $state('');
	let selectedGroup = $state<string>('all');
	let selectedPosition = $state<string>('all');
	let selectedSource = $state<string>('all');
	let sortColumn = $state('player_last_name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let selectedPlayerIds = $state<Set<string>>(new Set());
	let showComparison = $state(false);

	onMount(async () => {
		try {
			const [results, percentiles] = await Promise.all([
				combineApi.listAll(),
				combineApi.getPercentiles(),
			]);
			allResults = results;
			percentilesMap = buildPercentilesMap(percentiles);
		} catch (error) {
			logger.error('Failed to load combine results:', error);
		} finally {
			loading = false;
		}
	});

	const positionGroups = {
		offense: ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'],
		defense: ['DE', 'DT', 'LB', 'CB', 'S'],
		special_teams: ['K', 'P'],
	};

	let availablePositions = $derived(() => {
		if (selectedGroup === 'all') {
			return [...new Set(allResults.map((r) => r.position))].sort();
		}
		return positionGroups[selectedGroup as keyof typeof positionGroups] || [];
	});

	let availableSources = $derived(() => {
		return [...new Set(allResults.map((r) => r.source))].sort();
	});

	let filteredResults = $derived(() => {
		let results = allResults;

		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			results = results.filter(
				(r) =>
					r.player_first_name.toLowerCase().includes(query) ||
					r.player_last_name.toLowerCase().includes(query) ||
					r.college?.toLowerCase().includes(query)
			);
		}

		if (selectedGroup !== 'all') {
			const positions = positionGroups[selectedGroup as keyof typeof positionGroups] || [];
			results = results.filter((r) => positions.includes(r.position));
		}

		if (selectedPosition !== 'all') {
			results = results.filter((r) => r.position === selectedPosition);
		}

		if (selectedSource !== 'all') {
			results = results.filter((r) => r.source === selectedSource);
		}

		// Sort
		results = [...results].sort((a, b) => {
			const aVal = (a as Record<string, unknown>)[sortColumn];
			const bVal = (b as Record<string, unknown>)[sortColumn];

			if (aVal == null && bVal == null) return 0;
			if (aVal == null) return 1;
			if (bVal == null) return -1;

			let cmp: number;
			if (typeof aVal === 'number' && typeof bVal === 'number') {
				cmp = aVal - bVal;
			} else {
				cmp = String(aVal).localeCompare(String(bVal));
			}

			return sortDirection === 'asc' ? cmp : -cmp;
		});

		return results;
	});

	let selectedPlayers = $derived(() => {
		return allResults.filter((r) => selectedPlayerIds.has(r.id));
	});

	function handleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
	}

	function handleToggleSelect(id: string) {
		const next = new Set(selectedPlayerIds);
		if (next.has(id)) {
			next.delete(id);
		} else if (next.size < 3) {
			next.add(id);
		}
		selectedPlayerIds = next;
		if (next.size === 0) {
			showComparison = false;
		}
	}

	async function handleSelectPlayer(result: CombineResultsWithPlayer) {
		await goto(`/players/${result.player_id}`);
	}
</script>

<div class="space-y-4">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold text-gray-800">Combine Results</h1>
		<div class="flex items-center gap-4">
			{#if selectedPlayerIds.size >= 2}
				<button
					type="button"
					onclick={() => (showComparison = !showComparison)}
					class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
				>
					{showComparison ? 'Hide' : 'Compare'} ({selectedPlayerIds.size})
				</button>
			{/if}
			<div class="text-sm text-gray-600">
				{filteredResults().length} of {allResults.length} results
			</div>
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else}
		<!-- Filters -->
		<div class="bg-white rounded-lg shadow p-4">
			<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
				<!-- Search -->
				<div>
					<label for="combine-search" class="block text-sm font-medium text-gray-700 mb-1">
						Search
					</label>
					<input
						id="combine-search"
						type="text"
						bind:value={searchQuery}
						placeholder="Search by name or college..."
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					/>
				</div>

				<!-- Position Group -->
				<div>
					<label for="combine-group" class="block text-sm font-medium text-gray-700 mb-1">
						Position Group
					</label>
					<select
						id="combine-group"
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
					<label for="combine-position" class="block text-sm font-medium text-gray-700 mb-1">
						Position
					</label>
					<select
						id="combine-position"
						bind:value={selectedPosition}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					>
						<option value="all">All Positions</option>
						{#each availablePositions() as pos (pos)}
							<option value={pos}>{pos}</option>
						{/each}
					</select>
				</div>

				<!-- Source -->
				<div>
					<label for="combine-source" class="block text-sm font-medium text-gray-700 mb-1">
						Source
					</label>
					<select
						id="combine-source"
						bind:value={selectedSource}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					>
						<option value="all">All Sources</option>
						{#each availableSources() as src (src)}
							<option value={src}>{src === 'combine' ? 'Combine' : 'Pro Day'}</option>
						{/each}
					</select>
				</div>
			</div>

			<!-- Active Filters -->
			{#if searchQuery || selectedGroup !== 'all' || selectedPosition !== 'all' || selectedSource !== 'all'}
				<div class="mt-4 flex items-center gap-2 flex-wrap">
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
					{#if selectedSource !== 'all'}
						<span
							class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm"
						>
							Source: {selectedSource}
							<button
								type="button"
								onclick={() => (selectedSource = 'all')}
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
							selectedSource = 'all';
						}}
						class="text-sm text-blue-600 hover:text-blue-700"
					>
						Clear all
					</button>
				</div>
			{/if}
		</div>

		<!-- Comparison Panel -->
		{#if showComparison && selectedPlayers().length >= 2}
			<CombineComparisonPanel
				players={selectedPlayers()}
				{percentilesMap}
				onClose={() => {
					showComparison = false;
				}}
			/>
		{/if}

		<!-- Results Table -->
		<div class="bg-white rounded-lg shadow">
			{#if filteredResults().length === 0}
				<div class="text-center py-8 text-gray-600">
					No combine results found matching your filters.
				</div>
			{:else}
				<CombineResultsTable
					results={filteredResults()}
					{percentilesMap}
					{sortColumn}
					{sortDirection}
					selectedIds={selectedPlayerIds}
					onSort={handleSort}
					onToggleSelect={handleToggleSelect}
					onSelectPlayer={handleSelectPlayer}
				/>
			{/if}
		</div>
	{/if}
</div>
