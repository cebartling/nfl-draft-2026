import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { z } from 'zod';
import { ApiClient, ApiClientError } from './client';

describe('ApiClient', () => {
	let apiClient: ApiClient;
	let fetchMock: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		apiClient = new ApiClient('/api');
		fetchMock = vi.fn();
		globalThis.fetch = fetchMock as any;
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('constructor', () => {
		it('should use default baseUrl if not provided', () => {
			const client = new ApiClient();
			expect(client).toBeInstanceOf(ApiClient);
		});

		it('should use provided baseUrl', () => {
			const client = new ApiClient('/custom-api');
			expect(client).toBeInstanceOf(ApiClient);
		});
	});

	describe('GET requests', () => {
		it('should make successful GET request with valid response', async () => {
			const mockData = { id: '123', name: 'Test' };
			const schema = z.object({ id: z.string(), name: z.string() });

			fetchMock.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockData,
			});

			const result = await apiClient.get('/test', schema);

			expect(fetchMock).toHaveBeenCalledWith('/api/test', {
				method: 'GET',
				headers: {
					'Content-Type': 'application/json',
				},
			});
			expect(result).toEqual(mockData);
		});

		it('should throw ApiClientError on 404', async () => {
			const errorData = {
				error: 'Not found',
				details: 'Resource does not exist',
			};

			fetchMock.mockResolvedValue({
				ok: false,
				status: 404,
				statusText: 'Not Found',
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => errorData,
			});

			const schema = z.object({ id: z.string() });

			await expect(apiClient.get('/test', schema)).rejects.toThrow(ApiClientError);
			await expect(apiClient.get('/test', schema)).rejects.toMatchObject({
				status: 404,
				message: 'Not found',
			});
		});

		it('should throw ApiClientError on 500', async () => {
			fetchMock.mockResolvedValue({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				headers: new Headers({ 'content-type': 'text/html' }),
				json: async () => {
					throw new Error('Not JSON');
				},
			});

			const schema = z.object({ id: z.string() });

			await expect(apiClient.get('/test', schema)).rejects.toThrow(ApiClientError);
			await expect(apiClient.get('/test', schema)).rejects.toMatchObject({
				status: 500,
			});
		});

		it('should throw ApiClientError on network error', async () => {
			fetchMock.mockRejectedValueOnce(new Error('Network error'));

			const schema = z.object({ id: z.string() });

			await expect(apiClient.get('/test', schema)).rejects.toThrow('Network error');
		});

		it('should throw ApiClientError on schema validation failure', async () => {
			const mockData = { id: 123, name: 'Test' }; // id is number, not string
			const schema = z.object({ id: z.string(), name: z.string() });

			fetchMock.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockData,
			});

			await expect(apiClient.get('/test', schema)).rejects.toThrow(ApiClientError);
			await expect(apiClient.get('/test', schema)).rejects.toMatchObject({
				status: 200,
				message: expect.stringContaining('Response validation failed'),
			});
		});
	});

	describe('POST requests', () => {
		it('should make successful POST request with valid response', async () => {
			const requestData = { name: 'Test' };
			const mockResponse = { id: '123', name: 'Test' };
			const schema = z.object({ id: z.string(), name: z.string() });

			fetchMock.mockResolvedValue({
				ok: true,
				status: 201,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockResponse,
			});

			const result = await apiClient.post('/test', requestData, schema);

			expect(fetchMock).toHaveBeenCalledWith('/api/test', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify(requestData),
			});
			expect(result).toEqual(mockResponse);
		});

		it('should throw ApiClientError on 400', async () => {
			const requestData = { name: '' };
			const errorData = { error: 'Bad Request', details: 'Name is required' };

			fetchMock.mockResolvedValue({
				ok: false,
				status: 400,
				statusText: 'Bad Request',
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => errorData,
			});

			const schema = z.object({ id: z.string(), name: z.string() });

			await expect(apiClient.post('/test', requestData, schema)).rejects.toThrow(ApiClientError);
			await expect(apiClient.post('/test', requestData, schema)).rejects.toMatchObject({
				status: 400,
				message: 'Bad Request',
			});
		});

		it('should throw ApiClientError on 409 conflict', async () => {
			const requestData = { name: 'Duplicate' };
			const errorData = {
				error: 'Conflict',
				details: 'Resource already exists',
			};

			fetchMock.mockResolvedValue({
				ok: false,
				status: 409,
				statusText: 'Conflict',
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => errorData,
			});

			const schema = z.object({ id: z.string(), name: z.string() });

			await expect(apiClient.post('/test', requestData, schema)).rejects.toThrow(ApiClientError);
			await expect(apiClient.post('/test', requestData, schema)).rejects.toMatchObject({
				status: 409,
				message: 'Conflict',
			});
		});
	});

	describe('PUT requests', () => {
		it('should make successful PUT request with valid response', async () => {
			const requestData = { name: 'Updated' };
			const mockResponse = { id: '123', name: 'Updated' };
			const schema = z.object({ id: z.string(), name: z.string() });

			fetchMock.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockResponse,
			});

			const result = await apiClient.put('/test/123', requestData, schema);

			expect(fetchMock).toHaveBeenCalledWith('/api/test/123', {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify(requestData),
			});
			expect(result).toEqual(mockResponse);
		});

		it('should throw ApiClientError on 404 for non-existent resource', async () => {
			const requestData = { name: 'Updated' };
			const errorData = { error: 'Not found' };

			fetchMock.mockResolvedValue({
				ok: false,
				status: 404,
				statusText: 'Not Found',
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => errorData,
			});

			const schema = z.object({ id: z.string(), name: z.string() });

			await expect(apiClient.put('/test/999', requestData, schema)).rejects.toThrow(ApiClientError);
		});
	});

	describe('DELETE requests', () => {
		it('should make successful DELETE request with valid response', async () => {
			const mockResponse = { success: true };
			const schema = z.object({ success: z.boolean() });

			fetchMock.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockResponse,
			});

			const result = await apiClient.delete('/test/123', schema);

			expect(fetchMock).toHaveBeenCalledWith('/api/test/123', {
				method: 'DELETE',
				headers: {
					'Content-Type': 'application/json',
				},
			});
			expect(result).toEqual(mockResponse);
		});

		it('should throw ApiClientError on 404', async () => {
			const errorData = { error: 'Not found' };

			fetchMock.mockResolvedValue({
				ok: false,
				status: 404,
				statusText: 'Not Found',
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => errorData,
			});

			const schema = z.object({ success: z.boolean() });

			await expect(apiClient.delete('/test/999', schema)).rejects.toThrow(ApiClientError);
		});
	});

	describe('ApiClientError', () => {
		it('should create error with message and status', () => {
			const error = new ApiClientError('Test error', 404);

			expect(error.message).toBe('Test error');
			expect(error.status).toBe(404);
			expect(error.name).toBe('ApiClientError');
			expect(error.apiError).toBeUndefined();
		});

		it('should create error with apiError details', () => {
			const apiError = {
				error: 'Not found',
				details: 'Resource does not exist',
			};
			const error = new ApiClientError('Test error', 404, apiError);

			expect(error.message).toBe('Test error');
			expect(error.status).toBe(404);
			expect(error.apiError).toEqual(apiError);
		});

		it('should be instanceof Error', () => {
			const error = new ApiClientError('Test error', 404);
			expect(error).toBeInstanceOf(Error);
		});
	});
});
