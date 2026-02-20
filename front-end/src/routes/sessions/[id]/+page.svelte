<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { page } from '$app/stores';
	import { draftState } from '$stores/draft.svelte';
	import { toastState } from '$stores';
	import { draftsApi, sessionsApi } from '$lib/api';
	import DraftCommandCenter from '$components/draft/DraftCommandCenter.svelte';
	import DraftBoard from '$components/draft/DraftBoard.svelte';
	import PickActivityFeed from '$components/draft/PickActivityFeed.svelte';
	import PlayerList from '$components/player/PlayerList.svelte';
	import PlayerDetails from '$components/player/PlayerDetails.svelte';
	import Modal from '$components/ui/Modal.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import Tabs from '$components/ui/Tabs.svelte';
	import type { UUID, AvailablePlayer } from '$lib/types';

	let sessionId = $derived($page.params.id! as UUID);

	let selectedPlayer = $state<AvailablePlayer | null>(null);
	let detailPlayer = $state<AvailablePlayer | null>(null);
	let making_pick = $state(false);
	let players_loading = $state(true);
	let activeTab = $state('draft-board');
	let availablePlayers = $state<AvailablePlayer[]>([]);
	let playersLoaded = $state(false);

	const tabs = [
		{ id: 'draft-board', label: 'Draft Board' },
		{ id: 'available-players', label: 'Available Players' }
	];

	async function loadAvailablePlayers() {
		if (!draftState.session) return;
		players_loading = true;
		try {
			const teamId = draftState.controlledTeamIds[0];
			availablePlayers = await draftsApi.getAvailablePlayers(
				draftState.session.draft_id,
				teamId
			);
			playersLoaded = true;
		} catch (error) {
			logger.error('Failed to load available players:', error);
			toastState.error('Failed to load available players');
		} finally {
			players_loading = false;
		}
	}

	// Load available players once the session becomes available
	$effect(() => {
		const session = draftState.session;
		if (session && !playersLoaded) {
			loadAvailablePlayers();
		}
	});

	async function handleMakePick() {
		if (!selectedPlayer || !draftState.session || !draftState.currentPick) {
			return;
		}
		if (draftState.session.status !== 'InProgress') return;

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
			if (draftState.session?.auto_pick_enabled && !draftState.isCurrentPickUserControlled) {
				draftState.isAutoPickRunning = true;
				try {
					const result = await sessionsApi.autoPickRun(sessionId);
					draftState.session = result.session;
					// Reload picks to reflect AI picks
					await draftState.loadDraft(draftState.session.draft_id);
				} catch (err) {
					logger.error('Auto-pick run failed:', err);
					toastState.error('Auto-pick failed');
				} finally {
					draftState.isAutoPickRunning = false;
				}
			}

			// Refresh available players so the list reflects all picks just made
			await loadAvailablePlayers();

			toastState.success('Pick submitted');
		} catch (error: unknown) {
			logger.error('Failed to make pick:', error);
			// Check if the player was already drafted (stale list)
			const message =
				error instanceof Error && error.message.includes('already been drafted')
					? 'That player was already drafted — please pick another.'
					: 'Failed to make pick';
			toastState.error(message);
			selectedPlayer = null;
			// Refresh the list to remove stale players
			await loadAvailablePlayers();
		} finally {
			making_pick = false;
		}
	}

	function handleSelectPlayer(player: AvailablePlayer) {
		selectedPlayer = player;
	}

	function handleViewDetails(player: AvailablePlayer) {
		detailPlayer = player;
	}

	function handleCloseDetails() {
		detailPlayer = null;
	}
</script>

<div class="space-y-3">
	{#if !draftState.session}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else}
		<!-- Draft Command Center: Full-width clock + controls + selected player -->
		<DraftCommandCenter
			{sessionId}
			{selectedPlayer}
			makingPick={making_pick}
			onConfirmPick={handleMakePick}
			onCancelPick={() => (selectedPlayer = null)}
		/>

		<!-- Pick Activity Feed -->
		<PickActivityFeed />

		<!-- Tab Navigation -->
		<Tabs {tabs} {activeTab} onTabChange={(id) => (activeTab = id)} />

		<!-- Tab Panels -->
		<div id="tabpanel-draft-board" role="tabpanel" aria-labelledby="tab-draft-board" hidden={activeTab !== 'draft-board'}>
			{#if activeTab === 'draft-board'}
				<div class="bg-white rounded-lg shadow p-4">
					<h2 class="text-xl font-bold text-gray-800 mb-4">Draft Board</h2>
					<DraftBoard picks={draftState.picks} />
				</div>
			{/if}
		</div>

		<div id="tabpanel-available-players" role="tabpanel" aria-labelledby="tab-available-players" hidden={activeTab !== 'available-players'}>
			{#if activeTab === 'available-players'}
				<div class="bg-white rounded-lg shadow p-4">
					{#if players_loading}
						<div class="flex justify-center py-8">
							<LoadingSpinner />
						</div>
					{:else}
						<PlayerList
							players={availablePlayers}
							title="Available Players"
							onSelectPlayer={handleSelectPlayer}
							onViewDetails={handleViewDetails}
						/>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>

<!-- Player Detail Modal -->
<Modal open={detailPlayer !== null} onClose={handleCloseDetails} width="xl" title={`${detailPlayer?.first_name ?? ''} ${detailPlayer?.last_name ?? ''}`}>
	{#if detailPlayer}
		<PlayerDetails player={detailPlayer} />
	{/if}
</Modal>
