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
	let collapsedRounds = $state<Set<number>>(new Set());
	let initializedCollapse = $state(false);

	function toggleRound(round: number) {
		const next = new Set(collapsedRounds);
		if (next.has(round)) {
			next.delete(round);
		} else {
			next.add(round);
		}
		collapsedRounds = next;
	}

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

	// Collapse all rounds except round 1 on initial load
	$effect(() => {
		if (rounds.length > 0 && !initializedCollapse) {
			collapsedRounds = new Set(rounds.filter((r) => r !== 1));
			initializedCollapse = true;
		}
	});

	// Load teams and players when picks change
	$effect(() => {
		if (picks.length === 0) return;

		isLoading = true;

		const teamIds = new Set<string>();
		const playerIds = new Set<string>();

		picks.forEach((pick) => {
			teamIds.add(pick.team_id);
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

{#if isLoading}
	<div class="flex justify-center py-12">
		<LoadingSpinner size="lg" />
	</div>
{:else if picks.length === 0}
	<p class="text-center text-gray-500 py-12">No picks available</p>
{:else}
	<div class="space-y-8 max-h-[800px] overflow-y-auto p-1">
		{#each rounds as round (round)}
			<div>
				<button
					type="button"
					class="flex items-center space-x-3 mb-4 cursor-pointer group"
					aria-expanded={!collapsedRounds.has(round)}
					aria-controls="round-{round}-picks"
					onclick={() => toggleRound(round)}
				>
					<svg
						class="w-5 h-5 text-gray-500 transition-transform {collapsedRounds.has(round) ? '-rotate-90' : ''}"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
					</svg>
					<Badge variant="info" size="lg">
						Round {round}
					</Badge>
					<span class="text-sm text-gray-600">
						{picksByRound[round].length} picks
					</span>
				</button>
				{#if !collapsedRounds.has(round)}
					<div id="round-{round}-picks" class="flex flex-col gap-1">
						{#each picksByRound[round] as pick (pick.id)}
							{@const team = teams.get(pick.team_id)}
							{@const player = pick.player_id ? (players.get(pick.player_id) ?? null) : null}
							{#if team}
								<PickCard
									{pick}
									{player}
									{team}
									highlight={pick.overall_pick === draftState.currentPickNumber}
								/>
							{/if}
						{/each}
					</div>
				{/if}
			</div>
		{/each}
	</div>
{/if}
