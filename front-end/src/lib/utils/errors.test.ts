import { describe, it, expect } from 'vitest';
import { parseErrorMessage, extractErrorDetails } from './errors';

describe('parseErrorMessage', () => {
	// Known error pattern tests
	it('should match "Draft for year already exists" pattern', () => {
		const result = parseErrorMessage(new Error('Draft for year 2026 already exists'));
		expect(result).toBe('A draft for this year already exists. Please choose a different year.');
	});

	it('should match "Team not found" pattern', () => {
		const result = parseErrorMessage('Team abc123 not found');
		expect(result).toBe('The requested team could not be found.');
	});

	it('should match "Player not found" pattern', () => {
		const result = parseErrorMessage(new Error('Player xyz not found'));
		expect(result).toBe('The requested player could not be found.');
	});

	it('should match "Draft not found" pattern', () => {
		const result = parseErrorMessage('Draft def456 not found');
		expect(result).toBe('The requested draft could not be found.');
	});

	it('should match "Session not found" pattern', () => {
		const result = parseErrorMessage(new Error('Session not found'));
		expect(result).toBe('The draft session could not be found.');
	});

	it('should match "Pick not found" pattern', () => {
		const result = parseErrorMessage('Pick 123 not found');
		expect(result).toBe('The requested pick could not be found.');
	});

	it('should match "already drafted" pattern', () => {
		const result = parseErrorMessage(new Error('Player already drafted'));
		expect(result).toBe('This player has already been drafted.');
	});

	it('should match "not turn" pattern', () => {
		const result = parseErrorMessage("It's not your turn to make a pick");
		expect(result).toBe("It's not your turn to pick.");
	});

	it('should match "session not started" pattern', () => {
		const result = parseErrorMessage(new Error('Session not started yet'));
		expect(result).toBe('The draft session has not started yet.');
	});

	it('should match "session completed" pattern', () => {
		const result = parseErrorMessage('Session completed');
		expect(result).toBe('The draft session has already completed.');
	});

	it('should match "validation failed" pattern', () => {
		const result = parseErrorMessage(new Error('Data validation failed'));
		expect(result).toBe('The data provided is invalid. Please check your input.');
	});

	it('should match "network" pattern', () => {
		const result = parseErrorMessage(new Error('network error'));
		expect(result).toBe(
			'Unable to connect to the server. Please check your internet connection.'
		);
	});

	it('should match "fetch" pattern', () => {
		const result = parseErrorMessage('Failed to fetch');
		expect(result).toBe(
			'Unable to connect to the server. Please check your internet connection.'
		);
	});

	it('should match "connection" pattern', () => {
		const result = parseErrorMessage(new Error('connection refused'));
		expect(result).toBe(
			'Unable to connect to the server. Please check your internet connection.'
		);
	});

	it('should match "timeout" pattern', () => {
		const result = parseErrorMessage('Request timeout');
		expect(result).toBe('The request timed out. Please try again.');
	});

	it('should match "unauthorized" pattern', () => {
		const result = parseErrorMessage(new Error('unauthorized access'));
		expect(result).toBe('You are not authorized to perform this action.');
	});

	it('should match "401" pattern', () => {
		const result = parseErrorMessage('Error 401');
		expect(result).toBe('You are not authorized to perform this action.');
	});

	it('should match "forbidden" pattern', () => {
		const result = parseErrorMessage(new Error('forbidden'));
		expect(result).toBe('You do not have permission to access this resource.');
	});

	it('should match "403" pattern', () => {
		const result = parseErrorMessage('Error 403');
		expect(result).toBe('You do not have permission to access this resource.');
	});

	it('should match "internal server error" pattern', () => {
		const result = parseErrorMessage(new Error('Internal server error'));
		expect(result).toBe('An unexpected server error occurred. Please try again later.');
	});

	it('should match "500" pattern', () => {
		const result = parseErrorMessage('Error 500');
		expect(result).toBe('An unexpected server error occurred. Please try again later.');
	});

	// Special cases
	it('should handle "Response validation failed" special case', () => {
		// Note: "Response validation failed" also matches the "validation failed" pattern,
		// which is checked first. Verify the more specific path isn't reachable unless patterns change.
		// The "Response validation failed" check is a fallback after pattern matching.
		// Test the fallback with a message that only matches the special case:
		const result = parseErrorMessage(
			new Error('Response validation failed for /api/sessions')
		);
		// This matches the "validation failed" pattern first
		expect(result).toBe('The data provided is invalid. Please check your input.');
	});

	it('should capitalize and add period to unknown errors', () => {
		const result = parseErrorMessage('some unknown error happened');
		expect(result).toBe('Some unknown error happened.');
	});

	it('should not add period if already ends with period', () => {
		const result = parseErrorMessage('Already ends with period.');
		expect(result).toBe('Already ends with period.');
	});

	it('should not add period if ends with exclamation', () => {
		const result = parseErrorMessage('Already ends with exclamation!');
		expect(result).toBe('Already ends with exclamation!');
	});

	it('should not add period if ends with question mark', () => {
		const result = parseErrorMessage('Already ends with question?');
		expect(result).toBe('Already ends with question?');
	});

	it('should handle Error objects', () => {
		const result = parseErrorMessage(new Error('some error'));
		expect(result).toBe('Some error.');
	});

	it('should handle string errors', () => {
		const result = parseErrorMessage('string error');
		expect(result).toBe('String error.');
	});

	it('should handle unknown types', () => {
		const result = parseErrorMessage(42);
		expect(result).toBe('An unexpected error occurred.');
	});

	it('should handle null', () => {
		const result = parseErrorMessage(null);
		expect(result).toBe('An unexpected error occurred.');
	});

	it('should handle undefined', () => {
		const result = parseErrorMessage(undefined);
		expect(result).toBe('An unexpected error occurred.');
	});
});

describe('extractErrorDetails', () => {
	it('should extract message and stack from Error', () => {
		const error = new Error('test error');
		const details = extractErrorDetails(error);

		expect(details.message).toBe('test error');
		expect(details.stack).toBeDefined();
		expect(details.raw).toBe(error);
	});

	it('should convert non-Error to string', () => {
		const details = extractErrorDetails('string error');

		expect(details.message).toBe('string error');
		expect(details.stack).toBeUndefined();
		expect(details.raw).toBe('string error');
	});

	it('should convert number to string', () => {
		const details = extractErrorDetails(42);

		expect(details.message).toBe('42');
		expect(details.stack).toBeUndefined();
		expect(details.raw).toBe(42);
	});

	it('should convert null to string', () => {
		const details = extractErrorDetails(null);

		expect(details.message).toBe('null');
		expect(details.raw).toBeNull();
	});
});
