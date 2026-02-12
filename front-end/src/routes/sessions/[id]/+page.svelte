<script lang="ts">
	import { logger } from '$lib/utils/logger';
	import { page } from '$app/stores';
	import { draftState } from '$stores/draft.svelte';
	import { toastState } from '$stores';
	import { playersState } from '$stores/players.svelte';
	import { draftsApi, sessionsApi, teamsApi, rankingsApi } from '$lib/api';
	import DraftCommandCenter from '$components/draft/DraftCommandCenter.svelte';
	import DraftBoard from '$components/draft/DraftBoard.svelte';
	import PlayerList from '$components/player/PlayerList.svelte';
	import PlayerDetails from '$components/player/PlayerDetails.svelte';
	import Modal from '$components/ui/Modal.svelte';
	import LoadingSpinner from '$components/ui/LoadingSpinner.svelte';
	import Tabs from '$components/ui/Tabs.svelte';
	import { onMount } from 'svelte';
	import type { Player, UUID, RankingBadge } from '$lib/types';
	import { sortByScoutingGrade } from '$lib/utils/player-sort';

	let sessionId = $derived($page.params.id! as UUID);

	let selectedPlayer = $state<Player | null>(null);
	let detailPlayer = $state<Player | null>(null);
	let making_pick = $state(false);
	let players_loading = $state(true);
	let activeTab = $state('draft-board');
	let scoutingGrades = $state<Map<string, number>>(new Map());
	let playerRankings = $state<Map<string, RankingBadge[]>>(new Map());
	let rankingsLoaded = $state(false);

	const tabs = [
		{ id: 'draft-board', label: 'Draft Board' },
		{ id: 'available-players', label: 'Available Players' }
	];

	onMount(async () => {
		try {
			await playersState.loadAll();
		} catch (error) {
			logger.error('Failed to load players:', error);
		} finally {
			players_loading = false;
		}

		// Load rankings alongside players (awaited, not fire-and-forget)
		if (!rankingsLoaded) {
			try {
				const rankings = await rankingsApi.loadAllPlayerRankings();
				playerRankings = rankings;
				rankingsLoaded = true;
			} catch (error) {
				logger.error('Failed to load rankings:', error);
				toastState.warning('Rankings unavailable');
			}
		}
	});

	// Reactively load scouting grades when controlled team changes
	// Plain variable (not $state) to avoid the effect tracking it as a dependency.
	// Using $state here would cause an infinite loop: the effect reads and writes
	// the version counter, triggering itself repeatedly.
	let scoutingGradesVersion = 0;
	$effect(() => {
		const controlledTeamId = draftState.controlledTeamIds[0];
		if (controlledTeamId) {
			const currentVersion = ++scoutingGradesVersion;
			teamsApi
				.getScoutingReports(controlledTeamId)
				.then((reports) => {
					// Guard against stale responses
					if (currentVersion !== scoutingGradesVersion) return;
					const grades = new Map<string, number>();
					for (const report of reports) {
						grades.set(report.player_id, report.grade);
					}
					scoutingGrades = grades;
				})
				.catch((error) => {
					logger.error('Failed to load scouting grades:', error);
				});
		}
	});

	// Get available players (filter out already picked), sorted by scouting grade
	let availablePlayers = $derived.by(() => {
		const pickedPlayerIds = new Set(draftState.picks.map((p) => p.player_id));
		const available = playersState.allPlayers.filter((p) => !pickedPlayerIds.has(p.id));
		return sortByScoutingGrade(available, scoutingGrades);
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

			toastState.success('Pick submitted');
		} catch (error) {
			logger.error('Failed to make pick:', error);
			toastState.error('Failed to make pick');
		} finally {
			making_pick = false;
		}
	}

	function handleSelectPlayer(player: Player) {
		selectedPlayer = player;
	}

	function handleViewDetails(player: Player) {
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
							{scoutingGrades}
							{playerRankings}
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
<Modal open={detailPlayer !== null} onClose={handleCloseDetails} width="xl" title="{detailPlayer?.first_name ?? ''} {detailPlayer?.last_name ?? ''}">
	{#if detailPlayer}
		<PlayerDetails player={detailPlayer} />
	{/if}
</Modal>
