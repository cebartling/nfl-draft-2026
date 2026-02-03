<script lang="ts">
	import { TeamCard } from '$components/team';
	import { Badge } from '$components/ui';
	import type { Team, Conference, Division } from '$types';

	interface Props {
		teams: Team[];
		onSelectTeam?: (team: Team) => void;
	}

	let { teams, onSelectTeam }: Props = $props();

	let expandedDivisions = $state<Set<string>>(new Set());

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

	function toggleDivision(division: string) {
		const newSet = new Set(expandedDivisions);
		if (newSet.has(division)) {
			newSet.delete(division);
		} else {
			newSet.add(division);
		}
		expandedDivisions = newSet;
	}

	function getConferenceColor(conference: string): 'primary' | 'danger' {
		return conference === 'AFC' ? 'primary' : 'danger';
	}
</script>

<div class="bg-gray-50 rounded-lg p-6">
	<h2 class="text-2xl font-bold text-gray-900 mb-6">NFL Teams</h2>

	<div class="space-y-6">
		{#each Array.from(groupedTeams()) as [conference, divisions]}
			<div>
				<div class="flex items-center space-x-3 mb-4">
					<Badge variant={getConferenceColor(conference)} size="lg">
						{conference}
					</Badge>
					<span class="text-sm text-gray-600">
						{teams.filter((t) => t.conference === conference).length} teams
					</span>
				</div>

				<div class="space-y-4">
					{#each Array.from(divisions) as [division, divisionTeams]}
						<div class="border border-gray-200 rounded-lg overflow-hidden">
							<button
								type="button"
								class="w-full px-4 py-3 bg-white hover:bg-gray-50 transition-colors flex items-center justify-between"
								onclick={() => toggleDivision(division)}
							>
								<div class="flex items-center space-x-3">
									<Badge variant="default" size="md">
										{division}
									</Badge>
									<span class="text-sm text-gray-600">
										{divisionTeams.length} teams
									</span>
								</div>
								<svg
									class="w-5 h-5 text-gray-400 transition-transform {expandedDivisions.has(
										division
									)
										? 'rotate-180'
										: ''}"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M19 9l-7 7-7-7"
									/>
								</svg>
							</button>

							{#if expandedDivisions.has(division)}
								<div class="p-4 bg-gray-50 border-t border-gray-200">
									<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
										{#each divisionTeams as team (team.id)}
											<TeamCard {team} onSelect={onSelectTeam} />
										{/each}
									</div>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/each}
	</div>
</div>
