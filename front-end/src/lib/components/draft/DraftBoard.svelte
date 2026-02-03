<script lang="ts">
	import { PickCard } from '$components/draft';
	import { LoadingSpinner, Badge } from '$components/ui';
	import type { DraftPick, Player, Team } from '$types';
	import { teamsApi, playersApi } from '$api';
	import { draftState } from '$stores';
	import { logger } from '$lib/utils/logger';

	interface Props {
		picks: DraftPick[];
	}

	let { picks }: Props = $props();

	let teams = $state<Map<string, Team>>(new Map());
	let players = $state<Map<string, Player>>(new Map());
	let isLoading = $state(false);

	// Group picks by round
	const picksByRound = $derived(
		picks.reduce(
			(acc, pick) => {
				if (!acc[pick.round]) {
					acc[pick.round] = [];
				}
				acc[pick.round].push(pick);
				return acc;
			},
			{} as Record<number, DraftPick[]>
		)
	);

	const rounds = $derived(
		Object.keys(picksByRound)
			.map(Number)
			.sort((a, b) => a - b)
	);

	// Load teams and players when picks change
	$effect(() => {
		if (picks.length === 0) return;

		isLoading = true;

		const teamIds = new Set<string>();
		const playerIds = new Set<string>();

		picks.forEach((pick) => {
			teamIds.add(pick.current_team_id);
			if (pick.player_id) {
				playerIds.add(pick.player_id);
			}
		});

		Promise.all([
			...Array.from(teamIds).map((id) =>
				teamsApi.get(id).then((team) => {
					teams.set(id, team);
				})
			),
			...Array.from(playerIds).map((id) =>
				playersApi.get(id).then((player) => {
					players.set(id, player);
				})
			),
		])
			.catch((err) => {
				logger.error('Failed to load data:', err);
			})
			.finally(() => {
				isLoading = false;
			});
	});
</script>

<div class="bg-gray-50 rounded-lg p-6">
	<h2 class="text-2xl font-bold text-gray-900 mb-6">Draft Board</h2>

	{#if isLoading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if picks.length === 0}
		<p class="text-center text-gray-500 py-12">No picks available</p>
	{:else}
		<div class="space-y-8 max-h-[800px] overflow-y-auto">
			{#each rounds as round (round)}
				<div>
					<div class="flex items-center space-x-3 mb-4">
						<Badge variant="info" size="lg">
							Round {round}
						</Badge>
						<span class="text-sm text-gray-600">
							{picksByRound[round].length} picks
						</span>
					</div>
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
						{#each picksByRound[round] as pick (pick.id)}
							{@const team = teams.get(pick.current_team_id)}
							{@const player = pick.player_id ? players.get(pick.player_id) ?? null : null}
							{#if team}
								<PickCard
									{pick}
									player={player}
									{team}
									highlight={pick.overall_pick === draftState.currentPickNumber}
								/>
							{/if}
						{/each}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
