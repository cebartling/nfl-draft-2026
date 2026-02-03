<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { playersState } from '$stores/players.svelte';
	import PlayerDetails from '$components/player/PlayerDetails.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';

	let playerId = $derived($page.params.id!);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			await playersState.loadPlayer(playerId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load player';
			logger.error('Failed to load player:', e);
		} finally {
			loading = false;
		}
	});

	let player = $derived(() => {
		return playersState.allPlayers.find((p) => p.id === playerId);
	});
</script>

<div class="space-y-6">
	<!-- Back Button -->
	<div>
		<button
			type="button"
			onclick={() => goto('/players')}
			class="inline-flex items-center text-blue-600 hover:text-blue-700 font-medium"
		>
			<svg class="w-5 h-5 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M15 19l-7-7 7-7"
				/>
			</svg>
			Back to Players
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
			<h2 class="text-xl font-semibold text-gray-800 mb-2">Player Not Found</h2>
			<p class="text-gray-600 mb-4">{error}</p>
			<button
				type="button"
				onclick={() => goto('/players')}
				class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
			>
				Back to Players
			</button>
		</div>
	{:else if player()}
		<PlayerDetails player={player()!} />
	{:else}
		<div class="bg-white rounded-lg shadow p-8 text-center">
			<p class="text-gray-600">Player not found.</p>
		</div>
	{/if}
</div>
