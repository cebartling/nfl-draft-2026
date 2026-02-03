import { z } from 'zod';

// UUID schema and type
export const UUIDSchema = z.string().uuid();
export type UUID = z.infer<typeof UUIDSchema>;

// API Error schema
export const ApiErrorSchema = z.object({
	error: z.string(),
	details: z.string().optional(),
});
export type ApiError = z.infer<typeof ApiErrorSchema>;

// API Response schema
export const ApiResponseSchema = <T extends z.ZodTypeAny>(dataSchema: T) =>
	z.object({
		data: dataSchema.optional(),
		error: ApiErrorSchema.optional(),
	});
export type ApiResponse<T> = {
	data?: T;
	error?: ApiError;
};

// Paginated Response schema
export const PaginatedResponseSchema = <T extends z.ZodTypeAny>(itemSchema: T) =>
	z.object({
		items: z.array(itemSchema),
		total: z.number(),
		page: z.number(),
		per_page: z.number(),
	});
export type PaginatedResponse<T> = {
	items: T[];
	total: number;
	page: number;
	per_page: number;
};
