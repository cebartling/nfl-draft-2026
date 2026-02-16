<script lang="ts">
	import { Badge } from '$components/ui';
	import { getPositionColor, formatHeight } from '$lib/utils/formatters';
	import type { Player, RankingBadge } from '$types';
	import type { ProspectRanking } from '$lib/utils/prospect-ranking';

	interface Props {
		players: Player[];
		sortedPlayerIds: string[];
		playerRankings: Map<string, RankingBadge[]>;
		consensusRankings: Map<string, ProspectRanking>;
		onSelectPlayer?: (player: Player) => void;
	}

	let { players, sortedPlayerIds, playerRankings, consensusRankings, onSelectPlayer }: Props =
		$props();

	const playerMap = $derived(
		(() => {
			const map = new Map<string, Player>();
			for (const p of players) {
				map.set(p.id, p);
			}
			return map;
		})(),
	);

	const rankedPlayers = $derived(
		sortedPlayerIds
			.map((id) => playerMap.get(id))
			.filter((p): p is Player => p !== undefined),
	);
</script>

<div class="overflow-x-auto">
	<table class="min-w-full divide-y divide-gray-200">
		<thead class="bg-gray-50">
			<tr>
				<th
					scope="col"
					class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-16"
				>
					#
				</th>
				<th
					scope="col"
					class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
				>
					Player
				</th>
				<th
					scope="col"
					class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-20"
				>
					Pos
				</th>
				<th
					scope="col"
					class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider hidden md:table-cell"
				>
					College
				</th>
				<th
					scope="col"
					class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider hidden lg:table-cell w-28"
				>
					Size
				</th>
				<th
					scope="col"
					class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
				>
					Big Board Rankings
				</th>
			</tr>
		</thead>
		<tbody class="bg-white divide-y divide-gray-200">
			{#each rankedPlayers as player, index (player.id)}
				{@const ranking = consensusRankings.get(player.id)}
				{@const badges = playerRankings.get(player.id) ?? []}
				<tr
					class="hover:bg-gray-50 {onSelectPlayer ? 'cursor-pointer' : ''}"
					onclick={() => onSelectPlayer?.(player)}
					role={onSelectPlayer ? 'button' : undefined}
					tabindex={onSelectPlayer ? 0 : undefined}
					onkeydown={(e) => {
						if (onSelectPlayer && (e.key === 'Enter' || e.key === ' ')) {
							e.preventDefault();
							onSelectPlayer(player);
						}
					}}
				>
					<td class="px-4 py-3 whitespace-nowrap">
						<div class="text-sm font-bold text-gray-900">{index + 1}</div>
					</td>
					<td class="px-4 py-3 whitespace-nowrap">
						<div class="text-sm font-semibold text-gray-900">
							{player.first_name}
							{player.last_name}
						</div>
					</td>
					<td class="px-4 py-3 whitespace-nowrap">
						<Badge variant={getPositionColor(player.position)} size="sm">
							{player.position}
						</Badge>
					</td>
					<td class="px-4 py-3 whitespace-nowrap text-sm text-gray-600 hidden md:table-cell">
						{player.college || 'N/A'}
					</td>
					<td
						class="px-4 py-3 whitespace-nowrap text-sm text-gray-500 hidden lg:table-cell"
					>
						{formatHeight(player.height_inches)}{#if player.weight_pounds}, {player.weight_pounds} lbs{/if}
					</td>
					<td class="px-4 py-3">
						<div class="flex flex-wrap items-center gap-1.5">
							{#each badges as badge (badge.source_name)}
								<span
									class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-700"
									title="{badge.source_name}: #{badge.rank}"
								>
									{badge.abbreviation}:&nbsp;#{badge.rank}
								</span>
							{/each}
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
