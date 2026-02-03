<script lang="ts">
	import { Button, Badge, LoadingSpinner, Modal } from '$components/ui';
	import { teamsApi } from '$api';
	import { toastState } from '$stores';
	import { logger } from '$lib/utils/logger';
	import type { TeamNeed, Position } from '$types';
	import { PositionSchema } from '$types';

	interface Props {
		teamId: string;
	}

	let { teamId }: Props = $props();

	let needs = $state<TeamNeed[]>([]);
	let isLoading = $state(false);
	let showAddModal = $state(false);

	// New need form state
	let newPosition = $state<Position>('QB');
	let newPriority = $state(5);
	let newNotes = $state('');
	let isSubmitting = $state(false);

	const positions = PositionSchema.options;

	// Load needs on mount
	$effect(() => {
		if (teamId) {
			isLoading = true;
			teamsApi
				.getNeeds(teamId)
				.then((data) => {
					needs = data;
				})
				.catch((err) => {
					logger.error('Failed to load team needs:', err);
					toastState.error('Failed to load team needs');
				})
				.finally(() => {
					isLoading = false;
				});
		}
	});

	async function handleAddNeed(event: Event) {
		event.preventDefault();

		isSubmitting = true;

		try {
			const need = await teamsApi.createNeed({
				team_id: teamId,
				position: newPosition,
				priority: newPriority,
				notes: newNotes.trim() || undefined,
			});

			needs = [...needs, need];
			toastState.success('Team need added successfully');

			// Reset form
			newPosition = 'QB';
			newPriority = 5;
			newNotes = '';
			showAddModal = false;
		} catch (err) {
			toastState.error('Failed to add team need');
			logger.error('Failed to add team need:', err);
		} finally {
			isSubmitting = false;
		}
	}

	function getPriorityColor(priority: number): string {
		if (priority >= 8) return 'bg-red-500';
		if (priority >= 5) return 'bg-yellow-500';
		return 'bg-green-500';
	}

	function getPositionColor(position: string): 'primary' | 'danger' | 'info' {
		const offensePositions = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
		const defensePositions = ['DE', 'DT', 'LB', 'CB', 'S'];

		if (offensePositions.includes(position)) return 'primary';
		if (defensePositions.includes(position)) return 'danger';
		return 'info';
	}
</script>

<div class="bg-white rounded-lg shadow-md p-6">
	<div class="flex items-center justify-between mb-6">
		<h2 class="text-xl font-semibold text-gray-900">Team Needs</h2>
		<Button variant="primary" size="sm" onclick={() => (showAddModal = true)}>
			Add Need
		</Button>
	</div>

	{#if isLoading}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else if needs.length === 0}
		<p class="text-center text-gray-500 py-12">No team needs defined</p>
	{:else}
		<div class="space-y-4">
			{#each needs as need (need.id)}
				<div class="border border-gray-200 rounded-lg p-4">
					<div class="flex items-start justify-between mb-3">
						<Badge variant={getPositionColor(need.position)} size="lg">
							{need.position}
						</Badge>
						<div class="text-right">
							<p class="text-xs font-medium text-gray-600 mb-1">Priority</p>
							<div class="flex items-center space-x-2">
								<div class="w-24 bg-gray-200 rounded-full h-2">
									<div
										class="h-2 rounded-full {getPriorityColor(need.priority)}"
										style="width: {need.priority * 10}%"
									></div>
								</div>
								<span class="text-sm font-semibold text-gray-900">
									{need.priority}/10
								</span>
							</div>
						</div>
					</div>
					{#if need.notes}
						<p class="text-sm text-gray-600">{need.notes}</p>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Add Need Modal -->
<Modal bind:open={showAddModal} title="Add Team Need" width="md">
	<form onsubmit={handleAddNeed} class="space-y-4">
		<div>
			<label for="position" class="block text-sm font-medium text-gray-700 mb-2">
				Position
			</label>
			<select
				id="position"
				bind:value={newPosition}
				class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
				required
			>
				{#each positions as position (position)}
					<option value={position}>{position}</option>
				{/each}
			</select>
		</div>

		<div>
			<label for="priority" class="block text-sm font-medium text-gray-700 mb-2">
				Priority: {newPriority}/10
			</label>
			<input
				id="priority"
				type="range"
				bind:value={newPriority}
				min="1"
				max="10"
				step="1"
				class="w-full"
				required
			/>
			<div class="flex justify-between text-xs text-gray-500 mt-1">
				<span>Low</span>
				<span>High</span>
			</div>
		</div>

		<div>
			<label for="notes" class="block text-sm font-medium text-gray-700 mb-2">
				Notes
			</label>
			<textarea
				id="notes"
				bind:value={newNotes}
				rows="3"
				class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
				placeholder="Additional details about this need..."
			></textarea>
		</div>

		<div class="flex justify-end space-x-3">
			<Button variant="secondary" onclick={() => (showAddModal = false)}>
				Cancel
			</Button>
			<Button
				type="submit"
				variant="primary"
				disabled={isSubmitting}
				loading={isSubmitting}
			>
				Add Need
			</Button>
		</div>
	</form>
</Modal>
