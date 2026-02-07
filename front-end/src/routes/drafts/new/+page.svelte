<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { parseErrorMessage } from '$lib/utils/errors';
	import { goto } from '$app/navigation';
	import { draftsApi } from '$lib/api';
	import Card from '$components/ui/Card.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';

	const year = 2026;
	let rounds = $state(1);
	let submitting = $state(false);
	let error = $state<string | null>(null);


	async function handleSubmit(event: Event) {
		event.preventDefault();

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
					Number of Rounds: <span class="text-blue-600 font-bold">{rounds}</span>
				</label>
				<input
					type="range"
					id="rounds"
					bind:value={rounds}
					min="1"
					max="7"
					step="1"
					disabled={submitting}
					class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
				/>
				<div class="flex justify-between text-xs text-gray-400 mt-1 px-0.5">
					{#each [1, 2, 3, 4, 5, 6, 7] as n}
						<span>{n}</span>
					{/each}
				</div>
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
					disabled={submitting}
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
