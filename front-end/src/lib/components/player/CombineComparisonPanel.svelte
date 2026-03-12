<script lang="ts">
	import type { CombineResultsWithPlayer, RasScore } from '$lib/types';
	import {
		getPercentileForValue,
		getPercentileColor,
		LOWER_IS_BETTER_MEASUREMENTS,
		type PercentilesMap,
	} from '$lib/utils/combine-percentile';
	import { getScoreColor } from '$lib/utils/ras-format';
	import { getPositionColor } from '$lib/utils/formatters';
	import Badge from '$components/ui/Badge.svelte';

	interface Props {
		players: CombineResultsWithPlayer[];
		percentilesMap: PercentilesMap;
		rasScoresMap: Map<string, RasScore>;
		onClose: () => void;
	}

	let { players, percentilesMap, rasScoresMap, onClose }: Props = $props();

	const measurements = [
		{ key: 'forty_yard_dash', label: '40-Yard Dash', unit: 's', decimals: 2 },
		{ key: 'ten_yard_split', label: '10-Yard Split', unit: 's', decimals: 2 },
		{ key: 'twenty_yard_split', label: '20-Yard Split', unit: 's', decimals: 2 },
		{ key: 'bench_press', label: 'Bench Press', unit: ' reps', decimals: 0 },
		{ key: 'vertical_jump', label: 'Vertical Jump', unit: '"', decimals: 1 },
		{ key: 'broad_jump', label: 'Broad Jump', unit: '"', decimals: 0 },
		{ key: 'three_cone_drill', label: '3-Cone Drill', unit: 's', decimals: 2 },
		{ key: 'twenty_yard_shuttle', label: '20-Yard Shuttle', unit: 's', decimals: 2 },
		{ key: 'arm_length', label: 'Arm Length', unit: '"', decimals: 2 },
		{ key: 'hand_size', label: 'Hand Size', unit: '"', decimals: 2 },
		{ key: 'wingspan', label: 'Wingspan', unit: '"', decimals: 2 },
	];

	function getValue(player: CombineResultsWithPlayer, key: string): number | null | undefined {
		return (player as Record<string, unknown>)[key] as number | null | undefined;
	}

	function formatVal(value: number | null | undefined, decimals: number): string {
		if (value == null) return '—';
		return decimals === 0 ? value.toString() : value.toFixed(decimals);
	}

	function isBest(
		value: number | null | undefined,
		key: string,
		allValues: (number | null | undefined)[]
	): boolean {
		if (value == null) return false;
		const valid = allValues.filter((v): v is number => v != null);
		if (valid.length < 2) return false;

		if (LOWER_IS_BETTER_MEASUREMENTS.includes(key)) {
			return value === Math.min(...valid);
		}
		return value === Math.max(...valid);
	}

	function getRasScores(): (number | null)[] {
		return players.map((p) => rasScoresMap.get(p.player_id)?.overall_score ?? null);
	}

	function isBestRas(
		score: number | null | undefined,
		allScores: (number | null | undefined)[]
	): boolean {
		if (score == null) return false;
		const valid = allScores.filter((v): v is number => v != null);
		if (valid.length < 2) return false;
		return score === Math.max(...valid);
	}
</script>

<div class="bg-white rounded-lg shadow-lg border border-gray-200 p-4">
	<div class="flex items-center justify-between mb-4">
		<h3 class="text-lg font-semibold text-gray-800">Player Comparison</h3>
		<button
			type="button"
			onclick={onClose}
			aria-label="Close comparison panel"
			class="text-gray-400 hover:text-gray-600 text-xl leading-none"
		>
			×
		</button>
	</div>

	<div class="overflow-x-auto">
		<table class="min-w-full text-sm" aria-label="Player comparison">
			<thead>
				<tr class="border-b border-gray-200">
					<th scope="col" class="text-left py-2 px-3 font-medium text-gray-500"
						>Measurement</th
					>
					{#each players as player (player.id)}
						<th scope="col" class="text-center py-2 px-3 min-w-[120px]">
							<div class="font-semibold text-gray-900">
								{player.player_first_name}
								{player.player_last_name}
							</div>
							<Badge variant={getPositionColor(player.position)} size="sm">
								{player.position}
							</Badge>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody class="divide-y divide-gray-100">
				<!-- RAS Overall Score row -->
				<tr class="hover:bg-gray-50 bg-gray-50/50">
					<td class="py-2 px-3 font-semibold text-gray-700">RAS Score</td>
					{#each players as player, i (player.id)}
						{@const rasOverall = rasScoresMap.get(player.player_id)?.overall_score ?? null}
						{@const best = isBestRas(rasOverall, getRasScores())}
						<td class="py-2 px-3 text-center">
							<span
								class="inline-flex items-center gap-1 px-2 py-0.5 rounded font-mono font-semibold {getScoreColor(
									rasOverall
								)} {best ? 'ring-2 ring-blue-400' : ''}"
							>
								{rasOverall != null ? rasOverall.toFixed(1) : 'N/A'}
								{#if best}
									<span class="text-blue-500">★</span>
								{/if}
							</span>
						</td>
					{/each}
				</tr>
				{#each measurements as m (m.key)}
					{@const allValues = players.map((p) => getValue(p, m.key))}
					<tr class="hover:bg-gray-50">
						<td class="py-2 px-3 font-medium text-gray-600">{m.label}</td>
						{#each players as player, i (player.id)}
							{@const value = allValues[i]}
							{@const pct = getPercentileForValue(value, player.position, m.key, percentilesMap)}
							{@const best = isBest(value, m.key, allValues)}
							<td class="py-2 px-3 text-center">
								<span
									class="inline-flex items-center gap-1 px-2 py-0.5 rounded font-mono {getPercentileColor(
										pct
									)} {best ? 'ring-2 ring-blue-400 font-bold' : ''}"
								>
									{formatVal(value, m.decimals)}{value != null ? m.unit : ''}
									{#if best}
										<span class="text-blue-500">★</span>
									{/if}
								</span>
								{#if pct != null}
									<div class="text-xs text-gray-400 mt-0.5">P{pct}</div>
								{/if}
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
