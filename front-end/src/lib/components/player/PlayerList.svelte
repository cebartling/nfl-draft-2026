<script lang="ts">
	import { PlayerCard } from '$components/player';
	import { Button, Badge } from '$components/ui';
	import type { Player, Position } from '$types';
	import { OFFENSE_POSITIONS, DEFENSE_POSITIONS, SPECIAL_POSITIONS } from '$types';

	interface Props {
		players: Player[];
		title: string;
		onSelectPlayer?: (player: Player) => void;
	}

	let { players, title, onSelectPlayer }: Props = $props();

	let searchQuery = $state('');
	let selectedPosition = $state<Position | 'all'>('all');

	const filteredPlayers = $derived(
		players.filter((player) => {
			const matchesSearch =
				searchQuery === '' ||
				player.first_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				player.last_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				player.college?.toLowerCase().includes(searchQuery.toLowerCase());

			const matchesPosition = selectedPosition === 'all' || player.position === selectedPosition;

			return matchesSearch && matchesPosition;
		})
	);

	function getPositionGroups() {
		return [
			{ label: 'All', positions: ['all'] as const },
			{ label: 'Offense', positions: OFFENSE_POSITIONS },
			{ label: 'Defense', positions: DEFENSE_POSITIONS },
			{ label: 'Special Teams', positions: SPECIAL_POSITIONS },
		];
	}
</script>

<div class="bg-gray-50 rounded-lg p-6">
	<h2 class="text-2xl font-bold text-gray-900 mb-6">{title}</h2>

	<div class="space-y-4 mb-6">
		<!-- Search Input -->
		<div>
			<label for="player-search" class="sr-only">Search players</label>
			<input
				id="player-search"
				type="text"
				placeholder="Search by name or college..."
				bind:value={searchQuery}
				class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
			/>
		</div>

		<!-- Position Filters -->
		<div class="space-y-3">
			{#each getPositionGroups() as group (group.label)}
				<div>
					<p class="text-sm font-medium text-gray-700 mb-2">{group.label}</p>
					<div class="flex flex-wrap gap-2">
						{#each group.positions as position (position)}
							<button
								type="button"
								class="px-3 py-1.5 rounded-lg text-sm font-medium transition-colors {selectedPosition ===
								position
									? 'bg-blue-600 text-white'
									: 'bg-white text-gray-700 hover:bg-gray-100'}"
								onclick={() => (selectedPosition = position)}
							>
								{position === 'all' ? 'All' : position}
							</button>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</div>

	<!-- Results Count -->
	<div class="flex items-center justify-between mb-4">
		<p class="text-sm text-gray-600">
			{filteredPlayers.length}
			{filteredPlayers.length === 1 ? 'player' : 'players'}
		</p>
		{#if searchQuery || selectedPosition !== 'all'}
			<Button
				variant="secondary"
				size="sm"
				onclick={() => {
					searchQuery = '';
					selectedPosition = 'all';
				}}
			>
				Clear Filters
			</Button>
		{/if}
	</div>

	<!-- Player Cards -->
	<div class="max-h-[600px] overflow-y-auto">
		{#if filteredPlayers.length === 0}
			<p class="text-center text-gray-500 py-12">No players found</p>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each filteredPlayers as player (player.id)}
					<PlayerCard {player} onSelect={onSelectPlayer} />
				{/each}
			</div>
		{/if}
	</div>
</div>
