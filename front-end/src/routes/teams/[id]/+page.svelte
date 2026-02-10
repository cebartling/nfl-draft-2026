<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { teamsApi, teamSeasonsApi } from '$lib/api';
	import { draftsApi } from '$lib/api';
	import TeamCard from '$components/team/TeamCard.svelte';
	import TeamNeeds from '$components/team/TeamNeeds.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import Card from '$components/ui/Card.svelte';
	import Badge from '$components/ui/Badge.svelte';
	import type { Team, DraftPick, TeamSeason } from '$lib/types';
	import { getTeamLogoPath } from '$lib/utils/logo';
	import { STANDINGS_SEASON_YEAR } from '$lib/config/draft';

	let teamId = $derived($page.params.id!);
	let team = $state<Team | null>(null);
	let teamSeason = $state<TeamSeason | null>(null);
	let teamPicks = $state<DraftPick[]>([]);
	let loading = $state(true);
	let picksLoading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		// Load team details
		try {
			team = await teamsApi.get(teamId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load team';
			logger.error('Failed to load team:', e);
		} finally {
			loading = false;
		}

		// Load team's season record
		try {
			teamSeason = await teamSeasonsApi.getByTeamAndYear(teamId, STANDINGS_SEASON_YEAR);
		} catch (e) {
			logger.error('Failed to load team season:', e);
		}

		// Load team's picks (if any active draft exists)
		try {
			// This is a simplified version - in a real app, you'd get the current draft ID
			// For now, we'll just show empty picks
			teamPicks = [];
		} catch (e) {
			logger.error('Failed to load team picks:', e);
		} finally {
			picksLoading = false;
		}
	});

	// Format the record as W-L-T
	function formatRecord(season: TeamSeason): string {
		if (season.ties > 0) {
			return `${season.wins}-${season.losses}-${season.ties}`;
		}
		return `${season.wins}-${season.losses}`;
	}

	// Get playoff result display text
	function getPlayoffDisplay(result: string | null | undefined): string {
		if (!result) return 'N/A';
		switch (result) {
			case 'MissedPlayoffs':
				return 'Missed Playoffs';
			case 'WildCard':
				return 'Wild Card Round';
			case 'Divisional':
				return 'Divisional Round';
			case 'Conference':
				return 'Conference Championship';
			case 'SuperBowlLoss':
				return 'Super Bowl (Lost)';
			case 'SuperBowlWin':
				return 'Super Bowl Champions';
			default:
				return result;
		}
	}
</script>

<div class="space-y-6">
	<!-- Back Button -->
	<div>
		<button
			type="button"
			onclick={async () => {
				await goto('/teams');
			}}
			class="inline-flex items-center text-blue-600 hover:text-blue-700 font-medium"
		>
			<svg class="w-5 h-5 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Teams
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if error}
		<div class="bg-white rounded-lg shadow p-8 text-center">
			<div class="text-red-600 mb-4">
				<svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
					/>
				</svg>
			</div>
			<h2 class="text-xl font-semibold text-gray-800 mb-2">Team Not Found</h2>
			<p class="text-gray-600 mb-4">{error}</p>
			<button
				type="button"
				onclick={async () => {
					await goto('/teams');
				}}
				class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
			>
				Back to Teams
			</button>
		</div>
	{:else if team}
		<!-- Team Header -->
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-center gap-6">
				<img
					src={getTeamLogoPath(team.abbreviation)}
					alt="{team.name} logo"
					class="w-24 h-24 object-contain"
					onerror={(e) => ((e.currentTarget as HTMLImageElement).style.display = 'none')}
				/>
				<div class="flex-1">
					<h1 class="text-3xl font-bold text-gray-800">
						{team.city}
						{team.name}
					</h1>
					<div class="flex items-center gap-4 mt-2 text-sm text-gray-600">
						<span>{team.conference} {team.division}</span>
						{#if team.abbreviation}
							<Badge variant="primary">{team.abbreviation}</Badge>
						{/if}
					</div>
				</div>
			</div>
		</div>

		<!-- Team Card and Needs -->
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<!-- Team Information -->
			<div>
				<TeamCard {team} />
			</div>

			<!-- Team Needs -->
			<div>
				<TeamNeeds teamId={team.id} />
			</div>
		</div>

		<!-- Team's Draft Picks -->
		<div class="bg-white rounded-lg shadow p-6">
			<h2 class="text-2xl font-bold text-gray-800 mb-4">Draft Picks</h2>

			{#if picksLoading}
				<div class="flex justify-center py-8">
					<LoadingSpinner />
				</div>
			{:else if teamPicks.length === 0}
				<div class="text-center py-8 text-gray-600">
					<p>No draft picks available for this team in the current draft.</p>
					<p class="text-sm mt-2">Start a new draft to see team picks.</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each teamPicks as pick (pick.id)}
						<Card>
							<div class="flex items-center justify-between">
								<div>
									<div class="font-semibold text-gray-800">
										Round {pick.round}, Pick {pick.pick_number}
									</div>
									{#if pick.player_id}
										<div class="text-sm text-gray-600">
											Player selected (ID: {pick.player_id})
										</div>
									{:else}
										<div class="text-sm text-gray-600">Not yet selected</div>
									{/if}
								</div>
								<Badge variant={pick.player_id ? 'success' : 'primary'}>
									{pick.player_id ? 'Picked' : 'Available'}
								</Badge>
							</div>
						</Card>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Team Statistics -->
		<div class="bg-white rounded-lg shadow p-6">
			<h2 class="text-2xl font-bold text-gray-800 mb-4">{STANDINGS_SEASON_YEAR} Season</h2>
			{#if teamSeason}
				<div class="grid grid-cols-2 md:grid-cols-5 gap-4">
					<Card>
						<div class="text-center">
							<div class="text-3xl font-bold text-green-600">{teamSeason.wins}</div>
							<div class="text-sm text-gray-600 mt-1">Wins</div>
						</div>
					</Card>
					<Card>
						<div class="text-center">
							<div class="text-3xl font-bold text-red-600">{teamSeason.losses}</div>
							<div class="text-sm text-gray-600 mt-1">Losses</div>
						</div>
					</Card>
					<Card>
						<div class="text-center">
							<div class="text-3xl font-bold text-gray-500">{teamSeason.ties}</div>
							<div class="text-sm text-gray-600 mt-1">Ties</div>
						</div>
					</Card>
					<Card>
						<div class="text-center">
							<div class="text-3xl font-bold text-blue-600">
								{teamSeason.draft_position ?? '--'}
							</div>
							<div class="text-sm text-gray-600 mt-1">2026 Draft Pick</div>
						</div>
					</Card>
					<Card>
						<div class="text-center">
							<div class="text-3xl font-bold text-purple-600">
								{(teamSeason.win_percentage * 100).toFixed(1)}%
							</div>
							<div class="text-sm text-gray-600 mt-1">Win %</div>
						</div>
					</Card>
				</div>
				<div class="mt-4 text-center">
					<span class="text-gray-600">Record: </span>
					<span class="font-semibold">{formatRecord(teamSeason)}</span>
					<span class="mx-2">|</span>
					<span class="text-gray-600">Playoff Result: </span>
					<span class="font-semibold">{getPlayoffDisplay(teamSeason.playoff_result)}</span>
				</div>
			{:else}
				<div class="text-center py-4 text-gray-600">
					<p>No season data available.</p>
				</div>
			{/if}
		</div>
	{:else}
		<div class="bg-white rounded-lg shadow p-8 text-center">
			<p class="text-gray-600">Team not found.</p>
		</div>
	{/if}
</div>
