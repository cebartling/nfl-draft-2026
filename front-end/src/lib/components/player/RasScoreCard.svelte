<script lang="ts">
	import type { RasScore } from '$types';

	interface Props {
		rasScore: RasScore;
	}

	let { rasScore }: Props = $props();

	function getScoreColor(score: number | null): string {
		if (score === null) return 'text-gray-400';
		if (score >= 8.0) return 'text-emerald-600';
		if (score >= 6.0) return 'text-blue-600';
		if (score >= 4.0) return 'text-amber-600';
		return 'text-red-600';
	}

	function getBarColor(score: number | null): string {
		if (score === null) return 'bg-gray-200';
		if (score >= 8.0) return 'bg-emerald-500';
		if (score >= 6.0) return 'bg-blue-500';
		if (score >= 4.0) return 'bg-amber-500';
		return 'bg-red-500';
	}

	function getScoreLabel(score: number | null): string {
		if (score === null) return 'N/A';
		if (score >= 9.0) return 'Elite';
		if (score >= 8.0) return 'Great';
		if (score >= 6.0) return 'Good';
		if (score >= 4.0) return 'Average';
		if (score >= 2.0) return 'Below Avg';
		return 'Poor';
	}

	function formatMeasurement(name: string): string {
		const labels: Record<string, string> = {
			forty_yard_dash: '40-Yard Dash',
			bench_press: 'Bench Press',
			vertical_jump: 'Vertical Jump',
			broad_jump: 'Broad Jump',
			three_cone_drill: '3-Cone Drill',
			twenty_yard_shuttle: '20-Yard Shuttle',
			height: 'Height',
			weight: 'Weight',
			ten_yard_split: '10-Yard Split',
			twenty_yard_split: '20-Yard Split',
			arm_length: 'Arm Length',
			hand_size: 'Hand Size',
			wingspan: 'Wingspan',
		};
		return labels[name] ?? name;
	}

	function formatRawValue(measurement: string, value: number): string {
		const timedEvents = [
			'forty_yard_dash',
			'three_cone_drill',
			'twenty_yard_shuttle',
			'ten_yard_split',
			'twenty_yard_split',
		];
		if (timedEvents.includes(measurement)) return `${value.toFixed(2)}s`;
		if (measurement === 'bench_press') return `${value} reps`;
		if (measurement === 'vertical_jump') return `${value.toFixed(1)}"`;
		if (measurement === 'broad_jump') return `${value}"`;
		if (measurement === 'height') {
			const feet = Math.floor(value / 12);
			const inches = value % 12;
			return `${feet}'${inches}"`;
		}
		if (measurement === 'weight') return `${value} lbs`;
		if (['arm_length', 'hand_size', 'wingspan'].includes(measurement))
			return `${value.toFixed(2)}"`;
		return `${value}`;
	}

	const categories = $derived(
		[
			{ label: 'Size', score: rasScore.size_score },
			{ label: 'Speed', score: rasScore.speed_score },
			{ label: 'Strength', score: rasScore.strength_score },
			{ label: 'Explosion', score: rasScore.explosion_score },
			{ label: 'Agility', score: rasScore.agility_score },
		].filter((c) => c.score !== null),
	);
</script>

<div class="space-y-6">
	<!-- Overall Score -->
	{#if rasScore.overall_score !== null}
		<div class="text-center">
			<p class="text-sm font-medium text-gray-500 mb-1">Overall RAS</p>
			<p class="text-5xl font-bold {getScoreColor(rasScore.overall_score)}">
				{rasScore.overall_score.toFixed(2)}
			</p>
			<p class="text-sm {getScoreColor(rasScore.overall_score)} mt-1">
				{getScoreLabel(rasScore.overall_score)}
			</p>
			<p class="text-xs text-gray-400 mt-1">
				{rasScore.measurements_used} of {rasScore.measurements_total} measurements
			</p>
		</div>
	{:else}
		<div class="text-center py-4">
			<p class="text-sm font-medium text-gray-500 mb-1">Overall RAS</p>
			<p class="text-2xl font-bold text-gray-300">N/A</p>
			{#if rasScore.explanation}
				<p class="text-xs text-gray-400 mt-2">{rasScore.explanation}</p>
			{/if}
		</div>
	{/if}

	<!-- Category Scores -->
	{#if categories.length > 0}
		<div class="grid grid-cols-5 gap-2">
			{#each categories as cat (cat.label)}
				<div class="text-center">
					<p class="text-xs font-medium text-gray-500">{cat.label}</p>
					<p class="text-lg font-bold {getScoreColor(cat.score)}">
						{cat.score !== null ? cat.score.toFixed(1) : 'N/A'}
					</p>
					<div class="w-full bg-gray-100 rounded-full h-1.5 mt-1">
						<div
							class="h-1.5 rounded-full {getBarColor(cat.score)}"
							style="width: {cat.score !== null ? cat.score * 10 : 0}%"
						></div>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Individual Measurement Scores -->
	{#if rasScore.individual_scores.length > 0}
		<div>
			<h4 class="text-sm font-medium text-gray-700 mb-3">Individual Measurements</h4>
			<div class="space-y-2">
				{#each rasScore.individual_scores as ms (ms.measurement)}
					<div class="flex items-center gap-3">
						<span class="text-xs text-gray-600 w-28 shrink-0"
							>{formatMeasurement(ms.measurement)}</span
						>
						<div class="flex-1 bg-gray-100 rounded-full h-2">
							<div
								class="h-2 rounded-full {getBarColor(ms.score)}"
								style="width: {ms.score * 10}%"
							></div>
						</div>
						<span class="text-xs font-medium {getScoreColor(ms.score)} w-8 text-right"
							>{ms.score.toFixed(1)}</span
						>
						<span class="text-xs text-gray-400 w-16 text-right"
							>{formatRawValue(ms.measurement, ms.raw_value)}</span
						>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
