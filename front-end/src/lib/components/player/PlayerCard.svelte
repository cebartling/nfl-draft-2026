<script lang="ts">
	import { Badge } from '$components/ui';
	import type { Player } from '$types';

	interface Props {
		player: Player;
		onSelect?: (player: Player) => void;
	}

	let { player, onSelect }: Props = $props();

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
</script>

<div
	class="bg-white rounded-lg shadow-md p-4 transition-all {onSelect
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
	<div class="flex items-start justify-between mb-3">
		<div class="flex-1">
			<h3 class="text-lg font-semibold text-gray-900">
				{player.first_name} {player.last_name}
			</h3>
			<p class="text-sm text-gray-600">{player.college || 'N/A'}</p>
		</div>
		<Badge variant={getPositionColor(player.position)} size="lg">
			{player.position}
		</Badge>
	</div>

	<div class="grid grid-cols-2 gap-3 mb-3">
		{#if player.height_inches || player.weight_pounds}
			<div>
				<p class="text-xs font-medium text-gray-600">Measurables</p>
				<p class="text-sm text-gray-900">
					{formatHeight(player.height_inches)}
					{#if player.weight_pounds}
						/ {player.weight_pounds} lbs
					{/if}
				</p>
			</div>
		{/if}

		{#if player.projected_round}
			<div>
				<p class="text-xs font-medium text-gray-600">Projected Round</p>
				<Badge variant="default" size="sm">
					Round {player.projected_round}
				</Badge>
			</div>
		{/if}
	</div>

	<div class="pt-3 border-t border-gray-200">
		<p class="text-xs text-gray-500">Draft Year: {player.draft_year}</p>
	</div>
</div>
