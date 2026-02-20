<script lang="ts">
	import { draftState } from '$stores';

	const reversedNotifications = $derived(
		[...draftState.pickNotifications].reverse()
	);
</script>

{#if draftState.pickNotifications.length > 0}
	<div class="bg-white rounded-lg shadow p-4">
		<h3 class="text-sm font-semibold text-gray-700 mb-2">Pick Activity</h3>
		<div class="space-y-1 max-h-48 overflow-y-auto">
			{#each reversedNotifications as notification (notification.pick_id)}
				<div
					data-testid="pick-notification"
					class="flex items-center gap-2 text-sm px-2 py-1.5 rounded bg-gray-50"
				>
					<span class="font-medium text-gray-500 shrink-0">
						R{notification.round} P{notification.pick_number}
					</span>
					<span class="font-semibold text-gray-900 truncate">
						{notification.team_name}
					</span>
					<span class="text-gray-400 shrink-0">&rarr;</span>
					<span class="text-gray-700 truncate">
						{notification.player_name}
					</span>
				</div>
			{/each}
		</div>
	</div>
{/if}
