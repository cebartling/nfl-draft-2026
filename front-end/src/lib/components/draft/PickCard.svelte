<script lang="ts">
	import { Badge, Tooltip } from '$components/ui';
	import type { DraftPick, Player, Team, RankingBadge, FeldmanFreak } from '$types';
	import { getTeamLogoPath } from '$lib/utils/logo';
	import { getPositionColor } from '$lib/utils/formatters';

	interface Props {
		pick: DraftPick;
		player: Player | null;
		team: Team;
		rankings?: RankingBadge[];
		freak?: FeldmanFreak | null;
		highlight?: boolean;
	}

	let { pick, player, team, rankings = [], freak = null, highlight = false }: Props = $props();
	let logoError = $state(false);

	// Best rank across all sources — show as the prominent "overall" number
	const bestRank = $derived(rankings.length > 0 ? Math.min(...rankings.map((r) => r.rank)) : null);
</script>

<div
	data-pick={pick.overall_pick}
	data-current={highlight ? 'true' : undefined}
	class="flex items-center px-3 py-2 border-b border-gray-200 bg-white transition-all {highlight
		? 'ring-2 ring-blue-500 bg-blue-50'
		: 'hover:bg-gray-50'}"
>
	<!-- Pick number -->
	<span class="w-10 text-sm font-bold text-gray-500 shrink-0 text-right mr-3">
		#{pick.overall_pick}
	</span>

	<!-- Round/Pick info + badges -->
	<div class="flex items-center gap-1 w-36 shrink-0 mr-3">
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
				onerror={() => {
					logoError = true;
				}}
			/>
		{/if}
		<span class="text-sm font-semibold text-gray-900">{team.abbreviation}</span>
	</div>

	<!-- Player -->
	<div class="flex-1 min-w-0 mr-3">
		{#if player}
			<div class="flex items-center gap-2 flex-wrap">
				<span data-testid="pick-player-position">
					<Badge variant={getPositionColor(player.position)} size="sm">
						{player.position}
					</Badge>
				</span>
				<span data-testid="pick-player-name" class="text-sm font-semibold text-gray-900">
					{player.first_name}
					{player.last_name}
				</span>
				<span data-testid="pick-player-college" class="text-xs text-gray-500">
					{player.college || 'N/A'}
				</span>
			</div>
		{:else}
			<span class="text-sm text-gray-400 italic">&mdash;</span>
		{/if}
	</div>

	<!-- Rankings & Freak badges (only for completed picks) -->
	{#if player}
		<div class="flex items-center gap-1.5 shrink-0">
			{#if bestRank !== null}
				<span
					class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-bold bg-indigo-100 text-indigo-800"
					title="Best consensus rank: #{bestRank}"
				>
					#{bestRank}
				</span>
			{/if}
			{#each rankings as badge (badge.source_name)}
				<span
					class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-700"
					title="{badge.source_name}: #{badge.rank}"
				>
					{badge.abbreviation}:#{badge.rank}
				</span>
			{/each}
			{#if freak}
				<Tooltip text="Feldman Freak #{freak.rank}: {freak.description}" width="w-80">
					<span
						class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-bold bg-amber-100 text-amber-800 border border-amber-300"
					>
						FREAK #{freak.rank}
					</span>
				</Tooltip>
			{/if}
		</div>
	{/if}

	<!-- Notes -->
	{#if pick.notes}
		<span class="text-xs text-gray-400 italic truncate max-w-32 shrink-0 ml-2">
			{pick.notes}
		</span>
	{/if}
</div>
