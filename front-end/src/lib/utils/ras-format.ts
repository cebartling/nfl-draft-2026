/**
 * Utility functions for RAS (Relative Athletic Score) display formatting.
 * Extracted from RasScoreCard.svelte for testability.
 */

export function getScoreColor(score: number | null): string {
	if (score === null) return 'text-gray-400';
	if (score >= 8.0) return 'text-emerald-600';
	if (score >= 6.0) return 'text-blue-600';
	if (score >= 4.0) return 'text-amber-600';
	return 'text-red-600';
}

export function getBarColor(score: number | null): string {
	if (score === null) return 'bg-gray-200';
	if (score >= 8.0) return 'bg-emerald-500';
	if (score >= 6.0) return 'bg-blue-500';
	if (score >= 4.0) return 'bg-amber-500';
	return 'bg-red-500';
}

export function getScoreLabel(score: number | null): string {
	if (score === null) return 'N/A';
	if (score >= 9.0) return 'Elite';
	if (score >= 8.0) return 'Great';
	if (score >= 6.0) return 'Good';
	if (score >= 4.0) return 'Average';
	if (score >= 2.0) return 'Below Avg';
	return 'Poor';
}

export function formatMeasurement(name: string): string {
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

export function formatRawValue(measurement: string, value: number): string {
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
