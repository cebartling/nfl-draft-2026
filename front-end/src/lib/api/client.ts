import { z } from 'zod';
import { ApiErrorSchema, type ApiError } from '$lib/types';

/**
 * Custom error class for API errors
 */
export class ApiClientError extends Error {
	constructor(
		message: string,
		public status: number,
		public apiError?: ApiError
	) {
		super(message);
		this.name = 'ApiClientError';
	}
}

/**
 * Base API client with Zod validation and error handling
 */
export class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = '/api') {
		this.baseUrl = baseUrl;
	}

	/**
	 * Make a GET request with schema validation
	 */
	async get<T>(path: string, schema: z.ZodType<T>): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'GET',
			headers: {
				'Content-Type': 'application/json',
			},
		});

		return this.handleResponse(response, schema);
	}

	/**
	 * Make a POST request with schema validation
	 */
	async post<T>(path: string, data: unknown, schema: z.ZodType<T>): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(data),
		});

		return this.handleResponse(response, schema);
	}

	/**
	 * Make a PUT request with schema validation
	 */
	async put<T>(path: string, data: unknown, schema: z.ZodType<T>): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(data),
		});

		return this.handleResponse(response, schema);
	}

	/**
	 * Make a DELETE request with schema validation
	 */
	async delete<T>(path: string, schema: z.ZodType<T>): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json',
			},
		});

		return this.handleResponse(response, schema);
	}

	/**
	 * Handle API response with error checking and Zod validation
	 */
	private async handleResponse<T>(response: Response, schema: z.ZodType<T>): Promise<T> {
		// Handle non-OK responses
		if (!response.ok) {
			const contentType = response.headers.get('content-type');
			if (contentType?.includes('application/json')) {
				try {
					const errorData = await response.json();
					const validatedError = ApiErrorSchema.safeParse(errorData);
					if (validatedError.success) {
						throw new ApiClientError(
							validatedError.data.error,
							response.status,
							validatedError.data
						);
					}
				} catch (e) {
					if (e instanceof ApiClientError) {
						throw e;
					}
					// Fall through to generic error
				}
			}
			throw new ApiClientError(
				`HTTP error ${response.status}: ${response.statusText}`,
				response.status
			);
		}

		// Parse and validate response
		const data = await response.json();
		const result = schema.safeParse(data);

		if (!result.success) {
			console.error('Schema validation failed:', result.error);
			throw new ApiClientError(
				'Response validation failed: ' + result.error.message,
				response.status
			);
		}

		return result.data;
	}
}

/**
 * Singleton API client instance
 */
export const apiClient = new ApiClient();
