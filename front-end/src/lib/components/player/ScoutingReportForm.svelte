<script lang="ts">
	import { Button } from '$components/ui';
	import { playersApi } from '$api';
	import { toastState } from '$stores';
	import { logger } from '$lib/utils/logger';
	import type { Player } from '$types';

	interface Props {
		player: Player;
		teamId: string;
		onSuccess?: () => void;
	}

	let { player, teamId, onSuccess }: Props = $props();

	let grade = $state(50);
	let notes = $state('');
	let strengths = $state('');
	let weaknesses = $state('');
	let isSubmitting = $state(false);

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!teamId) {
			toastState.error('Team ID is required');
			return;
		}

		isSubmitting = true;

		try {
			await playersApi.createScoutingReport({
				player_id: player.id,
				team_id: teamId,
				grade,
				notes: notes.trim() || undefined,
				strengths: strengths.trim() || undefined,
				weaknesses: weaknesses.trim() || undefined,
			});

			toastState.success('Scouting report created successfully');

			// Reset form
			grade = 50;
			notes = '';
			strengths = '';
			weaknesses = '';

			onSuccess?.();
		} catch (err) {
			toastState.error('Failed to create scouting report');
			logger.error('Failed to create scouting report:', err);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="bg-white rounded-lg shadow-md p-6">
	<h3 class="text-xl font-semibold text-gray-900 mb-6">
		Create Scouting Report for {player.first_name} {player.last_name}
	</h3>

	<form onsubmit={handleSubmit} class="space-y-6">
		<!-- Grade Slider -->
		<div>
			<label for="grade" class="block text-sm font-medium text-gray-700 mb-2">
				Grade: {grade}/100
			</label>
			<div class="flex items-center space-x-4">
				<input
					id="grade"
					type="range"
					bind:value={grade}
					min="1"
					max="100"
					step="1"
					class="flex-1"
					required
				/>
				<span class="text-lg font-semibold text-gray-900 w-12 text-right">
					{grade}
				</span>
			</div>
			<p class="text-xs text-gray-500 mt-1">1 = Poor, 100 = Elite</p>
		</div>

		<!-- Notes -->
		<div>
			<label for="notes" class="block text-sm font-medium text-gray-700 mb-2">
				Notes
			</label>
			<textarea
				id="notes"
				bind:value={notes}
				rows="4"
				class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
				placeholder="Overall evaluation and observations..."
			></textarea>
		</div>

		<!-- Strengths -->
		<div>
			<label for="strengths" class="block text-sm font-medium text-gray-700 mb-2">
				Strengths
			</label>
			<textarea
				id="strengths"
				bind:value={strengths}
				rows="3"
				class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
				placeholder="What does this player do well?"
			></textarea>
		</div>

		<!-- Weaknesses -->
		<div>
			<label for="weaknesses" class="block text-sm font-medium text-gray-700 mb-2">
				Weaknesses
			</label>
			<textarea
				id="weaknesses"
				bind:value={weaknesses}
				rows="3"
				class="w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
				placeholder="What areas need improvement?"
			></textarea>
		</div>

		<!-- Submit Button -->
		<div class="flex justify-end">
			<Button
				type="submit"
				variant="primary"
				disabled={isSubmitting}
				loading={isSubmitting}
			>
				Create Report
			</Button>
		</div>
	</form>
</div>
