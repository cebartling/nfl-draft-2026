<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { parseErrorMessage } from '$lib/utils/errors';
	import { goto } from '$app/navigation';
	import { draftsApi } from '$lib/api';
	import Card from '$components/ui/Card.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';

	const year = 2026;
	let rounds = $state(7);
	let submitting = $state(false);
	let error = $state<string | null>(null);

	// Validation
	let roundsError = $derived(
		rounds < 1 || rounds > 7 ? 'Rounds must be between 1 and 7' : null
	);
	let hasValidationErrors = $derived(!!roundsError);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (hasValidationErrors) return;

		error = null;
		submitting = true;

		try {
			const draft = await draftsApi.create({ year, rounds });
			await goto(`/drafts/${draft.id}`);
		} catch (e) {
			error = parseErrorMessage(e);
			logger.error('Failed to create draft:', e);
		} finally {
			submitting = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Back Button -->
	<div>
		<button
			type="button"
			onclick={async () => {
				await goto('/drafts');
			}}
			class="inline-flex items-center text-blue-600 hover:text-blue-700 font-medium"
		>
			<svg class="w-5 h-5 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Drafts
		</button>
	</div>

	<!-- Page Header -->
	<div>
		<h1 class="text-3xl font-bold text-gray-800">Create New Draft</h1>
		<p class="text-gray-600 mt-1">Configure your NFL draft simulation settings.</p>
	</div>

	<!-- Form Card -->
	<Card>
		<form onsubmit={handleSubmit} class="space-y-6">
			{#if error}
				<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
					<p class="font-medium">Error creating draft</p>
					<p class="text-sm">{error}</p>
				</div>
			{/if}

			<!-- Rounds Field -->
			<div>
				<label for="rounds" class="block text-sm font-medium text-gray-700 mb-2">
					Number of Rounds
				</label>
				<input
					type="number"
					id="rounds"
					bind:value={rounds}
					min="1"
					max="7"
					required
					disabled={submitting}
					class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed {roundsError ? 'border-red-500' : 'border-gray-300'}"
				/>
				{#if roundsError}
					<p class="text-sm text-red-600 mt-1">{roundsError}</p>
				{:else}
					<p class="text-sm text-gray-500 mt-1">Standard NFL drafts have 7 rounds.</p>
				{/if}
			</div>

			<!-- Summary -->
			<div class="bg-gray-50 rounded-lg p-4">
				<h3 class="text-sm font-medium text-gray-700 mb-2">Draft Summary</h3>
				<div class="grid grid-cols-3 gap-4 text-center">
					<div>
						<div class="text-2xl font-bold text-gray-800">{year}</div>
						<div class="text-xs text-gray-500">Year</div>
					</div>
					<div>
						<div class="text-2xl font-bold text-gray-800">{rounds}</div>
						<div class="text-xs text-gray-500">Rounds</div>
					</div>
					<div>
						<div class="text-lg font-bold text-blue-600">Realistic</div>
						<div class="text-xs text-gray-500">Draft Order: Loaded from schedule</div>
					</div>
				</div>
			</div>

			<!-- Submit Button -->
			<div class="flex justify-end gap-3">
				<button
					type="button"
					onclick={async () => {
						await goto('/drafts');
					}}
					disabled={submitting}
					class="px-6 py-2 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
				>
					Cancel
				</button>
				<button
					type="submit"
					disabled={submitting || hasValidationErrors}
					class="px-6 py-2 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center gap-2"
				>
					{#if submitting}
						<LoadingSpinner size="sm" />
						Creating...
					{:else}
						Create Draft
					{/if}
				</button>
			</div>
		</form>
	</Card>
</div>
