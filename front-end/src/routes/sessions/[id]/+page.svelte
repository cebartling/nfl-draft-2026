<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { page } from '$app/stores';
	import { draftState } from '$stores/draft.svelte';
	import { toastState } from '$stores';
	import { playersState } from '$stores/players.svelte';
	import { websocketState } from '$stores/websocket.svelte';
	import { draftsApi, sessionsApi } from '$lib/api';
	import DraftCommandCenter from '$components/draft/DraftCommandCenter.svelte';
	import DraftBoard from '$components/draft/DraftBoard.svelte';
	import PlayerList from '$components/player/PlayerList.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import Badge from '$components/ui/Badge.svelte';
	import { onMount } from 'svelte';
	import type { Player, UUID } from '$lib/types';

	let sessionId = $derived($page.params.id! as UUID);

	let selectedPlayer = $state<Player | null>(null);
	let making_pick = $state(false);
	let players_loading = $state(true);

	onMount(async () => {
		// Load all players
		try {
			await playersState.loadAll();
		} catch (error) {
			logger.error('Failed to load players:', error);
		} finally {
			players_loading = false;
		}
	});

	// Get available players (filter out already picked)
	let availablePlayers = $derived(() => {
		const pickedPlayerIds = new Set(draftState.picks.map((p) => p.player_id));
		return playersState.allPlayers.filter((p) => !pickedPlayerIds.has(p.id));
	});

	async function handleMakePick() {
		if (!selectedPlayer || !draftState.session || !draftState.currentPick) {
			return;
		}

		making_pick = true;
		try {
			await draftsApi.makePick(
				draftState.session.draft_id,
				draftState.currentPick.id,
				selectedPlayer.id
			);

			// Clear selection after successful pick
			selectedPlayer = null;

			// Advance pick number on the server and locally
			const updatedSession = await sessionsApi.advancePick(sessionId);
			draftState.session = updatedSession;

			// Reload picks to reflect the manual pick
			await draftState.loadDraft(draftState.session.draft_id);

			// Trigger AI auto-picks for subsequent AI teams
			if (draftState.hasControlledTeams && !draftState.isCurrentPickUserControlled) {
				try {
					const result = await sessionsApi.autoPickRun(sessionId);
					draftState.session = result.session;
					// Reload picks to reflect AI picks
					await draftState.loadDraft(draftState.session.draft_id);
				} catch (err) {
					logger.error('Auto-pick run failed:', err);
					toastState.error('Auto-pick failed');
				}
			}
		} catch (error) {
			logger.error('Failed to make pick:', error);
		} finally {
			making_pick = false;
		}
	}

	function handleSelectPlayer(player: Player) {
		selectedPlayer = player;
	}
</script>

<div class="space-y-6">
	<!-- Connection Status -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold text-gray-800">Draft Room</h1>
		<div class="flex items-center gap-2">
			<Badge variant={websocketState.isConnected ? 'success' : 'danger'}>
				{websocketState.isConnected ? '● Connected' : '○ Disconnected'}
			</Badge>
			{#if websocketState.lastMessage}
				<span class="text-sm text-gray-600">
					Last update: {new Date().toLocaleTimeString()}
				</span>
			{/if}
		</div>
	</div>

	{#if !draftState.session}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else}
		<!-- Draft Command Center: Full-width clock + controls -->
		<DraftCommandCenter {sessionId} />

		<!-- Draft Room Layout: 2-column -->
		<div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
			<!-- Left Column: Draft Board + On The Clock + Selected Player -->
			<div class="lg:col-span-8 space-y-4">
				<div class="bg-white rounded-lg shadow p-4">
					<h2 class="text-xl font-bold text-gray-800 mb-4">Draft Board</h2>
					<DraftBoard picks={draftState.picks} />
				</div>

				<!-- Current Pick Info -->
				{#if draftState.currentPick}
					<div class="bg-white rounded-lg shadow p-4 border-2 {draftState.hasControlledTeams && !draftState.isCurrentPickUserControlled ? 'border-gray-300' : 'border-blue-500'}">
						<div class="flex items-center gap-2 mb-2">
							<h3 class="text-sm font-semibold text-gray-600">ON THE CLOCK</h3>
							{#if draftState.hasControlledTeams}
								{#if draftState.isCurrentPickUserControlled}
									<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-bold bg-blue-600 text-white">
										YOUR PICK
									</span>
								{:else}
									<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-bold bg-gray-500 text-white">
										AI PICK
									</span>
								{/if}
							{/if}
						</div>
						<div class="space-y-2">
							<div class="text-lg font-bold text-gray-800">
								Team {draftState.currentPick.team_id}
							</div>
							<div class="text-sm text-gray-600">
								Round {draftState.currentPick.round}, Pick {draftState.currentPick.pick_number}
							</div>
							<div class="text-sm text-gray-600">
								Overall Pick: {draftState.currentPick.overall_pick}
							</div>
						</div>
					</div>
				{/if}

				<!-- Selected Player (only show when user controls current pick or no controlled teams) -->
				{#if selectedPlayer && (!draftState.hasControlledTeams || draftState.isCurrentPickUserControlled)}
					<div class="bg-white rounded-lg shadow p-4 border-2 border-green-500">
						<h3 class="text-sm font-semibold text-gray-600 mb-2">SELECTED PLAYER</h3>
						<div class="space-y-2">
							<div class="text-lg font-bold text-gray-800">
								{selectedPlayer.first_name}
								{selectedPlayer.last_name}
							</div>
							<div class="text-sm text-gray-600">
								{selectedPlayer.position} - {selectedPlayer.college}
							</div>
							<button
								type="button"
								onclick={handleMakePick}
								disabled={making_pick}
								class="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
							>
								{making_pick ? 'Making Pick...' : 'Confirm Pick'}
							</button>
							<button
								type="button"
								onclick={() => (selectedPlayer = null)}
								class="w-full bg-gray-200 hover:bg-gray-300 text-gray-800 font-medium py-2 px-4 rounded-lg transition-colors"
							>
								Cancel
							</button>
						</div>
					</div>
				{:else if draftState.hasControlledTeams && !draftState.isCurrentPickUserControlled && draftState.session?.status === 'InProgress'}
					<div class="bg-white rounded-lg shadow p-4 border-2 border-gray-300">
						<div class="text-center py-4">
							<div class="text-gray-400 mb-2">
								<svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
								</svg>
							</div>
							<p class="text-sm font-medium text-gray-600">AI is selecting...</p>
							<p class="text-xs text-gray-400 mt-1">Waiting for AI to make this pick</p>
						</div>
					</div>
				{/if}
			</div>

			<!-- Right Column: Available Players -->
			<div class="lg:col-span-4">
				<div class="bg-white rounded-lg shadow p-4">
					<h2 class="text-xl font-bold text-gray-800 mb-4">Available Players</h2>
					{#if players_loading}
						<div class="flex justify-center py-8">
							<LoadingSpinner />
						</div>
					{:else}
						<PlayerList
							players={availablePlayers()}
							title="Available Players"
							onSelectPlayer={handleSelectPlayer}
						/>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>
