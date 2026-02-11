<script lang="ts">
	import { Badge } from '$components/ui';
	import type { Player } from '$types';

	interface Props {
		player: Player;
		scoutingGrade?: number;
		onSelect?: (player: Player) => void;
	}

	let { player, scoutingGrade, onSelect }: Props = $props();

	function getPositionColor(position: string): 'primary' | 'danger' | 'info' {
		const offensePositions = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
		const defensePositions = ['DE', 'DT', 'LB', 'CB', 'S'];

		if (offensePositions.includes(position)) return 'primary';
		if (defensePositions.includes(position)) return 'danger';
		return 'info';
	}

	function formatHeight(inches?: number): string {
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
		{#if scoutingGrade !== undefined}
			<span class="inline-flex items-center justify-center w-12 px-2 py-1 rounded text-sm font-bold {getGradeColor(scoutingGrade)}">
				{scoutingGrade.toFixed(1)}
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
		<p class="text-sm text-gray-600 hidden sm:block">{player.college || 'N/A'}</p>
		{#if player.height_inches || player.weight_pounds}
			<p class="text-sm text-gray-500 hidden md:block">
				{formatHeight(player.height_inches)}{#if player.weight_pounds}, {player.weight_pounds} lbs{/if}
			</p>
		{/if}
	</div>
</div>
