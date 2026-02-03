<script lang="ts">
	import { Badge } from '$components/ui';
	import type { DraftPick, Player, Team } from '$types';
	import dayjs from 'dayjs';

	interface Props {
		pick: DraftPick;
		player: Player | null;
		team: Team;
		highlight?: boolean;
	}

	let { pick, player, team, highlight = false }: Props = $props();
</script>

<div
	class="bg-white rounded-lg shadow-md p-4 transition-all {highlight
		? 'ring-2 ring-blue-500 shadow-lg'
		: 'hover:shadow-lg'}"
>
	<div class="flex items-start justify-between mb-3">
		<div class="flex items-center space-x-2">
			<Badge variant="primary" size="sm">
				Round {pick.round}
			</Badge>
			<Badge variant="default" size="sm">
				Pick {pick.pick_number}
			</Badge>
		</div>
		<span class="text-sm font-medium text-gray-600">
			#{pick.overall_pick}
		</span>
	</div>

	<div class="space-y-2">
		<div>
			<p class="text-sm font-medium text-gray-600">Team</p>
			<p class="text-base font-semibold text-gray-900">
				{team.abbreviation}
			</p>
		</div>

		<div>
			<p class="text-sm font-medium text-gray-600">Player</p>
			{#if player}
				<p class="text-base font-semibold text-gray-900">
					{player.first_name}
					{player.last_name}
				</p>
				<p class="text-sm text-gray-600">
					{player.position} - {player.college || 'N/A'}
				</p>
			{:else}
				<p class="text-base text-gray-400 italic">Available</p>
			{/if}
		</div>

		{#if pick.picked_at}
			<div>
				<p class="text-xs text-gray-500">
					Picked at {dayjs(pick.picked_at).format('h:mm A')}
				</p>
			</div>
		{/if}
	</div>
</div>
