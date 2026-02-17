import { describe, it, expect } from 'vitest';
import {
	getScoreColor,
	getBarColor,
	getScoreLabel,
	formatMeasurement,
	formatRawValue,
} from './ras-format';

describe('getScoreColor', () => {
	it('returns gray for null', () => {
		expect(getScoreColor(null)).toBe('text-gray-400');
	});

	it('returns emerald for scores >= 8.0', () => {
		expect(getScoreColor(8.0)).toBe('text-emerald-600');
		expect(getScoreColor(10.0)).toBe('text-emerald-600');
	});

	it('returns blue for scores >= 6.0 and < 8.0', () => {
		expect(getScoreColor(6.0)).toBe('text-blue-600');
		expect(getScoreColor(7.99)).toBe('text-blue-600');
	});

	it('returns amber for scores >= 4.0 and < 6.0', () => {
		expect(getScoreColor(4.0)).toBe('text-amber-600');
		expect(getScoreColor(5.99)).toBe('text-amber-600');
	});

	it('returns red for scores < 4.0', () => {
		expect(getScoreColor(3.99)).toBe('text-red-600');
		expect(getScoreColor(0)).toBe('text-red-600');
	});
});

describe('getBarColor', () => {
	it('returns gray for null', () => {
		expect(getBarColor(null)).toBe('bg-gray-200');
	});

	it('returns emerald for scores >= 8.0', () => {
		expect(getBarColor(8.0)).toBe('bg-emerald-500');
	});

	it('returns blue for scores >= 6.0 and < 8.0', () => {
		expect(getBarColor(6.0)).toBe('bg-blue-500');
	});

	it('returns amber for scores >= 4.0 and < 6.0', () => {
		expect(getBarColor(4.0)).toBe('bg-amber-500');
	});

	it('returns red for scores < 4.0', () => {
		expect(getBarColor(3.99)).toBe('bg-red-500');
	});
});

describe('getScoreLabel', () => {
	it('returns N/A for null', () => {
		expect(getScoreLabel(null)).toBe('N/A');
	});

	it('returns Elite for scores >= 9.0', () => {
		expect(getScoreLabel(9.0)).toBe('Elite');
		expect(getScoreLabel(10.0)).toBe('Elite');
	});

	it('returns Great for scores >= 8.0 and < 9.0', () => {
		expect(getScoreLabel(8.0)).toBe('Great');
		expect(getScoreLabel(8.99)).toBe('Great');
	});

	it('returns Good for scores >= 6.0 and < 8.0', () => {
		expect(getScoreLabel(6.0)).toBe('Good');
		expect(getScoreLabel(7.99)).toBe('Good');
	});

	it('returns Average for scores >= 4.0 and < 6.0', () => {
		expect(getScoreLabel(4.0)).toBe('Average');
		expect(getScoreLabel(5.99)).toBe('Average');
	});

	it('returns Below Avg for scores >= 2.0 and < 4.0', () => {
		expect(getScoreLabel(2.0)).toBe('Below Avg');
		expect(getScoreLabel(3.99)).toBe('Below Avg');
	});

	it('returns Poor for scores < 2.0', () => {
		expect(getScoreLabel(1.99)).toBe('Poor');
		expect(getScoreLabel(0)).toBe('Poor');
	});
});

describe('formatMeasurement', () => {
	it('returns human-readable label for known measurements', () => {
		expect(formatMeasurement('forty_yard_dash')).toBe('40-Yard Dash');
		expect(formatMeasurement('bench_press')).toBe('Bench Press');
		expect(formatMeasurement('vertical_jump')).toBe('Vertical Jump');
		expect(formatMeasurement('broad_jump')).toBe('Broad Jump');
		expect(formatMeasurement('three_cone_drill')).toBe('3-Cone Drill');
		expect(formatMeasurement('twenty_yard_shuttle')).toBe('20-Yard Shuttle');
		expect(formatMeasurement('height')).toBe('Height');
		expect(formatMeasurement('weight')).toBe('Weight');
		expect(formatMeasurement('ten_yard_split')).toBe('10-Yard Split');
		expect(formatMeasurement('twenty_yard_split')).toBe('20-Yard Split');
		expect(formatMeasurement('arm_length')).toBe('Arm Length');
		expect(formatMeasurement('hand_size')).toBe('Hand Size');
		expect(formatMeasurement('wingspan')).toBe('Wingspan');
	});

	it('returns the raw name for unknown measurements', () => {
		expect(formatMeasurement('unknown_measurement')).toBe('unknown_measurement');
	});
});

describe('formatRawValue', () => {
	it('formats timed events with seconds', () => {
		expect(formatRawValue('forty_yard_dash', 4.32)).toBe('4.32s');
		expect(formatRawValue('three_cone_drill', 6.89)).toBe('6.89s');
		expect(formatRawValue('twenty_yard_shuttle', 4.15)).toBe('4.15s');
		expect(formatRawValue('ten_yard_split', 1.52)).toBe('1.52s');
		expect(formatRawValue('twenty_yard_split', 2.61)).toBe('2.61s');
	});

	it('formats bench press with reps', () => {
		expect(formatRawValue('bench_press', 25)).toBe('25 reps');
	});

	it('formats vertical jump with one decimal and inches', () => {
		expect(formatRawValue('vertical_jump', 36.5)).toBe('36.5"');
	});

	it('formats broad jump with inches', () => {
		expect(formatRawValue('broad_jump', 120)).toBe('120"');
	});

	it('formats height as feet and inches', () => {
		expect(formatRawValue('height', 73)).toBe("6'1\"");
		expect(formatRawValue('height', 72)).toBe("6'0\"");
		expect(formatRawValue('height', 76)).toBe("6'4\"");
	});

	it('formats weight with lbs', () => {
		expect(formatRawValue('weight', 215)).toBe('215 lbs');
	});

	it('formats body measurements with two decimals and inches', () => {
		expect(formatRawValue('arm_length', 33.25)).toBe('33.25"');
		expect(formatRawValue('hand_size', 9.75)).toBe('9.75"');
		expect(formatRawValue('wingspan', 78.5)).toBe('78.50"');
	});

	it('returns plain number for unknown measurements', () => {
		expect(formatRawValue('unknown', 42)).toBe('42');
	});
});
