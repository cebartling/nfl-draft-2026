<script lang="ts">
	import { PlayerCard } from '$components/player';
	import { Badge } from '$components/ui';
	import type { Player, Position, RankingBadge } from '$types';
	import { OFFENSE_POSITIONS, DEFENSE_POSITIONS, SPECIAL_POSITIONS } from '$types';

	interface Props {
		players: Player[];
		title: string;
		scoutingGrades?: Map<string, number>;
		playerRankings?: Map<string, RankingBadge[]>;
		onSelectPlayer?: (player: Player) => void;
	}

	let { players, title, scoutingGrades, playerRankings, onSelectPlayer }: Props = $props();

	let searchQuery = $state('');
	let selectedPosition = $state<Position | 'all'>('all');

	const allPositions: (Position | 'all')[] = ['all', ...OFFENSE_POSITIONS, ...DEFENSE_POSITIONS, ...SPECIAL_POSITIONS];

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
</script>

<div class="bg-gray-50 rounded-lg p-4">
	<!-- Header + Search + Filters -->
	<div class="flex flex-wrap items-center gap-2 mb-3">
		<h2 class="text-lg font-bold text-gray-900 mr-2">{title}</h2>
		<label for="player-search" class="sr-only">Search players</label>
		<input
			id="player-search"
			type="text"
			placeholder="Search..."
			bind:value={searchQuery}
			class="w-48 rounded-md border border-gray-300 px-2 py-1 text-sm focus:border-blue-500 focus:ring-blue-500"
		/>
		<span class="text-sm text-gray-500">{filteredPlayers.length} players</span>
		{#if searchQuery || selectedPosition !== 'all'}
			<button
				type="button"
				class="text-xs text-blue-600 hover:text-blue-800 font-medium"
				onclick={() => { searchQuery = ''; selectedPosition = 'all'; }}
			>
				Clear
			</button>
		{/if}
	</div>

	<!-- Position Filters - single row -->
	<div class="flex flex-wrap gap-1 mb-3">
		{#each allPositions as position (position)}
			<button
				type="button"
				class="px-2 py-0.5 rounded text-xs font-medium transition-colors {selectedPosition === position
					? 'bg-blue-600 text-white'
					: 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}"
				onclick={() => (selectedPosition = position)}
			>
				{position === 'all' ? 'All' : position}
			</button>
		{/each}
	</div>

	<!-- Player Cards -->
	<div class="max-h-[600px] overflow-y-auto">
		{#if filteredPlayers.length === 0}
			<p class="text-center text-gray-500 py-8">No players found</p>
		{:else}
			<div class="flex flex-col gap-2">
				{#each filteredPlayers as player (player.id)}
					<PlayerCard {player} scoutingGrade={scoutingGrades?.get(player.id)} rankings={playerRankings?.get(player.id)} onSelect={onSelectPlayer} />
				{/each}
			</div>
		{/if}
	</div>
</div>
