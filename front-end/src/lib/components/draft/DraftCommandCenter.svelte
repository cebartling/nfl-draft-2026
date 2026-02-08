<script lang="ts">
	import { draftState, toastState } from '$stores';
	import { teamsApi } from '$api';
	import { Badge, Button, LoadingSpinner } from '$components/ui';
	import type { ChartType, Team, UUID } from '$types';
	import { logger } from '$lib/utils/logger';

	interface Props {
		sessionId: UUID;
	}

	let { sessionId }: Props = $props();

	// --- Clock state ---
	let team = $state<Team | null>(null);
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
	$effect(() => {
		const session = draftState.session;
		if (!session || session.status !== 'InProgress') {
			timeRemaining = 0;
			return;
		}

		timeRemaining = session.time_per_pick_seconds;

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
				})
				.finally(() => {
					isLoadingTeam = false;
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
		} catch (err) {
			toastState.error('Failed to start session');
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
			<p class="text-xs font-medium text-gray-500 uppercase tracking-wide mb-1">On the Clock</p>
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
