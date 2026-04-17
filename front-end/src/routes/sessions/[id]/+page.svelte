<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { page } from '$app/stores';
	import { draftState } from '$stores/draft.svelte';
	import { toastState, tradesState } from '$stores';
	import { draftsApi, sessionsApi } from '$lib/api';
	import DraftCommandCenter from '$components/draft/DraftCommandCenter.svelte';
	import DraftBoard from '$components/draft/DraftBoard.svelte';
	import PickActivityFeed from '$components/draft/PickActivityFeed.svelte';

	import PlayerList from '$components/player/PlayerList.svelte';
	import PlayerDetails from '$components/player/PlayerDetails.svelte';
	import Modal from '$components/ui/Modal.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import Tabs from '$components/ui/Tabs.svelte';
	import TradeBuilder from '$components/trade/TradeBuilder.svelte';
	import TradeHistory from '$components/trade/TradeHistory.svelte';
	import type { UUID, AvailablePlayer } from '$lib/types';

	let sessionId = $derived($page.params.id! as UUID);

	let selectedPlayer = $state<AvailablePlayer | null>(null);
	let detailPlayer = $state<AvailablePlayer | null>(null);
	let making_pick = $state(false);
	let players_loading = $state(true);
	let activeTab = $state('draft-board');
	let availablePlayers = $state<AvailablePlayer[]>([]);

	const tabs = [
		{ id: 'draft-board', label: 'Draft Board' },
		{ id: 'available-players', label: 'Available Players' },
		{ id: 'trades', label: 'Trades' },
	];

	// Available picks = picks that haven't been used yet (no player assigned)
	const unusedPicks = $derived(draftState.picks.filter((p) => p.player_id == null));

	async function loadAvailablePlayers() {
		if (!draftState.session) return;
		players_loading = true;
		try {
			const teamId = draftState.controlledTeamIds[0];
			availablePlayers = await draftsApi.getAvailablePlayers(draftState.session.draft_id, teamId);
		} catch (error) {
			logger.error('Failed to load available players:', error);
			toastState.error('Failed to load available players');
		} finally {
			players_loading = false;
		}
	}

	const completedPickCount = $derived(draftState.picks.filter((p) => p.player_id != null).length);
	let fetchedForPickCount = -1;

	$effect(() => {
		const session = draftState.session;
		const count = completedPickCount;
		if (session && count !== fetchedForPickCount) {
			fetchedForPickCount = count;
			loadAvailablePlayers();
		}
	});

	// Load trades for this session once
	let loadedTradesForSession: string | null = null;
	$effect(() => {
		if (sessionId && loadedTradesForSession !== sessionId) {
			loadedTradesForSession = sessionId;
			tradesState.load(sessionId);
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

			selectedPlayer = null;

			const updatedSession = await sessionsApi.advancePick(sessionId);
			draftState.session = updatedSession;

			await draftState.loadDraft(draftState.session.draft_id);

			if (draftState.session?.auto_pick_enabled && !draftState.isCurrentPickUserControlled) {
				draftState.isAutoPickRunning = true;
				try {
					const result = await sessionsApi.autoPickRun(sessionId);
					draftState.session = result.session;
					await draftState.loadDraft(draftState.session.draft_id);
				} catch (err) {
					logger.error('Auto-pick run failed:', err);
					toastState.error('Auto-pick failed');
				} finally {
					draftState.isAutoPickRunning = false;
				}
			}

			toastState.success('Pick submitted');
		} catch (error: unknown) {
			logger.error('Failed to make pick:', error);
			const message =
				error instanceof Error && error.message.includes('already been drafted')
					? 'That player was already drafted — please pick another.'
					: 'Failed to make pick';
			toastState.error(message);
			selectedPlayer = null;
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

	async function handleTradeRespond(
		tradeId: string,
		teamId: string,
		action: 'accept' | 'reject'
	) {
		if (action === 'accept') {
			await tradesState.accept(tradeId, teamId);
		} else {
			await tradesState.reject(tradeId, teamId);
		}
	}
</script>

<div class="space-y-3">
	{#if !draftState.session}
		<div class="flex justify-center py-12">
			<LoadingSpinner size="lg" />
		</div>
	{:else}
		<DraftCommandCenter
			{sessionId}
			{selectedPlayer}
			makingPick={making_pick}
			onConfirmPick={handleMakePick}
			onCancelPick={() => (selectedPlayer = null)}
		/>

		<PickActivityFeed />

		<Tabs {tabs} {activeTab} onTabChange={(id) => (activeTab = id)} />

		<div
			id="tabpanel-draft-board"
			role="tabpanel"
			aria-labelledby="tab-draft-board"
			hidden={activeTab !== 'draft-board'}
		>
			{#if activeTab === 'draft-board'}
				<div class="bg-white rounded-lg shadow p-4">
					<h2 class="text-xl font-bold text-gray-800 mb-4">Draft Board</h2>
					<DraftBoard picks={draftState.picks} />
				</div>
			{/if}
		</div>

		<div
			id="tabpanel-available-players"
			role="tabpanel"
			aria-labelledby="tab-available-players"
			hidden={activeTab !== 'available-players'}
		>
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

		<div
			id="tabpanel-trades"
			role="tabpanel"
			aria-labelledby="tab-trades"
			hidden={activeTab !== 'trades'}
		>
			{#if activeTab === 'trades'}
				<div class="space-y-4">
					<TradeBuilder
						{sessionId}
						availablePicks={unusedPicks}
						onSuccess={() => tradesState.load(sessionId)}
					/>
					<TradeHistory
						proposals={tradesState.proposals}
						isLoading={tradesState.isLoading}
						currentTeamIds={draftState.controlledTeamIds}
						onRespond={handleTradeRespond}
					/>
				</div>
			{/if}
		</div>
	{/if}
</div>

<Modal
	open={detailPlayer !== null}
	onClose={handleCloseDetails}
	width="xl"
	title={`${detailPlayer?.first_name ?? ''} ${detailPlayer?.last_name ?? ''}`}
>
	{#if detailPlayer}
		<PlayerDetails player={detailPlayer} />
	{/if}
</Modal>
