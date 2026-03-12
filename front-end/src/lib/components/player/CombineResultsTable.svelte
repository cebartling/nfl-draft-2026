<script lang="ts">
	import type { CombineResultsWithPlayer, RasScore } from '$lib/types';
	import { getPositionColor } from '$lib/utils/formatters';
	import {
		getPercentileForValue,
		getPercentileColor,
		type PercentilesMap,
	} from '$lib/utils/combine-percentile';
	import { getScoreColor } from '$lib/utils/ras-format';
	import Badge from '$components/ui/Badge.svelte';

	interface Props {
		results: CombineResultsWithPlayer[];
		percentilesMap: PercentilesMap;
		rasScoresMap: Map<string, RasScore>;
		sortColumn: string;
		sortDirection: 'asc' | 'desc';
		selectedIds: Set<string>;
		onSort: (column: string) => void;
		onToggleSelect: (id: string) => void;
		onSelectPlayer: (result: CombineResultsWithPlayer) => void;
	}

	let {
		results,
		percentilesMap,
		rasScoresMap,
		sortColumn,
		sortDirection,
		selectedIds,
		onSort,
		onToggleSelect,
		onSelectPlayer,
	}: Props = $props();

	function sortArrow(column: string): string {
		if (sortColumn !== column) return '';
		return sortDirection === 'asc' ? ' ↑' : ' ↓';
	}

	function cellColor(
		value: number | null | undefined,
		position: string,
		measurement: string
	): string {
		const pct = getPercentileForValue(value, position, measurement, percentilesMap);
		return getPercentileColor(pct);
	}

	function formatValue(value: number | null | undefined, decimals: number = 2): string {
		if (value == null) return '—';
		return value.toFixed(decimals);
	}

	function formatInt(value: number | null | undefined): string {
		if (value == null) return '—';
		return value.toString();
	}

	function formatRas(score: number | null | undefined): string {
		if (score == null) return 'N/A';
		return score.toFixed(1);
	}
</script>

<div class="overflow-x-auto">
	<table class="min-w-full divide-y divide-gray-200 text-sm">
		<thead class="bg-gray-50">
			<tr>
				<th class="w-10 px-3 py-3">
					<!-- Checkbox column -->
				</th>
				<th
					class="px-3 py-3 text-left font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('player_last_name')}
				>
					Player{sortArrow('player_last_name')}
				</th>
				<th
					class="px-3 py-3 text-left font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('position')}
				>
					Pos{sortArrow('position')}
				</th>
				<th
					class="hidden md:table-cell px-3 py-3 text-left font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('college')}
				>
					College{sortArrow('college')}
				</th>
				<th
					class="px-3 py-3 text-left font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('source')}
				>
					Source{sortArrow('source')}
				</th>
				<th
					class="px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('ras_score')}
					title="Relative Athletic Score (0-10)"
				>
					RAS{sortArrow('ras_score')}
				</th>
				<th
					class="px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('forty_yard_dash')}
				>
					40yd{sortArrow('forty_yard_dash')}
				</th>
				<th
					class="px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('bench_press')}
				>
					Bench{sortArrow('bench_press')}
				</th>
				<th
					class="px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('vertical_jump')}
				>
					Vert{sortArrow('vertical_jump')}
				</th>
				<th
					class="px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('broad_jump')}
				>
					Broad{sortArrow('broad_jump')}
				</th>
				<th
					class="hidden md:table-cell px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('three_cone_drill')}
				>
					3-Cone{sortArrow('three_cone_drill')}
				</th>
				<th
					class="hidden md:table-cell px-3 py-3 text-right font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700"
					onclick={() => onSort('twenty_yard_shuttle')}
				>
					Shuttle{sortArrow('twenty_yard_shuttle')}
				</th>
			</tr>
		</thead>
		<tbody class="bg-white divide-y divide-gray-200">
			{#each results as result (result.id)}
				{@const rasScore = rasScoresMap.get(result.player_id)}
				<tr
					class="hover:bg-gray-50 cursor-pointer transition-colors {selectedIds.has(result.id)
						? 'bg-blue-50'
						: ''}"
					onclick={() => onSelectPlayer(result)}
				>
					<td class="px-3 py-2">
						<input
							type="checkbox"
							checked={selectedIds.has(result.id)}
							aria-label="Select {result.player_first_name} {result.player_last_name} for comparison"
							onclick={(e: MouseEvent) => {
								e.stopPropagation();
								onToggleSelect(result.id);
							}}
							class="h-4 w-4 text-blue-600 rounded border-gray-300"
						/>
					</td>
					<td class="px-3 py-2 font-medium text-gray-900 whitespace-nowrap">
						{result.player_first_name}
						{result.player_last_name}
					</td>
					<td class="px-3 py-2">
						<Badge variant={getPositionColor(result.position)} size="sm">
							{result.position}
						</Badge>
					</td>
					<td class="hidden md:table-cell px-3 py-2 text-gray-600 truncate max-w-[200px]">
						{result.college ?? '—'}
					</td>
					<td class="px-3 py-2">
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {result.source ===
							'combine'
								? 'bg-blue-100 text-blue-700'
								: 'bg-purple-100 text-purple-700'}"
						>
							{result.source === 'combine' ? 'Combine' : 'Pro Day'}
						</span>
					</td>
					<td
						class="px-3 py-2 text-right font-mono font-semibold {getScoreColor(
							rasScore?.overall_score ?? null
						)}"
					>
						{formatRas(rasScore?.overall_score)}
					</td>
					<td
						class="px-3 py-2 text-right font-mono {cellColor(
							result.forty_yard_dash,
							result.position,
							'forty_yard_dash'
						)}"
					>
						{formatValue(result.forty_yard_dash)}
					</td>
					<td
						class="px-3 py-2 text-right font-mono {cellColor(
							result.bench_press,
							result.position,
							'bench_press'
						)}"
					>
						{formatInt(result.bench_press)}
					</td>
					<td
						class="px-3 py-2 text-right font-mono {cellColor(
							result.vertical_jump,
							result.position,
							'vertical_jump'
						)}"
					>
						{formatValue(result.vertical_jump, 1)}
					</td>
					<td
						class="px-3 py-2 text-right font-mono {cellColor(
							result.broad_jump,
							result.position,
							'broad_jump'
						)}"
					>
						{formatInt(result.broad_jump)}
					</td>
					<td
						class="hidden md:table-cell px-3 py-2 text-right font-mono {cellColor(
							result.three_cone_drill,
							result.position,
							'three_cone_drill'
						)}"
					>
						{formatValue(result.three_cone_drill)}
					</td>
					<td
						class="hidden md:table-cell px-3 py-2 text-right font-mono {cellColor(
							result.twenty_yard_shuttle,
							result.position,
							'twenty_yard_shuttle'
						)}"
					>
						{formatValue(result.twenty_yard_shuttle)}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
