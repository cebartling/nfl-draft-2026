<script lang="ts">
	import Badge from '$components/ui/Badge.svelte';
	import { getTeamLogoPath } from '$lib/utils/logo';
	import type { Team, Conference, Division } from '$types';

	interface Props {
		teams: Team[];
		selectedTeamIds: string[];
		onSelectionChange: (ids: string[]) => void;
	}

	let { teams, selectedTeamIds, onSelectionChange }: Props = $props();

	let expandedDivisions = $state<Set<string>>(new Set());
	let failedLogos = $state<Set<string>>(new Set());

	const groupedTeams = $derived(() => {
		const groups = new Map<Conference, Map<Division, Team[]>>();

		teams.forEach((team) => {
			if (!groups.has(team.conference)) {
				groups.set(team.conference, new Map());
			}
			const conferenceGroups = groups.get(team.conference)!;
			if (!conferenceGroups.has(team.division)) {
				conferenceGroups.set(team.division, []);
			}
			conferenceGroups.get(team.division)!.push(team);
		});

		return groups;
	});

	const selectedTeams = $derived(
		teams.filter((t) => selectedTeamIds.includes(t.id))
	);

	function toggleTeam(teamId: string) {
		if (selectedTeamIds.includes(teamId)) {
			onSelectionChange(selectedTeamIds.filter((id) => id !== teamId));
		} else {
			onSelectionChange([...selectedTeamIds, teamId]);
		}
	}

	function toggleDivision(division: string) {
		const next = new Set(expandedDivisions);
		if (next.has(division)) {
			next.delete(division);
		} else {
			next.add(division);
		}
		expandedDivisions = next;
	}

	function selectAll() {
		onSelectionChange(teams.map((t) => t.id));
	}

	function clearAll() {
		onSelectionChange([]);
	}
</script>

<div class="space-y-4">
	<!-- Header with count and actions -->
	<div class="flex items-center justify-between">
		<span class="text-sm font-medium text-gray-700">
			{selectedTeamIds.length} team{selectedTeamIds.length !== 1 ? 's' : ''} selected
		</span>
		<div class="flex gap-2">
			<button
				type="button"
				onclick={selectAll}
				class="text-xs text-blue-600 hover:text-blue-800 font-medium"
			>
				Select All
			</button>
			<button
				type="button"
				onclick={clearAll}
				class="text-xs text-gray-500 hover:text-gray-700 font-medium"
			>
				Clear All
			</button>
		</div>
	</div>

	<!-- Selected teams badges -->
	{#if selectedTeams.length > 0}
		<div class="flex flex-wrap gap-1.5">
			{#each selectedTeams as team (team.id)}
				<button
					type="button"
					onclick={() => toggleTeam(team.id)}
					class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 hover:bg-blue-200 transition-colors"
				>
					{team.abbreviation}
					<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			{/each}
		</div>
	{/if}

	<!-- Conference/Division accordion -->
	{#each Array.from(groupedTeams()) as [conference, divisions] (conference)}
		<div class="border border-gray-200 rounded-lg overflow-hidden">
			<div class="bg-gray-50 px-4 py-2">
				<Badge variant={conference === 'AFC' ? 'primary' : 'danger'} size="sm">
					{conference}
				</Badge>
			</div>

			{#each Array.from(divisions) as [division, divisionTeams] (division)}
				<div class="border-t border-gray-200">
					<button
						type="button"
						class="w-full flex items-center justify-between px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
						onclick={() => toggleDivision(division)}
					>
						<span>{division}</span>
						<svg
							class="w-4 h-4 text-gray-400 transition-transform {expandedDivisions.has(division) ? 'rotate-180' : ''}"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
						</svg>
					</button>

					{#if expandedDivisions.has(division)}
						<div class="px-4 pb-3 grid grid-cols-1 sm:grid-cols-2 gap-2">
							{#each divisionTeams as team (team.id)}
								{@const isSelected = selectedTeamIds.includes(team.id)}
								<button
									type="button"
									onclick={() => toggleTeam(team.id)}
									class="flex items-center gap-3 p-2 rounded-lg border transition-all text-left {isSelected
										? 'border-blue-500 bg-blue-50'
										: 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'}"
								>
									{#if failedLogos.has(team.id)}
										<span class="inline-flex items-center justify-center w-8 h-8 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
											{team.abbreviation}
										</span>
									{:else}
										<img
											src={team.logo_url || getTeamLogoPath(team.abbreviation)}
											alt="{team.name} logo"
											class="w-8 h-8 object-contain"
											onerror={() => {
												failedLogos = new Set(failedLogos).add(team.id);
											}}
										/>
									{/if}
									<div class="flex-1 min-w-0">
										<div class="text-sm font-medium text-gray-900 truncate">
											{team.city} {team.name}
										</div>
										<div class="text-xs text-gray-500">{team.abbreviation}</div>
									</div>
									{#if isSelected}
										<svg class="w-5 h-5 text-blue-600 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
											<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
										</svg>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/each}
</div>
