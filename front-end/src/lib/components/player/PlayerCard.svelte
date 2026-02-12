<script lang="ts">
	import { Badge } from '$components/ui';
	import type { AvailablePlayer } from '$types';

	interface Props {
		player: AvailablePlayer;
		onSelect?: (player: AvailablePlayer) => void;
		onViewDetails?: (player: AvailablePlayer) => void;
	}

	let { player, onSelect, onViewDetails }: Props = $props();

	function getPositionColor(position: string): 'primary' | 'danger' | 'info' {
		const offensePositions = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
		const defensePositions = ['DE', 'DT', 'LB', 'CB', 'S'];

		if (offensePositions.includes(position)) return 'primary';
		if (defensePositions.includes(position)) return 'danger';
		return 'info';
	}

	function formatHeight(inches?: number | null): string {
		if (!inches) return 'N/A';
		const feet = Math.floor(inches / 12);
		const remainingInches = inches % 12;
		return `${feet}'${remainingInches}"`;
	}

	function getGradeColor(grade: number): string {
		if (grade >= 80) return 'text-green-700 bg-green-100';
		if (grade >= 60) return 'text-blue-700 bg-blue-100';
		if (grade >= 40) return 'text-yellow-700 bg-yellow-100';
		return 'text-gray-700 bg-gray-100';
	}

	function handleInfoClick(event: MouseEvent) {
		event.stopPropagation();
		onViewDetails?.(player);
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	class="bg-white rounded-lg shadow-md px-4 py-3 transition-all {onSelect
		? 'hover:shadow-lg cursor-pointer'
		: ''}"
	onclick={() => onSelect?.(player)}
	role={onSelect ? 'button' : 'article'}
	tabindex={onSelect ? 0 : undefined}
	onkeydown={(e) => {
		if (onSelect && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onSelect(player);
		}
	}}
>
	<div class="flex items-center gap-4">
		{#if player.scouting_grade != null}
			<span class="inline-flex items-center justify-center w-12 px-2 py-1 rounded text-sm font-bold {getGradeColor(player.scouting_grade)}">
				{player.scouting_grade.toFixed(1)}
			</span>
		{/if}
		<Badge variant={getPositionColor(player.position)} size="lg">
			{player.position}
		</Badge>
		<div class="flex-1 min-w-0">
			<h3 class="text-base font-semibold text-gray-900 truncate">
				{player.first_name}
				{player.last_name}
			</h3>
		</div>
		{#if player.rankings && player.rankings.length > 0}
			<div class="flex items-center gap-1.5">
				{#each player.rankings as badge (badge.source_name)}
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-700" title="{badge.source_name}: #{badge.rank}">
						{badge.abbreviation}:&nbsp;#{badge.rank}
					</span>
				{/each}
			</div>
		{/if}
		<p class="text-sm text-gray-600 hidden sm:block">{player.college || 'N/A'}</p>
		{#if player.height_inches || player.weight_pounds}
			<p class="text-sm text-gray-500 hidden md:block">
				{formatHeight(player.height_inches)}{#if player.weight_pounds}, {player.weight_pounds} lbs{/if}
			</p>
		{/if}
		{#if onViewDetails}
			<button
				type="button"
				class="flex-shrink-0 p-1.5 rounded-full text-gray-400 hover:text-blue-600 hover:bg-blue-50 transition-colors"
				onclick={handleInfoClick}
				aria-label="View details for {player.first_name} {player.last_name}"
				title="View player details"
			>
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
			</button>
		{/if}
	</div>
</div>
