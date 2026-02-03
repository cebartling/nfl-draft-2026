<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { teamsApi } from '$lib/api';
	import TeamList from '$components/team/TeamList.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import type { Team } from '$lib/types';

	let teams = $state<Team[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let groupBy = $state<'conference' | 'division'>('conference');

	onMount(async () => {
		try {
			teams = await teamsApi.list();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load teams';
			logger.error('Failed to load teams:', e);
		} finally {
			loading = false;
		}
	});

	// Group teams by conference
	let teamsByConference = $derived(() => {
		const groups: Record<string, Team[]> = {
			AFC: [],
			NFC: [],
		};

		teams.forEach((team) => {
			if (team.conference) {
				if (!groups[team.conference]) {
					groups[team.conference] = [];
				}
				groups[team.conference].push(team);
			}
		});

		// Sort teams within each conference by city
		Object.keys(groups).forEach((conf) => {
			groups[conf].sort((a, b) => a.city.localeCompare(b.city));
		});

		return groups;
	});

	// Group teams by division
	let teamsByDivision = $derived(() => {
		const groups: Record<string, Team[]> = {};

		teams.forEach((team) => {
			if (team.division) {
				const key = `${team.conference} ${team.division}`;
				if (!groups[key]) {
					groups[key] = [];
				}
				groups[key].push(team);
			}
		});

		// Sort teams within each division by city
		Object.keys(groups).forEach((div) => {
			groups[div].sort((a, b) => a.city.localeCompare(b.city));
		});

		return groups;
	});

	async function handleSelectTeam(team: Team) {
		await goto(`/teams/${team.id}`);
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold text-gray-800">Teams</h1>
		<div class="text-sm text-gray-600">
			{teams.length} teams
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if error}
		<div class="bg-white rounded-lg shadow p-8 text-center">
			<div class="text-red-600 mb-4">
				<p class="font-semibold">Error loading teams</p>
				<p class="text-sm">{error}</p>
			</div>
		</div>
	{:else}
		<!-- Group By Selector -->
		<div class="bg-white rounded-lg shadow p-4">
			<div class="flex items-center gap-4">
				<span class="text-sm font-medium text-gray-700">Group by:</span>
				<div class="flex gap-2">
					<button
						type="button"
						onclick={() => (groupBy = 'conference')}
						class={`px-4 py-2 rounded-lg font-medium transition-colors ${
							groupBy === 'conference'
								? 'bg-blue-600 text-white'
								: 'bg-gray-200 text-gray-700 hover:bg-gray-300'
						}`}
					>
						Conference
					</button>
					<button
						type="button"
						onclick={() => (groupBy = 'division')}
						class={`px-4 py-2 rounded-lg font-medium transition-colors ${
							groupBy === 'division'
								? 'bg-blue-600 text-white'
								: 'bg-gray-200 text-gray-700 hover:bg-gray-300'
						}`}
					>
						Division
					</button>
				</div>
			</div>
		</div>

		<!-- Teams Grouped by Conference -->
		{#if groupBy === 'conference'}
			<div class="space-y-6">
				{#each Object.entries(teamsByConference()) as [conference, conferenceTeams] (conference)}
					<div class="bg-white rounded-lg shadow p-6">
						<h2 class="text-2xl font-bold text-gray-800 mb-4">
							{conference}
							<span class="text-sm text-gray-600 font-normal ml-2">
								({conferenceTeams.length} teams)
							</span>
						</h2>
						<TeamList teams={conferenceTeams} onSelectTeam={handleSelectTeam} />
					</div>
				{/each}
			</div>
		{/if}

		<!-- Teams Grouped by Division -->
		{#if groupBy === 'division'}
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
				{#each Object.entries(teamsByDivision()).sort() as [division, divisionTeams] (division)}
					<div class="bg-white rounded-lg shadow p-6">
						<h2 class="text-xl font-bold text-gray-800 mb-4">
							{division}
							<span class="text-sm text-gray-600 font-normal ml-2">
								({divisionTeams.length})
							</span>
						</h2>
						<TeamList teams={divisionTeams} onSelectTeam={handleSelectTeam} />
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>
