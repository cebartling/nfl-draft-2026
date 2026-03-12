import type { CombinePercentile } from '$lib/types';

/** Measurements where lower values are better (time-based) */
export const LOWER_IS_BETTER_MEASUREMENTS = [
	'forty_yard_dash',
	'three_cone_drill',
	'twenty_yard_shuttle',
	'ten_yard_split',
	'twenty_yard_split',
];

export type PercentilesMap = Map<string, Map<string, CombinePercentile>>;

/**
 * Build a nested map: position -> measurement -> CombinePercentile
 */
export function buildPercentilesMap(percentiles: CombinePercentile[]): PercentilesMap {
	const map: PercentilesMap = new Map();

	for (const p of percentiles) {
		if (!map.has(p.position)) {
			map.set(p.position, new Map());
		}
		map.get(p.position)!.set(p.measurement, p);
	}

	return map;
}

/**
 * Get the percentile (0-100) for a given value, position, and measurement.
 * Returns null if value is null/undefined or if no percentile data exists.
 * Inverts for "lower is better" metrics.
 */
export function getPercentileForValue(
	value: number | null | undefined,
	position: string,
	measurement: string,
	map: PercentilesMap
): number | null {
	if (value == null) return null;

	const posMap = map.get(position);
	if (!posMap) return null;

	const percentile = posMap.get(measurement);
	if (!percentile) return null;

	const breakpoints = [
		{ pct: 0, val: percentile.min_value },
		{ pct: 10, val: percentile.p10 },
		{ pct: 20, val: percentile.p20 },
		{ pct: 30, val: percentile.p30 },
		{ pct: 40, val: percentile.p40 },
		{ pct: 50, val: percentile.p50 },
		{ pct: 60, val: percentile.p60 },
		{ pct: 70, val: percentile.p70 },
		{ pct: 80, val: percentile.p80 },
		{ pct: 90, val: percentile.p90 },
		{ pct: 100, val: percentile.max_value },
	];

	let rawPercentile: number;

	if (value <= breakpoints[0].val) {
		rawPercentile = 0;
	} else if (value >= breakpoints[breakpoints.length - 1].val) {
		rawPercentile = 100;
	} else {
		// Interpolate between breakpoints
		rawPercentile = 50; // fallback
		for (let i = 0; i < breakpoints.length - 1; i++) {
			const low = breakpoints[i];
			const high = breakpoints[i + 1];
			if (value >= low.val && value <= high.val) {
				const range = high.val - low.val;
				const ratio = range === 0 ? 0.5 : (value - low.val) / range;
				rawPercentile = low.pct + ratio * (high.pct - low.pct);
				break;
			}
		}
	}

	// Invert for lower-is-better measurements
	if (LOWER_IS_BETTER_MEASUREMENTS.includes(measurement)) {
		rawPercentile = 100 - rawPercentile;
	}

	return Math.round(rawPercentile);
}

/**
 * Get Tailwind color classes for a percentile value.
 */
export function getPercentileColor(percentile: number | null): string {
	if (percentile == null) return '';

	if (percentile >= 80) return 'bg-green-100 text-green-800';
	if (percentile >= 60) return 'bg-green-50 text-green-700';
	if (percentile >= 40) return 'bg-yellow-50 text-yellow-700';
	if (percentile >= 20) return 'bg-orange-50 text-orange-700';
	return 'bg-red-50 text-red-700';
}
