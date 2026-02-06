<script lang="ts">
	import { draftState } from '$stores';
	import { teamsApi } from '$api';
	import { Badge, LoadingSpinner } from '$components/ui';
	import type { Team, UUID } from '$types';
	import { logger } from '$lib/utils/logger';

	interface Props {
		sessionId: UUID;
	}

	let { sessionId }: Props = $props();

	let team = $state<Team | null>(null);
	let timeRemaining = $state(0);
	let isLoading = $state(false);

	// Calculate time remaining based on session time per pick
	$effect(() => {
		const session = draftState.session;
		if (!session || session.status !== 'InProgress') {
			timeRemaining = 0;
			return;
		}

		timeRemaining = session.time_per_pick_seconds;

		const interval = setInterval(() => {
			if (timeRemaining > 0) {
				timeRemaining -= 1;
			} else {
				clearInterval(interval);
			}
		}, 1000);

		return () => clearInterval(interval);
	});

	// Load team data when current pick changes
	$effect(() => {
		const currentPick = draftState.currentPick;
		if (currentPick?.team_id) {
			isLoading = true;
			teamsApi
				.get(currentPick.team_id)
				.then((t) => {
					team = t;
				})
				.catch((err) => {
					logger.error('Failed to load team:', err);
				})
				.finally(() => {
					isLoading = false;
				});
		}
	});

	function formatTime(seconds: number): string {
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}
</script>

<div class="bg-white rounded-lg shadow-md p-6">
	<div class="flex items-center justify-between mb-4">
		<div>
			<h2 class="text-2xl font-bold text-gray-900">Draft Clock</h2>
			<p class="text-sm text-gray-600 mt-1">
				Pick #{draftState.currentPickNumber}
			</p>
		</div>
		<Badge variant="info" size="lg">
			{#if draftState.currentPick}
				Round {draftState.currentPick.round}
			{:else}
				-
			{/if}
		</Badge>
	</div>

	<div class="space-y-4">
		<div class="text-center">
			<div
				class="text-6xl font-bold tabular-nums {timeRemaining < 10
					? 'text-red-600 animate-pulse-slow'
					: 'text-gray-900'}"
			>
				{formatTime(timeRemaining)}
			</div>
			<p class="text-sm text-gray-600 mt-2">Time Remaining</p>
		</div>

		<div class="border-t border-gray-200 pt-4">
			<p class="text-sm font-medium text-gray-600 mb-2">Team on the Clock</p>
			{#if isLoading}
				<div class="flex justify-center py-4">
					<LoadingSpinner size="md" />
				</div>
			{:else if team}
				<div class="flex items-center space-x-3">
					<div class="flex-1">
						<p class="text-lg font-semibold text-gray-900">{team.city} {team.name}</p>
						<p class="text-sm text-gray-600">{team.abbreviation}</p>
					</div>
				</div>
			{:else}
				<p class="text-gray-500">No team on the clock</p>
			{/if}
		</div>
	</div>
</div>
