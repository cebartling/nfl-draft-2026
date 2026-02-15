/**
 * Date, string, and player formatting utilities
 */

/**
 * Get Badge variant color for a player position.
 */
export function getPositionColor(position: string): 'primary' | 'danger' | 'info' {
	const offensePositions = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
	const defensePositions = ['DE', 'DT', 'LB', 'CB', 'S'];

	if (offensePositions.includes(position)) return 'primary';
	if (defensePositions.includes(position)) return 'danger';
	return 'info';
}

/**
 * Format height in inches to feet/inches display string.
 */
export function formatHeight(inches?: number | null): string {
	if (!inches) return 'N/A';
	const feet = Math.floor(inches / 12);
	const remainingInches = inches % 12;
	return `${feet}'${remainingInches}"`;
}

/**
 * Format a date string to locale date format
 * @param date - ISO date string or undefined
 * @returns Formatted date string or empty string if undefined
 */
export function formatDate(date: string | undefined | null): string {
	if (!date) return '';
	try {
		return new Date(date).toLocaleDateString();
	} catch {
		return '';
	}
}

/**
 * Format a date string to locale date and time format
 * @param date - ISO date string or undefined
 * @returns Formatted datetime string or empty string if undefined
 */
export function formatDateTime(date: string | undefined | null): string {
	if (!date) return '';
	try {
		return new Date(date).toLocaleString();
	} catch {
		return '';
	}
}

/**
 * Format a date string to relative time (e.g., "2 hours ago")
 * @param date - ISO date string or undefined
 * @returns Relative time string or empty string if undefined
 */
export function formatRelativeTime(date: string | undefined | null): string {
	if (!date) return '';
	try {
		const now = new Date();
		const then = new Date(date);
		const diffMs = now.getTime() - then.getTime();
		const diffSecs = Math.floor(diffMs / 1000);
		const diffMins = Math.floor(diffSecs / 60);
		const diffHours = Math.floor(diffMins / 60);
		const diffDays = Math.floor(diffHours / 24);

		if (diffSecs < 60) return 'just now';
		if (diffMins < 60) return `${diffMins} minute${diffMins === 1 ? '' : 's'} ago`;
		if (diffHours < 24) return `${diffHours} hour${diffHours === 1 ? '' : 's'} ago`;
		if (diffDays < 7) return `${diffDays} day${diffDays === 1 ? '' : 's'} ago`;
		return formatDate(date);
	} catch {
		return '';
	}
}
