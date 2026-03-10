<script lang="ts">
	import { Badge } from '$components/ui';
	import type { DraftPick, Player, Team } from '$types';
	import { getTeamLogoPath } from '$lib/utils/logo';

	interface Props {
		pick: DraftPick;
		player: Player | null;
		team: Team;
		highlight?: boolean;
	}

	let { pick, player, team, highlight = false }: Props = $props();
	let logoError = $state(false);
</script>

<div
	data-pick={pick.overall_pick}
	data-current={highlight ? "true" : undefined}
	class="flex items-center px-3 py-2 border-b border-gray-200 bg-white transition-all {highlight
		? 'ring-2 ring-blue-500 bg-blue-50'
		: 'hover:bg-gray-50'}"
>
	<!-- Pick number -->
	<span class="w-10 text-sm font-bold text-gray-500 shrink-0 text-right mr-3">
		#{pick.overall_pick}
	</span>

	<!-- Round/Pick info + badges -->
	<div class="flex items-center gap-1 w-40 shrink-0 mr-3">
		<span class="text-xs text-gray-500">R{pick.round} P{pick.pick_number}</span>
		{#if pick.is_compensatory}
			<Badge variant="warning" size="sm">COMP</Badge>
		{/if}
		{#if pick.is_traded}
			<Badge variant="info" size="sm">TRADED</Badge>
		{/if}
	</div>

	<!-- Team -->
	<div class="flex items-center gap-2 w-20 shrink-0 mr-3">
		{#if !logoError}
			<img
				src={getTeamLogoPath(team.abbreviation)}
				alt={`${team.abbreviation} logo`}
				class="w-5 h-5 object-contain"
				onerror={() => { logoError = true; }}
			/>
		{/if}
		<span class="text-sm font-semibold text-gray-900">{team.abbreviation}</span>
	</div>

	<!-- Player -->
	<div class="flex-1 min-w-0 mr-3">
		{#if player}
			<span data-testid="pick-player-name" class="text-sm font-semibold text-gray-900">
				{player.first_name} {player.last_name}
			</span>
			<span data-testid="pick-player-position" class="text-xs text-gray-500 ml-1">
				{player.position}
			</span>
			<span data-testid="pick-player-college" class="text-xs text-gray-500 ml-1">
				{player.college || 'N/A'}
			</span>
		{:else}
			<span class="text-sm text-gray-400 italic">&mdash;</span>
		{/if}
	</div>

	<!-- Notes -->
	{#if pick.notes}
		<span class="text-xs text-gray-400 italic truncate max-w-48 shrink-0">
			{pick.notes}
		</span>
	{/if}
</div>
