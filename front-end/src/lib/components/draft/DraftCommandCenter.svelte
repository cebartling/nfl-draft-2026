<script lang="ts">
	import { draftState, toastState } from '$stores';
	import { teamsApi, sessionsApi } from '$api';
	import { Badge, Button, LoadingSpinner } from '$components/ui';
	import { getTeamLogoPath } from '$lib/utils/logo';
	import type { ChartType, Team, UUID } from '$types';
	import { logger } from '$lib/utils/logger';

	interface Props {
		sessionId: UUID;
	}

	let { sessionId }: Props = $props();

	// --- Clock state ---
	let team = $state<Team | null>(null);
	let controlledTeams = $state<Map<string, Team>>(new Map());
	let failedLogos = $state<Set<string>>(new Set());
	let timeRemaining = $state(0);
	let isLoadingTeam = $state(false);

	// --- Controls state ---
	const chartTypes: ChartType[] = [
		'JimmyJohnson',
		'RichHill',
		'ChaseStudartAV',
		'FitzgeraldSpielberger',
		'PffWar',
		'SurplusValue',
	];

	let selectedChartType = $state<ChartType>('JimmyJohnson');
	let autoPickEnabled = $state(false);
	let timePerPick = $state(120);

	// --- Clock effects ---
	// Track only the values that should restart the clock to avoid resetting
	// the timer when unrelated session fields change (e.g., after auto-pick-run).
	$effect(() => {
		const status = draftState.session?.status;
		const pickNumber = draftState.currentPickNumber;
		const isUserPick = draftState.isCurrentPickUserControlled;
		const hasControlled = draftState.hasControlledTeams;
		const timePerPickSeconds = draftState.session?.time_per_pick_seconds ?? 0;

		if (status !== 'InProgress') {
			timeRemaining = 0;
			return;
		}

		// Only run clock for user-controlled picks (or when no teams are controlled)
		if (hasControlled && !isUserPick) {
			timeRemaining = 0;
			return;
		}

		// Timer resets when pickNumber (read above) or timePerPickSeconds changes
		timeRemaining = timePerPickSeconds;

		const interval = setInterval(() => {
			if (timeRemaining > 0) {
				timeRemaining -= 1;
			} else {
				clearInterval(interval);
			}
		}, 1000);

		return () => clearInterval(interval);
	});

	$effect(() => {
		const currentPick = draftState.currentPick;
		if (currentPick?.team_id) {
			isLoadingTeam = true;
			teamsApi
				.get(currentPick.team_id)
				.then((t) => {
					team = t;
				})
				.catch((err) => {
					logger.error('Failed to load team:', err);
					toastState.error('Failed to load current team');
				})
				.finally(() => {
					isLoadingTeam = false;
				});
		}
	});

	// --- Load controlled teams ---
	$effect(() => {
		const ids = draftState.controlledTeamIds;
		if (ids.length > 0 && controlledTeams.size === 0) {
			Promise.all(ids.map((id) => teamsApi.get(id)))
				.then((teams) => {
					controlledTeams = new Map(teams.map((t) => [t.id, t]));
				})
				.catch((err) => {
					logger.error('Failed to load controlled teams:', err);
					toastState.error('Failed to load team data');
				});
		}
	});

	// --- Controls effects ---
	$effect(() => {
		if (draftState.session) {
			selectedChartType = draftState.session.chart_type;
			autoPickEnabled = draftState.session.auto_pick_enabled;
			timePerPick = draftState.session.time_per_pick_seconds;
		}
	});

	// --- Handlers ---
	async function handleStart() {
		try {
			await draftState.startSession(sessionId);
			toastState.success('Draft session started');
			// Trigger AI auto-picks if current pick is not user-controlled
			await triggerAutoPickRun();
		} catch (err) {
			toastState.error('Failed to start session');
		}
	}

	async function triggerAutoPickRun() {
		if (!draftState.session?.auto_pick_enabled) return;
		if (draftState.isCurrentPickUserControlled) return;
		if (draftState.isAutoPickRunning) return;

		draftState.isAutoPickRunning = true;
		try {
			const result = await sessionsApi.autoPickRun(sessionId);
			draftState.session = result.session;
			// Reload picks to reflect AI picks
			if (draftState.session) {
				await draftState.loadDraft(draftState.session.draft_id);
			}
		} catch (err) {
			logger.error('Auto-pick run failed:', err);
			toastState.error('Auto-pick failed');
		} finally {
			draftState.isAutoPickRunning = false;
		}
	}

	async function handlePause() {
		try {
			await draftState.pauseSession(sessionId);
			toastState.success('Draft session paused');
		} catch (err) {
			toastState.error('Failed to pause session');
		}
	}

	// --- Derived ---
	function getStatusBadge() {
		const status = draftState.session?.status;
		if (!status) return { variant: 'default' as const, text: 'Unknown' };

		switch (status) {
			case 'NotStarted':
				return { variant: 'default' as const, text: 'Not Started' };
			case 'InProgress':
				return { variant: 'success' as const, text: 'In Progress' };
			case 'Paused':
				return { variant: 'warning' as const, text: 'Paused' };
			case 'Completed':
				return { variant: 'info' as const, text: 'Completed' };
			default:
				return { variant: 'default' as const, text: 'Unknown' };
		}
	}

	const statusBadge = $derived(getStatusBadge());

	function formatTime(seconds: number): string {
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}
</script>

<div class="bg-white rounded-lg shadow-md p-4 lg:p-6 space-y-4">
	<!-- Row 1: Clock, Round/Pick, Team on the Clock -->
	<div class="flex flex-col lg:flex-row lg:items-center lg:divide-x lg:divide-gray-200 gap-4 lg:gap-0">
		<!-- Timer & Round/Pick -->
		<div class="flex items-center gap-4 lg:pr-6">
			<div
				class="text-4xl lg:text-5xl font-bold tabular-nums {timeRemaining < 10 && timeRemaining > 0
					? 'text-red-600 animate-pulse-slow'
					: 'text-gray-900'}"
			>
				{formatTime(timeRemaining)}
			</div>
			<div class="text-sm text-gray-600">
				<div class="font-semibold text-gray-900">
					{#if draftState.currentPick}
						Round {draftState.currentPick.round}
					{:else}
						-
					{/if}
				</div>
				<div>Pick #{draftState.currentPickNumber}</div>
			</div>
		</div>

		<!-- Team on the Clock -->
		<div class="lg:px-6 min-w-0">
			<div class="flex items-center gap-2 mb-1">
				<p class="text-xs font-medium text-gray-500 uppercase tracking-wide">On the Clock</p>
				{#if draftState.hasControlledTeams}
					{#if draftState.isCurrentPickUserControlled}
						<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold bg-blue-600 text-white leading-none">
							YOUR PICK
						</span>
					{:else}
						<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold bg-gray-500 text-white leading-none">
							AI PICK
						</span>
					{/if}
				{/if}
			</div>
			{#if isLoadingTeam}
				<LoadingSpinner size="sm" />
			{:else if team}
				<div class="flex items-center gap-2">
					<span class="text-lg font-bold text-gray-900 truncate">{team.city} {team.name}</span>
					<Badge variant="info" size="sm">{team.abbreviation}</Badge>
				</div>
			{:else}
				<span class="text-gray-400">-</span>
			{/if}
		</div>

		<!-- Status Badge (right-aligned) -->
		<div class="lg:pl-6 lg:ml-auto">
			<Badge variant={statusBadge.variant} size="lg">
				{statusBadge.text}
			</Badge>
		</div>
	</div>

	<!-- Your Teams row (only if controlled teams exist) -->
	{#if draftState.hasControlledTeams && controlledTeams.size > 0}
		<div class="flex items-center gap-2 border-t border-gray-100 pt-3">
			<span class="text-xs font-medium text-gray-500">Your Teams:</span>
			{#each Array.from(controlledTeams.values()) as ct (ct.id)}
				{#if failedLogos.has(ct.id)}
					<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
						{ct.abbreviation}
					</span>
				{:else}
					<img
						src={ct.logo_url || getTeamLogoPath(ct.abbreviation)}
						alt="{ct.city} {ct.name}"
						title="{ct.city} {ct.name}"
						class="w-7 h-7 object-contain"
						onerror={() => {
							failedLogos = new Set(failedLogos).add(ct.id);
						}}
					/>
				{/if}
			{/each}
		</div>
	{/if}

	<!-- Row 2: Session Controls -->
	<div class="flex flex-col lg:flex-row lg:items-center gap-3 lg:gap-6 border-t border-gray-200 pt-4">
		<!-- Start/Pause Button -->
		<div>
			{#if !draftState.session || draftState.session.status === 'NotStarted'}
				<Button
					variant="primary"
					onclick={handleStart}
					disabled={draftState.isLoading}
					loading={draftState.isLoading}
				>
					Start Draft
				</Button>
			{:else if draftState.session.status === 'InProgress'}
				<Button
					variant="secondary"
					onclick={handlePause}
					disabled={draftState.isLoading}
					loading={draftState.isLoading}
				>
					Pause Draft
				</Button>
			{:else if draftState.session.status === 'Paused'}
				<Button
					variant="primary"
					onclick={handleStart}
					disabled={draftState.isLoading}
					loading={draftState.isLoading}
				>
					Resume Draft
				</Button>
			{/if}
		</div>

		<!-- Chart Selector -->
		<div class="flex items-center gap-2">
			<label for="chart-type" class="text-sm font-medium text-gray-600 whitespace-nowrap">
				Trade Value Chart:
			</label>
			<select
				id="chart-type"
				bind:value={selectedChartType}
				class="text-sm rounded border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 py-1.5 px-2"
				disabled={draftState.session?.status === 'InProgress'}
			>
				{#each chartTypes as chartType (chartType)}
					<option value={chartType}>
						{chartType.replace(/([A-Z])/g, ' $1').trim()}
					</option>
				{/each}
			</select>
		</div>

		<!-- Auto-pick Toggle -->
		<div class="flex items-center gap-2">
			<label for="auto-pick" class="text-sm font-medium text-gray-600 whitespace-nowrap">
				Auto-pick:
			</label>
			<input
				id="auto-pick"
				type="checkbox"
				bind:checked={autoPickEnabled}
				class="h-4 w-4 rounded border border-gray-300 text-blue-600 focus:ring-blue-500"
				disabled={draftState.session?.status === 'InProgress'}
			/>
		</div>

		<!-- Time Per Pick -->
		<div class="flex items-center gap-2">
			<label for="time-per-pick" class="text-sm font-medium text-gray-600 whitespace-nowrap">
				Time per Pick:
			</label>
			<input
				id="time-per-pick"
				type="range"
				bind:value={timePerPick}
				min="30"
				max="600"
				step="30"
				class="w-24 lg:w-32"
				disabled={draftState.session?.status === 'InProgress'}
			/>
			<span class="text-sm font-medium text-gray-900 tabular-nums w-12">
				{Math.floor(timePerPick / 60)}:{(timePerPick % 60).toString().padStart(2, '0')}
			</span>
		</div>
	</div>
</div>
