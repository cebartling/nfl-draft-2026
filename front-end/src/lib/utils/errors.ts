/**
 * Error message parsing and formatting utilities
 */

/**
 * Known API error patterns and their user-friendly messages
 */
const ERROR_PATTERNS: Array<{ pattern: RegExp; message: string }> = [
	{
		pattern: /Draft for year (\d+) already exists/i,
		message: 'A draft for this year already exists. Please choose a different year.',
	},
	{
		pattern: /Team.*not found/i,
		message: 'The requested team could not be found.',
	},
	{
		pattern: /Player.*not found/i,
		message: 'The requested player could not be found.',
	},
	{
		pattern: /Draft.*not found/i,
		message: 'The requested draft could not be found.',
	},
	{
		pattern: /Session.*not found/i,
		message: 'The draft session could not be found.',
	},
	{
		pattern: /Pick.*not found/i,
		message: 'The requested pick could not be found.',
	},
	{
		pattern: /already.*drafted/i,
		message: 'This player has already been drafted.',
	},
	{
		pattern: /not.*turn/i,
		message: "It's not your turn to pick.",
	},
	{
		pattern: /session.*not.*started/i,
		message: 'The draft session has not started yet.',
	},
	{
		pattern: /session.*completed/i,
		message: 'The draft session has already completed.',
	},
	{
		pattern: /validation failed/i,
		message: 'The data provided is invalid. Please check your input.',
	},
	{
		pattern: /network|fetch|connection/i,
		message: 'Unable to connect to the server. Please check your internet connection.',
	},
	{
		pattern: /timeout/i,
		message: 'The request timed out. Please try again.',
	},
	{
		pattern: /unauthorized|401/i,
		message: 'You are not authorized to perform this action.',
	},
	{
		pattern: /forbidden|403/i,
		message: 'You do not have permission to access this resource.',
	},
	{
		pattern: /internal server error|500/i,
		message: 'An unexpected server error occurred. Please try again later.',
	},
];

/**
 * Parse an error into a user-friendly message
 * @param error - Error object or string
 * @returns User-friendly error message
 */
export function parseErrorMessage(error: unknown): string {
	// Get the raw error message
	let rawMessage: string;
	if (error instanceof Error) {
		rawMessage = error.message;
	} else if (typeof error === 'string') {
		rawMessage = error;
	} else {
		rawMessage = 'An unexpected error occurred';
	}

	// Check for known patterns
	for (const { pattern, message } of ERROR_PATTERNS) {
		if (pattern.test(rawMessage)) {
			return message;
		}
	}

	// If no pattern matches, return a cleaned-up version of the original
	// Remove technical details like JSON parsing errors
	if (rawMessage.includes('Response validation failed')) {
		return 'The server returned unexpected data. Please try again.';
	}

	// Capitalize first letter and ensure it ends with a period
	let cleaned = rawMessage.trim();
	if (cleaned.length > 0) {
		cleaned = cleaned.charAt(0).toUpperCase() + cleaned.slice(1);
		if (!cleaned.endsWith('.') && !cleaned.endsWith('!') && !cleaned.endsWith('?')) {
			cleaned += '.';
		}
	}

	return cleaned || 'An unexpected error occurred.';
}

/**
 * Extract error details for logging (keeps technical details)
 * @param error - Error object or string
 * @returns Error details object
 */
export function extractErrorDetails(error: unknown): {
	message: string;
	stack?: string;
	raw: unknown;
} {
	if (error instanceof Error) {
		return {
			message: error.message,
			stack: error.stack,
			raw: error,
		};
	}
	return {
		message: String(error),
		raw: error,
	};
}
