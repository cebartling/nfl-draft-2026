import { describe, it, expect } from 'vitest';
import {
	buildPercentilesMap,
	getPercentileForValue,
	getPercentileColor,
	LOWER_IS_BETTER_MEASUREMENTS,
} from './combine-percentile';
import type { CombinePercentile } from '$lib/types';

function makePercentile(
	position: string,
	measurement: string,
	overrides: Partial<CombinePercentile> = {}
): CombinePercentile {
	return {
		id: '00000000-0000-0000-0000-000000000001',
		position,
		measurement,
		sample_size: 100,
		min_value: 4.3,
		p10: 4.4,
		p20: 4.5,
		p30: 4.55,
		p40: 4.6,
		p50: 4.65,
		p60: 4.7,
		p70: 4.75,
		p80: 4.8,
		p90: 4.9,
		max_value: 5.0,
		years_start: 2020,
		years_end: 2025,
		...overrides,
	};
}

describe('buildPercentilesMap', () => {
	it('builds a nested map from position to measurement to percentile', () => {
		const percentiles = [
			makePercentile('QB', 'forty_yard_dash'),
			makePercentile('QB', 'bench_press', {
				min_value: 10,
				p10: 12,
				p20: 14,
				p30: 16,
				p40: 18,
				p50: 20,
				p60: 22,
				p70: 24,
				p80: 26,
				p90: 28,
				max_value: 30,
			}),
			makePercentile('WR', 'forty_yard_dash'),
		];

		const map = buildPercentilesMap(percentiles);

		expect(map.has('QB')).toBe(true);
		expect(map.has('WR')).toBe(true);
		expect(map.get('QB')!.has('forty_yard_dash')).toBe(true);
		expect(map.get('QB')!.has('bench_press')).toBe(true);
		expect(map.get('WR')!.has('forty_yard_dash')).toBe(true);
	});

	it('returns an empty map for empty input', () => {
		const map = buildPercentilesMap([]);
		expect(map.size).toBe(0);
	});
});

describe('getPercentileForValue', () => {
	const percentiles = [makePercentile('QB', 'forty_yard_dash')];
	const map = buildPercentilesMap(percentiles);

	it('returns null for null/undefined value', () => {
		expect(getPercentileForValue(null, 'QB', 'forty_yard_dash', map)).toBeNull();
		expect(getPercentileForValue(undefined, 'QB', 'forty_yard_dash', map)).toBeNull();
	});

	it('returns null for missing position', () => {
		expect(getPercentileForValue(4.5, 'TE', 'forty_yard_dash', map)).toBeNull();
	});

	it('returns null for missing measurement', () => {
		expect(getPercentileForValue(4.5, 'QB', 'bench_press', map)).toBeNull();
	});

	it('inverts for lower-is-better measurements (40yd)', () => {
		// For 40yd, a value of 4.3 (min/fastest) should be ~100th percentile
		// A value of 5.0 (max/slowest) should be ~0th percentile
		const fast = getPercentileForValue(4.3, 'QB', 'forty_yard_dash', map)!;
		const slow = getPercentileForValue(5.0, 'QB', 'forty_yard_dash', map)!;
		expect(fast).toBeGreaterThan(slow);
		expect(fast).toBeGreaterThanOrEqual(90);
		expect(slow).toBeLessThanOrEqual(10);
	});

	it('does not invert for higher-is-better measurements', () => {
		const benchPercentiles = [
			makePercentile('QB', 'bench_press', {
				min_value: 10,
				p10: 12,
				p20: 14,
				p30: 16,
				p40: 18,
				p50: 20,
				p60: 22,
				p70: 24,
				p80: 26,
				p90: 28,
				max_value: 30,
			}),
		];
		const benchMap = buildPercentilesMap(benchPercentiles);

		const high = getPercentileForValue(30, 'QB', 'bench_press', benchMap)!;
		const low = getPercentileForValue(10, 'QB', 'bench_press', benchMap)!;
		expect(high).toBeGreaterThan(low);
	});

	it('returns a value between 0 and 100', () => {
		const result = getPercentileForValue(4.65, 'QB', 'forty_yard_dash', map)!;
		expect(result).toBeGreaterThanOrEqual(0);
		expect(result).toBeLessThanOrEqual(100);
	});

	it('handles equal consecutive breakpoints without division by zero', () => {
		const flatPercentiles = [
			makePercentile('QB', 'bench_press', {
				min_value: 20,
				p10: 20,
				p20: 20,
				p30: 20,
				p40: 20,
				p50: 20,
				p60: 20,
				p70: 20,
				p80: 20,
				p90: 20,
				max_value: 20,
			}),
		];
		const flatMap = buildPercentilesMap(flatPercentiles);

		// Should not throw or return NaN
		const result = getPercentileForValue(20, 'QB', 'bench_press', flatMap);
		expect(result).not.toBeNaN();
		expect(result).toBeTypeOf('number');
	});
});

describe('getPercentileColor', () => {
	it('returns green class for elite percentile (>=80)', () => {
		const result = getPercentileColor(90);
		expect(result).toContain('green');
	});

	it('returns green class for good percentile (>=60)', () => {
		const result = getPercentileColor(70);
		expect(result).toContain('green');
	});

	it('returns yellow class for average percentile (>=40)', () => {
		const result = getPercentileColor(50);
		expect(result).toContain('yellow');
	});

	it('returns orange class for below average percentile (>=20)', () => {
		const result = getPercentileColor(30);
		expect(result).toContain('orange');
	});

	it('returns red class for poor percentile (<20)', () => {
		const result = getPercentileColor(10);
		expect(result).toContain('red');
	});

	it('returns empty string for null', () => {
		expect(getPercentileColor(null)).toBe('');
	});
});

describe('LOWER_IS_BETTER_MEASUREMENTS', () => {
	it('includes time-based measurements', () => {
		expect(LOWER_IS_BETTER_MEASUREMENTS).toContain('forty_yard_dash');
		expect(LOWER_IS_BETTER_MEASUREMENTS).toContain('three_cone_drill');
		expect(LOWER_IS_BETTER_MEASUREMENTS).toContain('twenty_yard_shuttle');
		expect(LOWER_IS_BETTER_MEASUREMENTS).toContain('ten_yard_split');
		expect(LOWER_IS_BETTER_MEASUREMENTS).toContain('twenty_yard_split');
	});

	it('does not include strength/explosion measurements', () => {
		expect(LOWER_IS_BETTER_MEASUREMENTS).not.toContain('bench_press');
		expect(LOWER_IS_BETTER_MEASUREMENTS).not.toContain('vertical_jump');
		expect(LOWER_IS_BETTER_MEASUREMENTS).not.toContain('broad_jump');
	});
});
