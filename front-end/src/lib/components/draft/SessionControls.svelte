<script lang="ts">
	import { draftState, toastState } from '$stores';
	import { Button, Badge } from '$components/ui';
	import type { ChartType, UUID } from '$types';

	interface Props {
		sessionId: UUID;
	}

	let { sessionId }: Props = $props();

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

	// Initialize values from session
	$effect(() => {
		if (draftState.session) {
			selectedChartType = draftState.session.chart_type;
			autoPickEnabled = draftState.session.auto_pick_enabled;
			timePerPick = draftState.session.time_per_pick_seconds;
		}
	});

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
</script>

<div class="bg-white rounded-lg shadow-md p-6">
	<div class="flex items-center justify-between mb-6">
		<h2 class="text-xl font-semibold text-gray-900">Session Controls</h2>
		<Badge variant={statusBadge.variant} size="lg">
			{statusBadge.text}
		</Badge>
	</div>

	<div class="space-y-6">
		<!-- Start/Pause/Resume Buttons -->
		<div class="flex space-x-3">
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

		<!-- Chart Type Selector -->
		<div>
			<label for="chart-type" class="block text-sm font-medium text-gray-700 mb-2">
				Trade Value Chart
			</label>
			<select
				id="chart-type"
				bind:value={selectedChartType}
				class="block w-full rounded-lg border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
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
		<div class="flex items-center justify-between">
			<label for="auto-pick" class="text-sm font-medium text-gray-700"> Auto-pick Enabled </label>
			<input
				id="auto-pick"
				type="checkbox"
				bind:checked={autoPickEnabled}
				class="h-5 w-5 rounded border border-gray-300 text-blue-600 focus:ring-blue-500"
				disabled={draftState.session?.status === 'InProgress'}
			/>
		</div>

		<!-- Time Per Pick Adjustment -->
		<div>
			<label for="time-per-pick" class="block text-sm font-medium text-gray-700 mb-2">
				Time Per Pick (seconds)
			</label>
			<div class="flex items-center space-x-4">
				<input
					id="time-per-pick"
					type="range"
					bind:value={timePerPick}
					min="30"
					max="600"
					step="30"
					class="flex-1"
					disabled={draftState.session?.status === 'InProgress'}
				/>
				<span class="text-sm font-medium text-gray-900 w-16 text-right">
					{Math.floor(timePerPick / 60)}:{(timePerPick % 60).toString().padStart(2, '0')}
				</span>
			</div>
		</div>
	</div>
</div>
